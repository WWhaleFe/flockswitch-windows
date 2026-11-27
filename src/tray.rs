//! Notification-area icon with a small menu. The edge watcher and the network service
//! run on their own threads; the settings window changes the shared config and they pick
//! the change up on their next loop.

use crate::engine::Engine;
use crate::i18n::{texts, Texts};
use crate::icon;
use crate::net::NetState;
use crate::shared::{Shared, Status};
use crate::update::UpdateState;
use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Clone)]
enum Action {
    Enabled,
    Channel(u8),
    Settings,
    Update,
    Quit,
}

struct Ui {
    shared: Arc<Shared>,
    tray: Option<TrayIcon>,
    actions: HashMap<MenuId, Action>,
    shown: (Status, u64, NetState),
    uninstall_requested: bool,
    shortcuts: Option<crate::hotkey::Shortcuts>,
}

pub fn run(shared: Arc<Shared>) -> Result<()> {
    // Ctrl+C, logout or shutdown: let the loop below stop the threads and exit.
    let s = shared.clone();
    ctrlc::set_handler(move || s.request_stop())?;
    // The settings window and a newer copy read the running version from here.
    shared.set_status(|_| {});
    let engine = spawn_engine(shared.clone());
    crate::edge::spawn(shared.clone());
    let mut network = Some(crate::net::spawn(shared.clone()));
    crate::update::spawn_periodic(shared.clone());

    let event_loop = EventLoopBuilder::new().build();

    let mut ui = Ui {
        shared: shared.clone(),
        tray: None,
        actions: HashMap::new(),
        shown: (Status::default(), u64::MAX, NetState::Off),
        uninstall_requested: false,
        shortcuts: None,
    };
    let mut engine = Some(engine);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(500));

        // The tray icon is created once the event loop is running.
        if let Event::NewEvents(StartCause::Init) = event {
            crate::overlay::set_ready();
            match ui.create_tray() {
                Ok(()) => {}
                Err(e) => {
                    log::error!("cannot create the tray icon: {e:#}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            match crate::hotkey::Shortcuts::new(shared.clone()) {
                Ok(s) => ui.shortcuts = Some(s),
                Err(e) => log::warn!("keyboard shortcuts unavailable: {e:#}"),
            }
        }
        // Follow the shortcut settings (and let go while the settings window records one).
        if let Some(s) = ui.shortcuts.as_mut() {
            if let Some(failed) = s.sync(&shared.config()) {
                shared.set_status(|st| st.hotkeys_failed = failed);
            }
        }

        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if let Some(action) = ui.actions.get(&ev.id).cloned() {
                if ui.handle(action) {
                    shared.request_stop();
                }
            }
        }
        // Uninstall asked for from the settings window or the uninstaller,
        // quit from the installer (before an upgrade).
        let request = crate::shared::uninstall_request_path();
        if request.exists() {
            let _ = std::fs::remove_file(&request);
            ui.uninstall_requested = true;
            shared.request_stop();
        }
        let request = crate::shared::quit_request_path();
        if request.exists() {
            let _ = std::fs::remove_file(&request);
            shared.request_stop();
        }
        let destroyed = matches!(event, Event::LoopDestroyed);
        if shared.stopped() || destroyed {
            shared.request_stop();
            // An open settings window has nothing left to talk to: close it.
            crate::settings_ui::tell_settings_to_quit();
            if let Some(h) = network.take() {
                let _ = h.join();
            }
            if let Some(h) = engine.take() {
                join_briefly(h); // lets the engine hand the button back first
            }
            if ui.uninstall_requested {
                crate::uninstall();
            }
            ui.tray = None;
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Redraw when the update state or the settings changed, or the network state did.
        let now = (shared.status(), shared.version(), NetState::of(&shared.config(), crate::net::live_status().as_ref()));
        if ui.tray.is_some() && now != ui.shown {
            ui.refresh();
        }
    });
}

/// Passed by the start at login (`flockswitch tray --login`, here and in the Store
/// package's startup task), so that start never opens the settings window.
pub const LOGIN_ARG: &str = "--login";

/// Start at login through the Run registry key. Not for the Store version: its package
/// declares a startup task instead (a Run entry written from a package is not used).
/// Waits for the engine to finish, but not forever: it may be in the middle of a slow
/// device request, and the UI must not hang at quit or logout.
fn join_briefly(h: JoinHandle<()>) {
    // Long enough for one slow device request plus handing the button back.
    let deadline = Instant::now() + Duration::from_secs(6);
    while !h.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    if h.is_finished() {
        let _ = h.join();
    }
}

fn spawn_engine(shared: Arc<Shared>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        while !shared.stopped() {
            // A panic must not leave the app looking alive with a dead engine: catch it,
            // let Engine's Drop hand the button back, and start again.
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Engine::new(shared.clone()).and_then(|mut e| e.run())
            }));
            let failure = match run {
                Ok(Ok(())) => break,
                Ok(Err(e)) => format!("{e:#}"),
                Err(_) => "internal error (panic)".to_string(),
            };
            log::error!("engine stopped: {failure}; restarting in 5 s");
            for _ in 0..50 {
                if shared.stopped() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    })
}

pub fn autostart() -> Option<AutoLaunch> {
    if crate::store::packaged() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    AutoLaunchBuilder::new()
        .set_app_name(crate::APP_NAME)
        // Quoted: the arguments follow the path, which may contain spaces.
        .set_app_path(&format!("\"{}\"", exe.to_string_lossy()))
        .set_args(&["tray", LOGIN_ARG])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .build()
        .ok()
}

/// Rewrites an existing start-at-login entry, so one written before 1.0.1 (the program
/// path alone) gains `tray --login` and no longer looks like the user starting it. It
/// then also points to this copy, the one that runs. One the user switched off stays off.
pub fn refresh_autostart() {
    if let Some(al) = autostart() {
        if al.is_enabled().unwrap_or(false) {
            if let Err(e) = al.enable() {
                log::warn!("could not update start at login: {e}");
            }
        }
    }
}

/// White arrows on a blue tile, visible on light and dark taskbars.
fn tray_icon() -> Result<Icon> {
    Ok(Icon::from_rgba(icon::arrows(32, [255, 255, 255, 255], Some([0x25, 0x63, 0xEB, 255])), 32, 32)?)
}

impl Ui {
    fn create_tray(&mut self) -> Result<()> {
        // The icon's own events (hover, click) are not used: the menu does everything.
        // Without a handler tray-icon queues every one of them, and a queue nobody reads
        // grows for as long as the app runs.
        TrayIconEvent::set_event_handler(Some(|_: TrayIconEvent| {}));
        let menu = self.build_menu();
        let builder = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip(self.tooltip());
        let builder = builder.with_icon(tray_icon()?);
        let tray = builder.build()?;
        self.tray = Some(tray);
        self.shown = self.snapshot();
        Ok(())
    }

    fn refresh(&mut self) {
        let menu = self.build_menu();
        let tooltip = self.tooltip();
        if let Some(tray) = &self.tray {
            tray.set_menu(Some(Box::new(menu)));
            let _ = tray.set_tooltip(Some(tooltip));
        }
        self.shown = self.snapshot();
    }

    /// What the menu shows, to tell when it needs rebuilding.
    fn snapshot(&self) -> (Status, u64, NetState) {
        (self.shared.status(), self.shared.version(), NetState::of(&self.shared.config(), crate::net::live_status().as_ref()))
    }

    fn check(&mut self, text: impl AsRef<str>, checked: bool, enabled: bool, action: Action) -> CheckMenuItem {
        let item = CheckMenuItem::new(text, enabled, checked, None);
        self.actions.insert(item.id().clone(), action);
        item
    }

    fn item(&mut self, text: impl AsRef<str>, action: Action) -> MenuItem {
        let item = MenuItem::new(text, true, None);
        self.actions.insert(item.id().clone(), action);
        item
    }

    /// The first menu line: what this computer is doing on the network.
    fn status_line(&self) -> String {
        let state = NetState::of(&self.shared.config(), crate::net::live_status().as_ref());
        let mark = if state == NetState::Off { "○" } else { "●" };
        format!("{mark} {}", state.text(texts()))
    }

    /// Hovering the icon tells the same, since the app starts without a window.
    fn tooltip(&self) -> String {
        format!("{}\n{}", crate::APP_NAME, self.status_line())
    }

    fn build_menu(&mut self) -> Menu {
        self.actions.clear();
        let t: &Texts = texts();
        let cfg = self.shared.config();
        let st = self.shared.status();
        let menu = Menu::new();
        let _ = menu.append(&MenuItem::new(self.status_line(), false, None));

        let status = if !cfg.enabled {
            format!("○ {}", t.paused)
        } else if let Some(trigger) = &st.trigger {
            format!("● {} - {trigger}", t.running)
        } else {
            format!("◌ {}", t.searching)
        };
        let _ = menu.append(&MenuItem::new(status, false, None));
        if let Some(last) = &st.last_switch {
            let _ = menu.append(&MenuItem::new(format!("{}: {} {last}", t.last_switch, t.channel), false, None));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());

        let enabled = self.check(t.enabled, cfg.enabled, true, Action::Enabled);
        let _ = menu.append(&enabled);
        // Everything else lives in the settings window, which stays open while you
        // change things (a native menu closes after every click).
        for channel in 1..=3 {
            let item = self.item(format!("{} {channel}", t.channel), Action::Channel(channel));
            let _ = menu.append(&item);
        }
        for device in &st.devices {
            let _ = menu.append(&MenuItem::new(format!("{}: {}", t.section_devices, device.name), false, None));
        }
        let settings = self.item(t.settings, Action::Settings);
        let _ = menu.append(&settings);
        let _ = menu.append(&PredefinedMenuItem::separator());

        let _ = menu.append(&MenuItem::new(format!("{} {}", crate::APP_NAME, env!("CARGO_PKG_VERSION")), false, None));
        match &st.update {
            UpdateState::Available(v) | UpdateState::InstallFailed(v) => {
                let item = self.item(format!("{} ({v})", t.update_install), Action::Update);
                let _ = menu.append(&item);
            }
            UpdateState::Installing(v) => {
                let _ = menu.append(&MenuItem::new(format!("{} ({v})", t.update_installing), false, None));
            }
            _ => {}
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
        let quit = self.item(t.quit, Action::Quit);
        let _ = menu.append(&quit);
        menu
    }

    /// Applies a menu action. Returns true when the app should quit.
    fn handle(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::Channel(channel) => self.shared.request_switch(channel),
            Action::Enabled => {
                if let Err(e) = self.shared.update_config(|c| c.enabled = !c.enabled) {
                    log::info!("setting not changed: {e:#}");
                }
            }
            Action::Settings => open_settings_window(),
            Action::Update if crate::store::packaged() => {
                // The Store version goes through the settings window, which owns the
                // Store dialog and falls back to the Store page.
                open_settings_window_with("update");
            }
            Action::Update => {
                if let UpdateState::Available(v) | UpdateState::InstallFailed(v) = self.shared.status().update {
                    let shared = self.shared.clone();
                    std::thread::spawn(move || crate::update::install_into(&shared, v));
                }
            }
        }
        self.refresh();
        false
    }
}

/// Starts the settings window as a separate process of this same program.
pub fn open_settings_window() {
    open_settings_window_with("");
}

/// The same, with an extra argument (`update` opens on the page with the new version).
fn open_settings_window_with(extra: &str) {
    match std::env::current_exe() {
        Ok(exe) => {
            let mut command = std::process::Command::new(exe);
            command.arg("settings");
            if !extra.is_empty() {
                command.arg(extra);
            }
            if let Err(e) = command.spawn() {
                log::warn!("cannot open the settings window: {e}");
            }
        }
        Err(e) => log::warn!("cannot open the settings window: {e}"),
    }
}

