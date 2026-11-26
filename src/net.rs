//! Windows input adapter and network service lifetime.
use std::sync::atomic::{AtomicBool, Ordering};
use crate::{
    config::{Config, Edge},
    shared::Shared,
};
use flock_net::{keymap, InputEvent, MouseButton, NetConfig, Os, Platform, Rect, Role, Service, Side};
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

pub fn config_dir() -> PathBuf {
    Config::base_dir()
}
/// The running service and the config it runs with, for the edge watcher.
struct Live {
    service: Arc<Service>,
    config: NetConfig,
}
static LIVE: Mutex<Option<Live>> = Mutex::new(None);

fn side_of(edge: Edge) -> Side {
    match edge {
        Edge::Left => Side::Left,
        Edge::Right => Side::Right,
        Edge::Top => Side::Top,
        Edge::Bottom => Side::Bottom,
    }
}

fn displays() -> Vec<Rect> {
    crate::edge::platform::displays()
        .into_iter()
        .map(|r| Rect::new(r.x, r.y, r.w, r.h))
        .collect()
}

/// Whether the cursor can cross over `edge` now: this computer is in control and a
/// connected, paired computer sits on that side.
pub fn network_side_ready(edge: Edge) -> bool {
    let live = LIVE.lock().unwrap();
    let Some(live) = live.as_ref() else { return false };
    let side = side_of(edge);
    let status = live.service.status();
    status.running
        && status.state.mode == "local"
        && live.config.peers.iter().any(|p| {
            p.side == Some(side) && status.peers.iter().any(|s| s.id == p.id && s.connected)
        })
}

/// Moves control to the computer on `edge`, entering at the spot matching (x, y).
pub fn cross(edge: Edge, x: f64, y: f64) {
    let live = LIVE.lock().unwrap();
    let Some(live) = live.as_ref() else { return };
    let side = side_of(edge);
    let pos = flock_net::geometry::edge_ratio(&displays(), side, x, y);
    let _ = live.service.cross(side, pos);
}

/// The effect settings a master hands to its subs: the countdown look and the cursor
/// highlight.
const OWN_EDGE_KEYS: [&str; 5] = ["left", "right", "top", "bottom", "on"];
fn app_settings(config: &Config) -> Value {
    let mut edges = serde_json::to_value(&config.edges).unwrap_or_default();
    if let Some(map) = edges.as_object_mut() {
        for key in OWN_EDGE_KEYS {
            map.remove(key);
        }
    }
    json!({ "edges": edges, "haptics": config.haptics, "locate": config.locate,
        "enabled": config.enabled, "trigger_device": config.trigger_device, "follow": config.follow,
        "keep_wheel_mode": config.keep_wheel_mode, "buttons": config.buttons, "hotkeys": config.hotkeys })
}

/// Puts the master's effect settings over a sub's own, keeping its channels and `edges.on`.
fn apply_app_settings(config: &mut Config, app: &Value) {
    macro_rules! read { ($($field:ident),*) => { $(
        if let Some(value) = app.get(stringify!($field)).and_then(|v| serde_json::from_value(v.clone()).ok()) { config.$field = value; }
    )* }; }
    read!(enabled, trigger_device, follow, keep_wheel_mode, buttons, hotkeys);
    if let Some(Value::Object(master)) = app.get("edges") {
        let mut edges = serde_json::to_value(&config.edges).unwrap_or_default();
        if let Some(map) = edges.as_object_mut() {
            for (key, value) in master {
                if !OWN_EDGE_KEYS.contains(&key.as_str()) {
                    map.insert(key.clone(), value.clone());
                }
            }
        }
        if let Ok(edges) = serde_json::from_value(edges) {
            config.edges = edges;
        }
    }
    if let Some(haptics) = app.get("haptics").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        config.haptics = haptics;
    }
    if let Some(locate) = app.get("locate").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        config.locate = locate;
    }
}

/// What this computer is doing on the network, for the header and the tray menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetState {
    Off,
    Idle,
    /// Controlling the computer of this name.
    Controlling(String),
    /// Being controlled by the computer of this name.
    Controlled(String),
}

impl NetState {
    /// From the settings and the service's status (None: no status yet).
    pub fn of(config: &Config, status: Option<&flock_net::Status>) -> NetState {
        let Some(status) = status.filter(|s| s.running && config.network.on && config.network.notice_accepted) else {
            return NetState::Off;
        };
        let name = || {
            let id = status.state.peer.as_deref().unwrap_or_default();
            config.network.peers.iter().find(|p| p.id == id).map_or_else(|| id.to_string(), |p| p.name.clone())
        };
        match status.state.mode.as_str() {
            "controlling" => NetState::Controlling(name()),
            "controlled" => NetState::Controlled(name()),
            _ => NetState::Idle,
        }
    }
}

impl NetState {
    /// The line the header and the tray menu show.
    pub fn text(&self, t: &crate::i18n::Texts) -> String {
        match self {
            NetState::Off => t.net_state_off.to_string(),
            NetState::Idle => t.net_state_idle.to_string(),
            NetState::Controlling(name) => format!("{}{name}{}", t.net_state_controlling_pre, t.net_state_controlling_post),
            NetState::Controlled(name) => format!("{}{name}{}", t.net_state_controlled_pre, t.net_state_controlled_post),
        }
    }
}

/// The running service's status, for the tray menu (which lives in the same process).
pub fn live_status() -> Option<flock_net::Status> {
    LIVE.lock().unwrap().as_ref().map(|live| live.service.status())
}

/// The network config the service runs with: sharing only after consent, the master's
/// effect settings attached, and the app driving the edges when there is a wait.
fn runtime_config(shared: &Shared) -> NetConfig {
    let full = shared.config();
    let mut config = full.network.clone();
    config.config_dir = config_dir();
    config.on &= config.notice_accepted;
    config.app_drives_edges = full.edges.on || config.cross_delay_ms > 0;
    if config.role == Role::Master {
        config.app_settings = app_settings(&full);
    }
    config
}

pub fn spawn(shared: Arc<Shared>) -> JoinHandle<()> {
    thread::Builder::new()
        .name("network".into())
        .spawn(move || {
            let platform = match WindowsPlatform::new() {
                Ok(p) => Arc::new(p),
                Err(e) => {
                    log::error!("initialize network input: {e:#}");
                    return;
                }
            };
            let mut service: Option<Arc<Service>> = None;
            let mut mode = String::from("local");
            let mut previous: Option<NetConfig> = None;
            let mut retry = Instant::now() - Duration::from_secs(5);
            while !shared.stopped() {
                shared.reload_if_edited();
                let version = shared.version();
                let config = runtime_config(&shared);
                platform.edges_allowed.store(shared.config().network_edges_allowed(), Ordering::Release);
                if !config.on {
                    LIVE.lock().unwrap().take();
                    mode = "local".into();
                    if let Some(s) = service.take() {
                        s.stop();
                    }
                } else {
                    if service.is_none() && retry.elapsed() >= Duration::from_secs(5) {
                        retry = Instant::now();
                        let paired = shared.clone();
                        let unpaired = shared.clone();
                        let changed = shared.clone();
                        let master = shared.clone();
                        // No Logitech channels here: `switch_logitech` stays the no-op default.
                        let hooks = flock_net::Hooks {
                            on_paired: Box::new(move |peer| {
                                if let Err(e) = paired.update_config(|c| {
                                    c.network.peers.retain(|p| p.id != peer.id);
                                    c.network.peers.push(peer);
                                }) {
                                    log::error!("save paired computer: {e:#}");
                                }
                            }),
                            on_unpaired: Box::new(move |id| {
                                if let Err(e) = unpaired
                                    .update_config(|c| c.network.peers.retain(|p| p.id != id))
                                {
                                    log::error!("save unpaired computer: {e:#}");
                                }
                            }),
                            on_peer_changed: Box::new(move |peer| {
                                if let Err(e) = changed.update_config(|c| {
                                    match c.network.peers.iter_mut().find(|p| p.id == peer.id) {
                                        Some(slot) => *slot = peer,
                                        None => c.network.peers.push(peer),
                                    }
                                }) {
                                    log::error!("save computer: {e:#}");
                                }
                            }),
                            on_master_settings: Box::new(move |settings| {
                                if let Err(e) = master.update_config(|c| {
                                    settings.apply_to(&mut c.network);
                                    apply_app_settings(c, &settings.app);
                                }) {
                                    log::error!("save master settings: {e:#}");
                                }
                            }),
                            ..flock_net::Hooks::default()
                        };
                        match Service::start(config.clone(), platform.clone(), hooks) {
                            Ok(s) => {
                                let s = Arc::new(s);
                                *LIVE.lock().unwrap() =
                                    Some(Live { service: s.clone(), config: config.clone() });
                                service = Some(s);
                            }
                            Err(e) => {
                                log::error!("start network: {e:#}");
                                let status = flock_net::Status {
                                    running: false,
                                    me: flock_net::MeStatus {
                                        id: String::new(),
                                        name: config.name.clone(),
                                        port: config.port,
                                        error: Some(format!("{e:#}")),
                                    },
                                    discovered: vec![],
                                    peers: vec![],
                                    state: flock_net::StateStatus {
                                        mode: "local".into(),
                                        peer: None,
                                    },
                                    pairing: None,
                                    transfer: None,
                                    following: None,
                                };
                                let _ = flock_net::write_status(config_dir(), &status);
                            }
                        }
                    } else if previous.as_ref() != Some(&config) {
                        if let Some(s) = &service {
                            let _ = s.update_config(config.clone());
                            if let Some(live) = LIVE.lock().unwrap().as_mut() {
                                live.config = config.clone();
                            }
                        }
                    }
                }
                previous = Some(config);
                for _ in 0..5 {
                    if shared.stopped() || shared.version() != version {
                        break;
                    }
                    thread::sleep(Duration::from_millis(50));
                    if let Some(s) = &service {
                        let now = s.status().state.mode;
                        // The cursor arrived here, or came back here.
                        let arrived = (mode == "local" && now == "controlled")
                            || (mode == "controlling" && now == "local");
                        if arrived {
                            let locate = shared.config().locate;
                            if locate.on && locate.on_arrival {
                                crate::overlay::locate(&locate);
                            }
                        }
                        mode = now;
                    }
                }
            }
            LIVE.lock().unwrap().take();
            if let Some(s) = service {
                s.stop();
            }
            platform.stop_capture();
        })
        .expect("spawning network service")
}

struct Capture {
    sink: Sender<InputEvent>,
    park: (i32, i32),
}
thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
unsafe extern "system" fn mouse_hook(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let event = &*(data as *const MSLLHOOKSTRUCT);
    if event.flags & LLMHF_INJECTED != 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let captured = CAPTURE.with_borrow(|capture| {
        let Some(c) = capture else {
            return false;
        };
        let kind = message as u32;
        let wheel = (event.mouseData >> 16) as i16 as f64 / 120.0;
        let ev = match kind {
            WM_MOUSEMOVE => Some(InputEvent::Motion {
                dx: (event.pt.x - c.park.0) as f64,
                dy: (event.pt.y - c.park.1) as f64,
            }),
            WM_MOUSEWHEEL => Some(InputEvent::Scroll { dx: 0.0, dy: wheel }),
            // The wire uses positive left, Windows uses positive right.
            WM_MOUSEHWHEEL => Some(InputEvent::Scroll {
                dx: -wheel,
                dy: 0.0,
            }),
            _ => {
                let button = match kind {
                    WM_LBUTTONDOWN | WM_LBUTTONUP => Some(MouseButton::Left),
                    WM_RBUTTONDOWN | WM_RBUTTONUP => Some(MouseButton::Right),
                    WM_MBUTTONDOWN | WM_MBUTTONUP => Some(MouseButton::Middle),
                    WM_XBUTTONDOWN | WM_XBUTTONUP => match event.mouseData >> 16 {
                        1 => Some(MouseButton::Back),
                        2 => Some(MouseButton::Forward),
                        _ => None,
                    },
                    _ => None,
                };
                button.map(|button| InputEvent::Button {
                    button,
                    down: matches!(
                        kind,
                        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
                    ),
                })
            }
        };
        if let Some(ev) = ev {
            let _ = c.sink.send(ev);
        }
        true
    });
    if captured {
        1
    } else {
        CallNextHookEx(null_mut(), code, message, data)
    }
}
unsafe extern "system" fn keyboard_hook(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let event = &*(data as *const KBDLLHOOKSTRUCT);
    if event.flags & LLKHF_INJECTED != 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let captured = CAPTURE.with_borrow(|capture| {
        let Some(c) = capture else {
            return false;
        };
        if let Some(usage) = keymap::windows_key_to_hid(
            event.vkCode as u16,
            keymap::ScanCode::new(event.scanCode as u16, event.flags & LLKHF_EXTENDED != 0),
        ) {
            let _ = c.sink.send(InputEvent::Key {
                usage,
                down: event.flags & LLKHF_UP == 0,
            });
        }
        true
    });
    if captured {
        1
    } else {
        CallNextHookEx(null_mut(), code, message, data)
    }
}
fn restore_cursors() {
    unsafe {
        SystemParametersInfoW(SPI_SETCURSORS, 0, null_mut(), 0);
    }
}
fn hide_cursors() {
    // SetSystemCursor takes ownership of each cursor.
    for id in [
        OCR_NORMAL,
        OCR_IBEAM,
        OCR_WAIT,
        OCR_CROSS,
        OCR_UP,
        OCR_SIZENWSE,
        OCR_SIZENESW,
        OCR_SIZEWE,
        OCR_SIZENS,
        OCR_SIZEALL,
        OCR_NO,
        OCR_HAND,
        OCR_APPSTARTING,
        OCR_HELP,
    ] {
        unsafe {
            let and = [0xffu8; 128];
            let xor = [0u8; 128];
            let cursor = CreateCursor(
                null_mut(),
                0,
                0,
                32,
                32,
                and.as_ptr().cast(),
                xor.as_ptr().cast(),
            );
            if !cursor.is_null() && SetSystemCursor(cursor, id) == 0 {
                DestroyCursor(cursor);
            }
        }
    }
}
enum Command {
    Start(Sender<InputEvent>, Sender<()>),
    Stop(Sender<()>),
    Quit,
}
struct WindowsPlatform {
    commands: Sender<Command>,
    thread_id: u32,
    handle: Mutex<Option<JoinHandle<()>>>,
    scroll: Mutex<(f64, f64)>,
    edges_allowed: AtomicBool,
}
impl WindowsPlatform {
    fn new() -> anyhow::Result<Self> {
        restore_cursors();
        let (commands, receive) = mpsc::channel();
        let (ready, initialized) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name("network-input-hooks".into())
            .spawn(move || unsafe {
                // The executable manifest sets PerMonitorV2 for all native threads.
                let mut message: MSG = std::mem::zeroed();
                PeekMessageW(&mut message, null_mut(), 0, 0, PM_NOREMOVE);
                let module = GetModuleHandleW(null());
                let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), module, 0);
                let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module, 0);
                if mouse.is_null() || keyboard.is_null() {
                    let error = std::io::Error::last_os_error();
                    if !mouse.is_null() {
                        UnhookWindowsHookEx(mouse);
                    }
                    if !keyboard.is_null() {
                        UnhookWindowsHookEx(keyboard);
                    }
                    let _ = ready.send(Err(error));
                    return;
                }
                let _ = ready.send(Ok(GetCurrentThreadId()));
                'messages: while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                    while let Ok(command) = receive.try_recv() {
                        match command {
                            Command::Start(sink, ack) => {
                                CAPTURE.with_borrow_mut(|c| *c = None);
                                let park = (
                                    GetSystemMetrics(SM_CXSCREEN) / 2,
                                    GetSystemMetrics(SM_CYSCREEN) / 2,
                                );
                                crate::edge::platform::place(park.0 as f64, park.1 as f64);
                                hide_cursors();
                                // Include modifiers already held when the cursor crossed.
                                for (vk, usage) in [
                                    (VK_LCONTROL, 0xe0),
                                    (VK_LSHIFT, 0xe1),
                                    (VK_LMENU, 0xe2),
                                    (VK_LWIN, 0xe3),
                                    (VK_RCONTROL, 0xe4),
                                    (VK_RSHIFT, 0xe5),
                                    (VK_RMENU, 0xe6),
                                    (VK_RWIN, 0xe7),
                                ] {
                                    if GetAsyncKeyState(vk as i32) < 0 {
                                        let _ = sink.send(InputEvent::Key { usage, down: true });
                                    }
                                }
                                CAPTURE.with_borrow_mut(|c| *c = Some(Capture { sink, park }));
                                let _ = ack.send(());
                            }
                            Command::Stop(ack) => {
                                CAPTURE.with_borrow_mut(|c| *c = None);
                                restore_cursors();
                                let _ = ack.send(());
                            }
                            Command::Quit => break 'messages,
                        }
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                CAPTURE.with_borrow_mut(|c| *c = None);
                UnhookWindowsHookEx(mouse);
                UnhookWindowsHookEx(keyboard);
                restore_cursors();
            })?;
        match initialized.recv()? {
            Ok(thread_id) => Ok(Self {
                commands,
                thread_id,
                handle: Mutex::new(Some(handle)),
                scroll: Mutex::new((0.0, 0.0)),
                edges_allowed: AtomicBool::new(false),
            }),
            Err(error) => {
                let _ = handle.join();
                Err(error.into())
            }
        }
    }
    fn command(&self, command: Command) -> anyhow::Result<()> {
        self.commands.send(command)?;
        anyhow::ensure!(
            unsafe { PostThreadMessageW(self.thread_id, WM_APP, 0, 0) } != 0,
            "Cannot wake input hook thread"
        );
        Ok(())
    }
}
fn mouse_input(flags: u32, data: i32, dx: i32, dy: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data as u32,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
impl Platform for WindowsPlatform {
    fn os(&self) -> Os {
        Os::Windows
    }
    fn displays(&self) -> Vec<Rect> {
        displays()
    }
    fn cursor(&self) -> Option<(f64, f64)> {
        if !self.edges_allowed.load(Ordering::Acquire) { return None; }
        crate::edge::platform::cursor()
    }
    fn warp(&self, x: f64, y: f64) {
        crate::edge::platform::place(x, y);
    }
    fn control_down(&self) -> bool {
        crate::edge::platform::control_down()
    }
    fn clipboard_text(&self) -> Option<String> {
        arboard::Clipboard::new().ok()?.get_text().ok()
    }
    fn set_clipboard_text(&self, text: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }
    fn clipboard_files(&self) -> Vec<PathBuf> {
        arboard::Clipboard::new()
            .ok()
            .and_then(|mut c| c.get().file_list().ok())
            .unwrap_or_default()
    }
    fn set_clipboard_files(&self, files: &[PathBuf]) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set().file_list(files);
        }
    }
    fn clipboard_image_png(&self) -> Option<Vec<u8>> {
        let image = arboard::Clipboard::new().ok()?.get_image().ok()?;
        encode_png(image.width, image.height, &image.bytes)
    }
    fn set_clipboard_image_png(&self, data: &[u8]) {
        let Some((width, height, bytes)) = decode_png(data) else { return };
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_image(arboard::ImageData { width, height, bytes: bytes.into() });
        }
    }
    fn received_dir(&self) -> Option<PathBuf> {
        Some(dirs::download_dir()?.join("flockswitch2"))
    }
    fn ready(&self) -> Result<(), String> {
        Ok(())
    }
    fn start_capture(&self, sink: Sender<InputEvent>) -> anyhow::Result<()> {
        anyhow::ensure!(self.edges_allowed.load(Ordering::Acquire), "Network edge crossing is paused");
        let (ack, done) = mpsc::channel();
        self.command(Command::Start(sink, ack))?;
        done.recv()?;
        Ok(())
    }
    fn stop_capture(&self) {
        let (ack, done) = mpsc::channel();
        if self.command(Command::Stop(ack)).is_ok() {
            let _ = done.recv();
        }
    }
    fn inject(&self, event: &InputEvent) {
        let mut inputs = Vec::with_capacity(2);
        match *event {
            InputEvent::Motion { dx, dy } => {
                let Some((x, y)) = crate::edge::platform::cursor() else {
                    return;
                };
                let Some((x, y)) = flock_net::geometry::clamp(&self.displays(), x + dx, y + dy)
                else {
                    return;
                };
                unsafe {
                    let normalize = flock_net::windows_input::absolute_coordinate;
                    inputs.push(mouse_input(
                        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                        0,
                        normalize(
                            x,
                            GetSystemMetrics(SM_XVIRTUALSCREEN),
                            GetSystemMetrics(SM_CXVIRTUALSCREEN),
                        ),
                        normalize(
                            y,
                            GetSystemMetrics(SM_YVIRTUALSCREEN),
                            GetSystemMetrics(SM_CYVIRTUALSCREEN),
                        ),
                    ));
                }
            }
            InputEvent::Button { button, down } => {
                let (press, release, data) = match button {
                    MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, 0),
                    MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, 0),
                    MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 0),
                    MouseButton::Back => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, 1),
                    MouseButton::Forward => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, 2),
                };
                inputs.push(mouse_input(if down { press } else { release }, data, 0, 0));
            }
            InputEvent::Scroll { dx, dy } => {
                let mut scroll = self.scroll.lock().unwrap();
                let (horizontal, vertical) = &mut *scroll;
                for (remainder, notches, flag) in [
                    (horizontal, -dx, MOUSEEVENTF_HWHEEL),
                    (vertical, dy, MOUSEEVENTF_WHEEL),
                ] {
                    let amount = flock_net::windows_input::wheel_units(remainder, notches);
                    if amount != 0 {
                        inputs.push(mouse_input(flag, amount, 0, 0));
                    }
                }
            }
            InputEvent::Key { usage, down } => {
                let Some(scan) = keymap::hid_to_windows(usage) else {
                    return;
                };
                let vk = keymap::hid_to_windows_vk(usage);
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk.unwrap_or(0),
                            wScan: if vk.is_some() { 0 } else { scan.code },
                            dwFlags: if vk.is_some() {
                                0
                            } else {
                                KEYEVENTF_SCANCODE
                                    | if scan.extended {
                                        KEYEVENTF_EXTENDEDKEY
                                    } else {
                                        0
                                    }
                            } | if down { 0 } else { KEYEVENTF_KEYUP },
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }
        }
        if !inputs.is_empty() {
            let sent = unsafe {
                SendInput(
                    inputs.len() as u32,
                    inputs.as_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                )
            };
            if sent != inputs.len() as u32 {
                log::debug!("SendInput blocked or incomplete: {sent}/{}", inputs.len());
            }
        }
    }
}
impl Drop for WindowsPlatform {
    fn drop(&mut self) {
        let _ = self.command(Command::Quit);
        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
        restore_cursors();
    }
}
/// RGBA pixels as a PNG file.
fn encode_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, u32::try_from(width).ok()?, u32::try_from(height).ok()?);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().ok()?.write_image_data(rgba).ok()?;
    Some(out)
}
/// A PNG file as width, height and RGBA pixels.
fn decode_png(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    let (width, height) = (info.width as usize, info.height as usize);
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => buf,
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            buf.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect()
        }
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
            buf.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect()
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            buf.iter().flat_map(|&g| [g, g, g, 255]).collect()
        }
        _ => return None,
    };
    Some((width, height, rgba))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn png_round_trip() {
        let pixels: Vec<u8> = (0..2 * 3 * 4).map(|i| i as u8).collect();
        let png = encode_png(2, 3, &pixels).unwrap();
        assert_eq!(decode_png(&png), Some((2, 3, pixels)));
        assert_eq!(decode_png(b"not a png"), None);
    }
    #[test]
    fn effect_settings_follow_the_master() {
        let mut master = Config::default();
        master.edges.anim_style = crate::config::EdgeAnimStyle::Bar;
        master.locate.on = true;
        let app = app_settings(&master);
        let mut sub = Config::default();
        apply_app_settings(&mut sub, &app);
        assert_eq!(sub.edges.anim_style, crate::config::EdgeAnimStyle::Bar);
        assert_eq!(sub.edges.effects, master.edges.effects);
        assert_eq!(sub.locate, master.locate);
        // Haptics follow the master; edge channels and ownership stay local.
        let mut old = app.clone();
        old["haptics"] = json!({ "intensity": 60 });
        old["edges"]["left"] = json!(2);
        let mut sub = Config::default();
        apply_app_settings(&mut sub, &old);
        assert_eq!(sub.edges, master.edges);
        assert_eq!(sub.haptics.intensity, 60);
        sub.edges.on = true;
        sub.edges.right = 3;
        apply_app_settings(&mut sub, &old);
        assert!(sub.edges.on);
        assert_eq!(sub.edges.right, 3);
        for key in OWN_EDGE_KEYS { assert!(app["edges"].get(key).is_none()); }
        assert_eq!(sub.buttons, master.buttons);
        assert_eq!(sub.hotkeys, master.hotkeys);
    }
    #[test]
    fn logitech_owner_pauses_immediate_and_delayed_network_edges() {
        for delay in [0, 500] {
            let mut c = Config::default();
            c.network.on = true;
            c.network.notice_accepted = true;
            c.network.share_clipboard = true;
            c.network.cross_delay_ms = delay;
            c.edges.on = true;
            let shared = Shared::new(c);
            let runtime = runtime_config(&shared);
            assert!(runtime.on && runtime.app_drives_edges && runtime.share_clipboard);
            assert!(!shared.config().network_edges_allowed());
        }
    }
    #[test]
    fn network_state_text_source() {
        let mut config = Config::default();
        config.network.on = true;
        config.network.notice_accepted = true;
        config.network.peers.push(flock_net::PeerConfig { id: "p1".into(), name: "Laptop".into(), ..Default::default() });
        let status = |mode: &str, peer: Option<&str>| flock_net::Status {
            running: true,
            me: flock_net::MeStatus { id: String::new(), name: String::new(), port: 0, error: None },
            discovered: vec![],
            peers: vec![],
            state: flock_net::StateStatus { mode: mode.into(), peer: peer.map(Into::into) },
            pairing: None,
            transfer: None,
            following: None,
        };
        assert_eq!(NetState::of(&config, None), NetState::Off);
        assert_eq!(NetState::of(&config, Some(&status("local", None))), NetState::Idle);
        assert_eq!(NetState::of(&config, Some(&status("controlling", Some("p1")))), NetState::Controlling("Laptop".into()));
        assert_eq!(NetState::of(&config, Some(&status("controlled", Some("p1")))), NetState::Controlled("Laptop".into()));
        config.network.on = false;
        assert_eq!(NetState::of(&config, Some(&status("local", None))), NetState::Off);
    }
}

/// Start the tray executable, which takes its usual single-instance lock.
pub fn launch_background() -> anyhow::Result<()> {
    crate::update::hidden(&std::env::current_exe()?.to_string_lossy()).arg("tray").spawn()?;
    Ok(())
}
