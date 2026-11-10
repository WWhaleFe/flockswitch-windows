//! Animations drawn over the desktop: the countdown at a screen edge and the highlight
//! that shows where the cursor is. The effects themselves are the table of drawing
//! functions in overlay_render.rs, one per `EdgeAnimStyle` / `LocateStyle`; this file
//! puts them on the screen in click-through layered windows, updated at about 60 Hz by
//! one thread of its own. Windows exist only while an effect is showing.

use crate::config::{parse_color, Edge, Edges, Locate, DEFAULT_EFFECT_COLOR};
use crate::edge::Rect;
use crate::overlay_render as render;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const FRAME: Duration = Duration::from_micros(16_667);

enum Command {
    EdgeBegin { display: Rect, edge: i32, style: i32, direction: i32, shrink: bool, elapsed: Duration, duration: Duration, preview: bool, color: [u8; 3], size: f64 },
    EdgeEnd,
    Locate(i32, [u8; 3], f64),
}

static COMMANDS: OnceLock<Sender<Command>> = OnceLock::new();

/// Says this process has an interface and may draw. Without it (`flockswitch run`, as on
/// macOS) no thread is started and nothing is drawn.
pub fn set_ready() {
    COMMANDS.get_or_init(|| {
        let (commands, received) = channel();
        let started = std::thread::Builder::new().name("overlay".into()).spawn(move || run(received));
        if let Err(e) = started {
            log::warn!("overlay: cannot start its thread: {e}");
        }
        commands
    });
}

/// A colour from the settings as red, green and blue.
fn components(text: &str) -> [u8; 3] {
    parse_color(text).or(parse_color(DEFAULT_EFFECT_COLOR)).unwrap_or([255, 204, 0])
}

fn send(command: Command) {
    if let Some(commands) = COMMANDS.get() {
        let _ = commands.send(command);
    }
}

/// Starts the countdown on `edge` of `display`, `elapsed` into the wait of `duration`,
/// drawn as the chosen effect's own settings say (Random picks an effect, and its
/// options, each time).
pub fn edge_begin(edges: &Edges, edge: Edge, display: &Rect, elapsed: Duration, duration: Duration, preview: bool) {
    let Some(look) = edges.look() else {
        edge_end();
        return;
    };
    send(Command::EdgeBegin {
        display: *display,
        edge: edge as i32,
        style: look.style as i32,
        direction: look.direction as i32,
        shrink: look.shrink,
        elapsed,
        duration,
        preview,
        color: components(&look.color),
        size: f64::from(look.size) / 100.0,
    });
}

/// Takes the countdown away at once.
pub fn edge_end() {
    send(Command::EdgeEnd);
}

/// Shows where the cursor is, for a moment.
pub fn locate(locate: &Locate) {
    send(Command::Locate(locate.style.resolved() as i32, components(&locate.color), f64::from(locate.size) / 100.0));
}

/// An effect in progress. `Layer` goes when it does.
struct EdgeEffect {
    layer: platform::Layer,
    edge: i32,
    style: i32,
    direction: i32,
    shrink: bool,
    preview: bool,
    started: Instant,
    duration: f64,
    color: [u8; 3],
    /// The chosen size (1 = 100%).
    size: f64,
    scale: f64,
}

struct LocateEffect {
    layer: platform::Layer,
    style: i32,
    full_screen: bool,
    started: Instant,
    color: [u8; 3],
    size: f64,
    scale: f64,
}

#[derive(Default)]
struct Effects {
    edge: Option<EdgeEffect>,
    locate: Option<LocateEffect>,
}

impl Effects {
    fn handle(&mut self, command: Command) {
        match command {
            Command::EdgeBegin { display, edge, style, direction, shrink, elapsed, duration, preview, color, size } => {
                self.edge = None;
                if render::edge_has_effect(style) {
                    self.edge = begin_edge(&display, edge, style, direction, shrink, elapsed, duration, preview, color, size);
                }
            }
            Command::EdgeEnd => self.edge = None,
            Command::Locate(style, color, size) => {
                self.locate = None;
                if render::locate_exists(style) {
                    self.locate = begin_locate(style, color, size);
                }
            }
        }
    }

    fn animating(&self) -> bool {
        self.edge.is_some() || self.locate.is_some()
    }

    fn tick(&mut self) {
        if self.edge.as_mut().is_some_and(|e| !draw_edge(e)) {
            self.edge = None;
        }
        if self.locate.as_mut().is_some_and(|l| !draw_locate(l)) {
            self.locate = None;
        }
    }
}

fn run(commands: Receiver<Command>) {
    let mut effects = Effects::default();
    let mut next_frame = Instant::now();
    loop {
        let received = if effects.animating() {
            commands.recv_timeout(next_frame.saturating_duration_since(Instant::now()))
        } else {
            commands.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match received {
            Ok(command) => effects.handle(command),
            Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
        while let Ok(command) = commands.try_recv() {
            effects.handle(command);
        }
        platform::pump_messages();
        if Instant::now() >= next_frame {
            effects.tick();
            next_frame = Instant::now() + FRAME;
        }
    }
}

/// Where the strip along `edge` of `display` goes, in desktop pixels.
fn strip_of(display: &Rect, edge: i32, depth: i32) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = (display.x as i32, display.y as i32, display.w as i32, display.h as i32);
    match edge {
        render::EDGE_LEFT => (x, y, depth, h),
        render::EDGE_RIGHT => (x + w - depth, y, depth, h),
        render::EDGE_TOP => (x, y, w, depth),
        _ => (x, y + h - depth, w, depth),
    }
}

#[allow(clippy::too_many_arguments)]
fn begin_edge(display: &Rect, edge: i32, style: i32, direction: i32, shrink: bool, elapsed: Duration, duration: Duration, preview: bool, color: [u8; 3], size: f64) -> Option<EdgeEffect> {
    let center = (display.x + display.w / 2.0, display.y + display.h / 2.0);
    let scale = platform::scale_at(center.0 as i32, center.1 as i32);
    let depth = (render::STRIP_DEPTH * scale).round() as i32;
    let (x, y, w, h) = strip_of(display, edge, depth);
    let layer = platform::Layer::new(x, y, w, h)?;
    let started = Instant::now().checked_sub(elapsed).unwrap_or_else(Instant::now);
    let duration = duration.as_secs_f64().max(0.05);
    Some(EdgeEffect { layer, edge, style, direction, shrink, preview, started, duration, color, size, scale })
}

/// Draws the next frame of the countdown. False when it has finished.
fn draw_edge(effect: &mut EdgeEffect) -> bool {
    let t = effect.started.elapsed().as_secs_f64();
    if effect.preview && t >= effect.duration {
        return false;
    }
    let (x, y) = effect.layer.position();
    let (mx, my) = platform::cursor().unwrap_or((x, y));
    let draw = render::EdgeDraw {
        edge: effect.edge,
        style: effect.style,
        direction: effect.direction,
        shrink: effect.shrink,
        cursor: (f64::from(mx - x), f64::from(my - y)),
        scale: effect.scale,
        t,
        duration: effect.duration,
        color: effect.color,
        size: effect.size,
    };
    effect.layer.draw(|pixmap| render::draw_edge(pixmap, &draw));
    true
}

fn begin_locate(style: i32, color: [u8; 3], size: f64) -> Option<LocateEffect> {
    let (mx, my) = platform::cursor()?;
    let scale = platform::scale_at(mx, my);
    // Only the spotlight darkens the whole display; the others follow the cursor.
    let full_screen = style == render::LOCATE_SPOTLIGHT;
    let (x, y, w, h) = if full_screen {
        platform::monitor_at(mx, my)?
    } else {
        let side = (render::LOCATE_SIDE * scale * size).round() as i32;
        (mx - side / 2, my - side / 2, side, side)
    };
    let layer = platform::Layer::new(x, y, w, h)?;
    Some(LocateEffect { layer, style, full_screen, started: Instant::now(), color, size, scale })
}

/// Draws the next frame of the highlight. False when it has finished.
fn draw_locate(effect: &mut LocateEffect) -> bool {
    let t = effect.started.elapsed().as_secs_f64();
    if t >= render::LOCATE_SECONDS {
        return false;
    }
    let cursor = platform::cursor();
    let (x, y) = effect.layer.position();
    let (w, h) = effect.layer.size();
    let center = if effect.full_screen {
        let (mx, my) = cursor.unwrap_or((x + w / 2, y + h / 2));
        (f64::from(mx - x), f64::from(my - y))
    } else {
        if let Some((mx, my)) = cursor {
            effect.layer.move_to(mx - w / 2, my - h / 2);
        }
        (f64::from(w) / 2.0, f64::from(h) / 2.0)
    };
    let (style, scale, color, size) = (effect.style, effect.scale, effect.color, effect.size);
    effect.layer.draw(|pixmap| render::draw_locate(pixmap, style, center, scale, t, color, size));
    true
}

mod platform {
    use std::ptr::{null, null_mut};
    use std::sync::Once;
    use tiny_skia::{Color, Pixmap};
    use windows_sys::Win32::Foundation::{HMODULE, HWND, POINT, SIZE};
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetMonitorInfoW, MonitorFromPoint, SelectObject,
        AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
        HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, PeekMessageW, RegisterClassExW,
        ShowWindow, UpdateLayeredWindow, MSG, PM_REMOVE, SW_SHOWNOACTIVATE, ULW_ALPHA, WNDCLASSEXW, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain([0]).collect()
    }

    fn module() -> HMODULE {
        // Safety: a null name asks for this executable's handle; nothing is kept.
        unsafe { GetModuleHandleW(null()) }
    }

    fn class_name() -> Vec<u16> {
        wide("flockswitch-overlay")
    }

    fn register_class() {
        static REGISTERED: Once = Once::new();
        REGISTERED.call_once(|| {
            let name = class_name();
            // Safety: the class only forwards to the default window procedure.
            unsafe {
                let class = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    lpfnWndProc: Some(DefWindowProcW),
                    hInstance: module(),
                    lpszClassName: name.as_ptr(),
                    ..std::mem::zeroed()
                };
                RegisterClassExW(&class);
            }
        });
    }

    /// Where the cursor is now. Physical pixels: the app is per-monitor DPI aware.
    pub fn cursor() -> Option<(i32, i32)> {
        let mut p = POINT { x: 0, y: 0 };
        // Safety: a plain out-parameter.
        (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x, p.y))
    }

    /// Scale factor (physical pixels per point) of the monitor at a point.
    pub fn scale_at(x: i32, y: i32) -> f64 {
        let (mut dpi_x, mut dpi_y) = (96u32, 96u32);
        // Safety: the monitor handle comes straight from the system; plain out-parameters.
        unsafe { GetDpiForMonitor(MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST), MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
        f64::from(dpi_x.max(96)) / 96.0
    }

    /// The monitor at a point, as x, y, width, height in desktop pixels.
    pub fn monitor_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        // Safety: `cbSize` tells the size of the out-parameter.
        unsafe {
            let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return None;
            }
            let r = info.rcMonitor;
            Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
        }
    }

    /// Runs what the system queued for the layered windows of this thread.
    pub fn pump_messages() {
        // Safety: plain message pumping on the thread that owns the windows.
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                DispatchMessageW(&msg);
            }
        }
    }

    /// A click-through layered window, with what is drawn into it.
    pub struct Layer {
        hwnd: HWND,
        dc: HDC,
        bitmap: HBITMAP,
        previous: HGDIOBJ,
        /// The window's pixels: premultiplied BGRA, top row first.
        bits: *mut u8,
        pixmap: Pixmap,
        x: i32,
        y: i32,
        shown: bool,
    }

    impl Layer {
        pub fn new(x: i32, y: i32, w: i32, h: i32) -> Option<Layer> {
            if w <= 0 || h <= 0 {
                return None;
            }
            let pixmap = Pixmap::new(w as u32, h as u32)?;
            register_class();
            let (style_ex, style) = (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, WS_POPUP);
            let name = class_name();
            // Safety: the name outlives the call; the handles are released in `drop`.
            unsafe {
                let hwnd = CreateWindowExW(style_ex, name.as_ptr(), null(), style, x, y, w, h, null_mut(), null_mut(), module(), null());
                if hwnd.is_null() {
                    log::warn!("overlay: cannot create a window");
                    return None;
                }
                let dc = CreateCompatibleDC(null_mut());
                let mut info: BITMAPINFO = std::mem::zeroed();
                info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                info.bmiHeader.biWidth = w;
                info.bmiHeader.biHeight = -h; // Top row first, as tiny-skia has it.
                info.bmiHeader.biPlanes = 1;
                info.bmiHeader.biBitCount = 32;
                info.bmiHeader.biCompression = BI_RGB;
                let mut bits = null_mut();
                let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
                if dc.is_null() || bitmap.is_null() || bits.is_null() {
                    log::warn!("overlay: cannot create the drawing surface");
                    DeleteDC(dc);
                    DestroyWindow(hwnd);
                    return None;
                }
                let previous = SelectObject(dc, bitmap);
                Some(Layer { hwnd, dc, bitmap, previous, bits: bits.cast(), pixmap, x, y, shown: false })
            }
        }

        pub fn position(&self) -> (i32, i32) {
            (self.x, self.y)
        }

        pub fn size(&self) -> (i32, i32) {
            (self.pixmap.width() as i32, self.pixmap.height() as i32)
        }

        /// Where the next `draw` puts the window.
        pub fn move_to(&mut self, x: i32, y: i32) {
            (self.x, self.y) = (x, y);
        }

        /// Draws a frame and puts it on the screen.
        pub fn draw(&mut self, paint: impl FnOnce(&mut Pixmap)) {
            self.pixmap.fill(Color::TRANSPARENT);
            paint(&mut self.pixmap);
            let (w, h) = self.size();
            // Safety: `bits` holds w*h pixels of 4 bytes, as does the pixmap.
            let pixels = unsafe { std::slice::from_raw_parts_mut(self.bits, (w as usize) * (h as usize) * 4) };
            for (out, rgba) in pixels.chunks_exact_mut(4).zip(self.pixmap.data().chunks_exact(4)) {
                out.copy_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
            }
            let position = POINT { x: self.x, y: self.y };
            let size = SIZE { cx: w, cy: h };
            let source = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
            // Safety: every pointer refers to a local that outlives the call.
            unsafe {
                UpdateLayeredWindow(self.hwnd, null_mut(), &position, &size, self.dc, &source, 0, &blend, ULW_ALPHA);
                if !self.shown {
                    ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                    self.shown = true;
                }
            }
        }
    }

    impl Drop for Layer {
        fn drop(&mut self) {
            // Safety: everything here was created by `new` and is no longer used.
            unsafe {
                SelectObject(self.dc, self.previous);
                DeleteObject(self.bitmap);
                DeleteDC(self.dc);
                DestroyWindow(self.hwnd);
            }
        }
    }
}
