//! Switching at the screen edge, as Logi Flow does but without a network: when the
//! cursor rests against an outer edge of the desktop for a moment, the group and the
//! mouse go to the channel set for that edge. Each computer watches only its own
//! screen, so nothing is sent between computers; the one the mouse arrives at puts
//! its cursor at the edge the mouse left by, as if it had crossed over.
//!
//! The shared part (edge geometry, the watcher) is the same in every port; the
//! `platform` module at the end reads the cursor and the displays.

use crate::config::{Edge, Edges, ShakeKey};
use crate::config::EDGES;
use crate::device::waveform;
use crate::overlay;
use crate::shared::Shared;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// How often the cursor is read. Reading it is cheap; 25 ms is finer than anyone moves.
const POLL: Duration = Duration::from_millis(25);
/// How often the engine's status (this computer's channel, is the mouse here) is read.
const STATUS_POLL: Duration = Duration::from_millis(300);
/// How often the display arrangement is re-read.
const DISPLAYS_POLL: Duration = Duration::from_secs(2);
/// After one switch, the next needs the cursor to leave the edge and this much time.
const COOLDOWN: Duration = Duration::from_secs(2);
/// How far in from the edge the cursor is put when the mouse arrives.
const INSET: f64 = 24.0;
/// While the cursor waits at an edge, the mouse pulses this often (if it has a motor).
pub(crate) const HAPTIC_TICK: Duration = Duration::from_millis(150);
/// A pulse takes a moment to play; Change Host right after would cut it short.
const FINAL_PULSE: Duration = Duration::from_millis(150);
/// How often the cursor is read while shaking is being watched for. Real shakes turn
/// every 60-150 ms; at the usual 25 ms a stroke is only two or three samples long.
const SHAKE_POLL: Duration = Duration::from_millis(10);
/// A turn of a shake comes after at least this many points of travel one way. Quick
/// shakes by hand are short strokes (30-100 points), well under the 80 this once asked.
const SHAKE_MIN_STROKE: f64 = 30.0;
/// Coming back this far from the furthest point counts as turning round (smaller
/// moves are the hand's jitter and do not end a stroke).
const SHAKE_SLOP: f64 = 8.0;
/// A shake goes on while each turn follows the one before within this long; a longer
/// pause ends it, and the time starts again with the next movement.
const SHAKE_GAP: Duration = Duration::from_millis(350);
/// However short the time asked for, a shake has at least this many turns.
const SHAKE_MIN_TURNS: usize = 3;
/// How much shorter than asked a shake may be and still count (see `Axis::update`).
const SHAKE_SLACK: Duration = Duration::from_millis(150);
/// After a shake is noticed, the next one counts only after this long.
const SHAKE_REST: Duration = Duration::from_secs(2);

/// A display, in the global desktop coordinates the platform uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// The outer edge the cursor touches, if any: an edge of the display it is on beyond
/// which no other display continues (between two displays the cursor just crosses
/// over). The last `margin` pixels at each end do not count: hot corners, the window
/// buttons and the menu bar's corners live there.
pub fn edge_at(displays: &[Rect], x: f64, y: f64, margin: f64) -> Option<Edge> {
    let d = displays.iter().find(|d| d.contains(x, y))?;
    let beyond = |px: f64, py: f64| displays.iter().any(|o| o.contains(px, py));
    let along_y = y >= d.y + margin && y < d.y + d.h - margin;
    let along_x = x >= d.x + margin && x < d.x + d.w - margin;
    if along_y && x <= d.x && !beyond(d.x - 1.0, y) {
        return Some(Edge::Left);
    }
    if along_y && x >= d.x + d.w - 1.0 && !beyond(d.x + d.w, y) {
        return Some(Edge::Right);
    }
    if along_x && y <= d.y && !beyond(x, d.y - 1.0) {
        return Some(Edge::Top);
    }
    if along_x && y >= d.y + d.h - 1.0 && !beyond(x, d.y + d.h) {
        return Some(Edge::Bottom);
    }
    None
}

/// Where the cursor goes when the mouse arrives by `edge`: the middle of that edge
/// of the display that reaches furthest that way, a little way in.
pub fn landing(displays: &[Rect], edge: Edge) -> Option<(f64, f64)> {
    let d = match edge {
        Edge::Left => displays.iter().min_by(|a, b| a.x.total_cmp(&b.x)),
        Edge::Right => displays.iter().max_by(|a, b| (a.x + a.w).total_cmp(&(b.x + b.w))),
        Edge::Top => displays.iter().min_by(|a, b| a.y.total_cmp(&b.y)),
        Edge::Bottom => displays.iter().max_by(|a, b| (a.y + a.h).total_cmp(&(b.y + b.h))),
    }?;
    Some(match edge {
        Edge::Left => (d.x + INSET, d.y + d.h / 2.0),
        Edge::Right => (d.x + d.w - INSET, d.y + d.h / 2.0),
        Edge::Top => (d.x + d.w / 2.0, d.y + INSET),
        Edge::Bottom => (d.x + d.w / 2.0, d.y + d.h - INSET),
    })
}

/// The channel a "last switch" status text ends with ("1 → 2", "→ 2").
fn last_channel(text: &str) -> Option<u8> {
    text.rsplit(' ').next()?.parse().ok()
}

/// How the pulses go at `progress` (0..1) of the wait at an edge: the gap to the next
/// one and its waveform, quicker and sharper as the switch nears.
pub(crate) fn rising_pulse(progress: f64) -> (Duration, u8) {
    if progress < 0.40 {
        (Duration::from_millis(180), waveform::SUBTLE_COLLISION)
    } else if progress < 0.75 {
        (Duration::from_millis(110), waveform::DAMP_COLLISION)
    } else {
        (Duration::from_millis(60), waveform::SHARP_COLLISION)
    }
}

/// One axis of a shake: strokes one way and the other, told apart from jitter.
#[derive(Default)]
struct Axis {
    last: Option<f64>,
    /// Which way the stroke goes (+1, -1; 0 before the first movement), its furthest
    /// point so far, where it started and when.
    heading: f64,
    extreme: f64,
    start: f64,
    stroke_began: Option<Instant>,
    /// When the first stroke of the shake going on began, and its turns, oldest first.
    began: Option<Instant>,
    turns: Vec<Instant>,
    /// The last time the cursor really moved (not just jitter).
    last_move: Option<Instant>,
}

impl Axis {
    /// Forgets the shake going on; the next stroke starts from `at`.
    fn reset(&mut self, at: f64) {
        *self = Axis { last: Some(at), start: at, extreme: at, ..Axis::default() };
    }

    /// Takes the next position; true when a shake that has lasted `time` is going on.
    fn update(&mut self, v: f64, time: Duration, now: Instant) -> bool {
        let Some(prev) = self.last.replace(v) else {
            self.start = v;
            return false;
        };
        if (v - prev).abs() >= 2.0 {
            self.last_move = Some(now);
        }
        let stale = |t: Instant| now.duration_since(t) > SHAKE_GAP;
        let mut turned = false;
        if self.last_move.is_some_and(stale) || self.turns.last().is_some_and(|t| stale(*t)) {
            self.reset(v);
        }
        if self.heading == 0.0 {
            if (v - self.start).abs() >= SHAKE_SLOP {
                self.heading = (v - self.start).signum();
                self.extreme = v;
                self.stroke_began = Some(now);
            } else {
                // Still at rest: the stroke starts from the latest still point.
                self.start = v;
            }
        } else if (v - self.extreme) * self.heading >= 0.0 {
            self.extreme = v;
        } else if (self.extreme - v) * self.heading >= SHAKE_SLOP {
            // A stroke of a shake is long enough and quick enough; a slow sweep or a
            // short wobble ends the shake, and the time starts again from here.
            let quick = self.stroke_began.is_some_and(|t| !stale(t));
            if quick && (self.extreme - self.start).abs() >= SHAKE_MIN_STROKE {
                if self.turns.is_empty() {
                    self.began = self.stroke_began;
                }
                self.turns.push(now);
                turned = true;
            } else {
                self.turns.clear();
                self.began = None;
            }
            self.start = self.extreme;
            self.heading = -self.heading;
            self.extreme = v;
            self.stroke_began = Some(now);
        }
        // "A second of shaking" is judged a little short: the first stroke is already
        // under way before it is seen, and people stop as soon as the cursor reacts.
        let needed = time.saturating_sub(SHAKE_SLACK);
        // Judged at each new turn: a shake that has stopped is over.
        turned && self.turns.len() >= SHAKE_MIN_TURNS && self.began.is_some_and(|b| now.duration_since(b) >= needed)
    }
}

/// Notices the cursor being shaken for a while (in any direction: each axis is
/// watched): the turns must keep coming, each within `SHAKE_GAP` of the last, for the
/// whole time asked for, counted from where the movement began, and with a key asked
/// for, only while that key is held.
#[derive(Default)]
struct Shake {
    axes: [Axis; 2],
    noticed: Option<Instant>,
}

impl Shake {
    /// Takes the cursor's position and whether the key asked for is held (true when
    /// none is); true at the moment a shake that has lasted `time` is noticed.
    fn update(&mut self, (x, y): (f64, f64), held: bool, time: Duration, now: Instant) -> bool {
        if !held {
            // Let go of the key: the shake is over; the next starts from here.
            self.axes[0].reset(x);
            self.axes[1].reset(y);
            return false;
        }
        let a = self.axes[0].update(x, time, now);
        let b = self.axes[1].update(y, time, now);
        if !(a || b) {
            return false;
        }
        self.axes[0].reset(x);
        self.axes[1].reset(y);
        if self.noticed.is_some_and(|t| now.duration_since(t) < SHAKE_REST) {
            return false;
        }
        self.noticed = Some(now);
        true
    }
}

/// Plays the countdown once, at an edge of the display the cursor is on, for the
/// settings window. At least a second, so a short wait can still be judged.
pub fn preview(edges: &Edges, delay_ms: u64) {
    let displays = platform::displays();
    let cursor = platform::cursor();
    let on_cursor = cursor.and_then(|(x, y)| displays.iter().find(|d| d.contains(x, y)));
    let Some(display) = on_cursor.or(displays.first()) else { return };
    let edge = EDGES.into_iter().find(|&e| edges.channel(e) != 0).unwrap_or(Edge::Right);
    let duration = Duration::from_millis(delay_ms.max(1000));
    overlay::edge_begin(edges, edge, display, Duration::ZERO, duration, true);
}

/// Takes the countdown away, if one is showing.
fn end_countdown(showing: &mut bool) {
    if std::mem::take(showing) {
        overlay::edge_end();
    }
}

/// Whether this computer can watch its screen edges at all.
pub fn available() -> bool {
    platform::available()
}

/// Starts the watcher on its own thread; it runs until the shared stop flag is set.
pub fn spawn(shared: Arc<Shared>) {
    std::thread::Builder::new()
        .name("edges".into())
        .spawn(move || watch(shared))
        .expect("spawning the edge watcher");
}

/// The app handles delayed network crossings; netcore handles immediate crossings.
fn edge_activity(cfg: &crate::config::Config) -> (bool, bool) {
    (cfg.logitech_edges_allowed(), cfg.network_edges_allowed() && cfg.network.notice_accepted && cfg.network.cross_delay_ms > 0)
}

fn watch(shared: Arc<Shared>) {
    if !available() {
        log::info!("edge switching: not available on this computer");
        return;
    }
    let mut cfg = shared.config();
    let mut version = shared.version();
    let mut announced = false;
    let mut status_read: Option<Instant> = None;
    // From the engine's status: this computer's channel, whether the master mouse is
    // here, and the "last switch" text as last seen.
    let (mut here, mut mouse_here, mut last_switch_text): (Option<u8>, bool, Option<String>) = (None, false, None);
    // The mouse was sent away from here by this channel; when it comes back the cursor
    // goes to that channel's edge. Used once.
    let mut landing_channel: Option<u8> = None;
    let mut displays: Vec<Rect> = Vec::new();
    let mut displays_read: Option<Instant> = None;
    // The edge the cursor has rested on, and since when.
    let mut resting: Option<(Edge, Instant)> = None;
    // The cursor has left the edges since the last decision, so a new rest counts.
    let mut armed = true;
    let mut last_switch: Option<Instant> = None;
    let mut last_tick: Option<Instant> = None;
    // The countdown is on the screen.
    let mut counting_down = false;
    let mut shake = Shake::default();

    while !shared.stopped() {
        std::thread::sleep(if cfg.locate.on && cfg.locate.on_shake { SHAKE_POLL } else { POLL });
        if shared.version() != version {
            version = shared.version();
            cfg = shared.config();
            announced = false;
        }
        let (logitech_active, network_active) = edge_activity(&cfg);
        let active = logitech_active || network_active;
        if !announced {
            announced = true;
            if logitech_active {
                let list: Vec<String> = crate::config::EDGES
                    .iter()
                    .filter(|&&e| cfg.edges.channel(e) != 0)
                    .map(|&e| format!("{} → {}", e.name(), cfg.edges.channel(e)))
                    .collect();
                log::info!("edges: {} after {} ms", list.join(", "), cfg.edges.dwell_ms);
            } else if cfg.edges.on {
                log::info!("edges: on, but no edge has a channel");
            }
        }
        if status_read.is_none_or(|t| t.elapsed() >= STATUS_POLL) {
            let first = status_read.is_none();
            status_read = Some(Instant::now());
            let st = shared.status();
            here = st.channel;
            if st.last_switch != last_switch_text {
                last_switch_text = st.last_switch.clone();
                landing_channel = st.last_switch.as_deref().and_then(last_channel);
            }
            let now_here = st.trigger.is_some();
            let arrived = now_here && !mouse_here && !first;
            if arrived && logitech_active && cfg.edges.place_cursor {
                // The mouse has come back from the channel it was sent to: put the
                // cursor at that channel's edge, as if it had come in from there.
                if let Some(edge) = landing_channel.take().and_then(|c| cfg.edges.edge_for(c)) {
                    if displays.is_empty() {
                        displays = platform::displays();
                    }
                    if let Some((x, y)) = landing(&displays, edge) {
                        log::info!("mouse is back: cursor to the {} edge", edge.name());
                        platform::place(x, y);
                        resting = None;
                        armed = false;
                    }
                }
            }
            if arrived && cfg.locate.on && cfg.locate.on_arrival {
                overlay::locate(&cfg.locate);
            }
            mouse_here = now_here;
        }
        if shared.take_locate_request() && cfg.locate.on {
            overlay::locate(&cfg.locate);
        }
        if cfg.locate.on && cfg.locate.on_shake {
            let held = match cfg.locate.shake_key {
                ShakeKey::None => true,
                key => platform::key_down(key),
            };
            let time = Duration::from_millis(cfg.locate.shake_time_ms);
            if platform::cursor().is_some_and(|at| shake.update(at, held, time, Instant::now())) {
                overlay::locate(&cfg.locate);
            }
        }
        if !active {
            resting = None;
            end_countdown(&mut counting_down);
            continue;
        }
        if displays_read.is_none_or(|t| t.elapsed() >= DISPLAYS_POLL) {
            displays_read = Some(Instant::now());
            displays = platform::displays();
        }
        let Some((x, y)) = platform::cursor() else { continue };
        let margin = if logitech_active { f64::from(cfg.edges.corner_margin) } else { cfg.network.corner_margin };
        let Some(edge) = edge_at(&displays, x, y, margin) else {
            resting = None;
            armed = true;
            last_tick = None;
            end_countdown(&mut counting_down);
            continue;
        };
        // While network sharing is on the edges belong to it: only a side with a
        // connected computer waits and crosses, the others do nothing.
        let network = cfg.network_edges_allowed();
        if network && !(network_active && crate::net::network_side_ready(edge)) {
            resting = None;
            last_tick = None;
            end_countdown(&mut counting_down);
            continue;
        }
        // With "hold Control" the wait starts only with the key down; letting go calls
        // it off, and pressing again at the same edge starts over.
        let require_control = if network { cfg.network.require_control } else { cfg.edges.require_control };
        if require_control && !platform::control_down() {
            resting = None;
            last_tick = None;
            end_countdown(&mut counting_down);
            continue;
        }
        let since = match resting {
            Some((e, t)) if e == edge => t,
            _ => {
                resting = Some((edge, Instant::now()));
                last_tick = None;
                end_countdown(&mut counting_down);
                continue;
            }
        };
        let channel = cfg.edges.channel(edge);
        let dwell = Duration::from_millis(if network { cfg.network.cross_delay_ms } else { cfg.edges.dwell_ms });
        let cooling = last_switch.is_some_and(|t| t.elapsed() < COOLDOWN);
        let will_switch = if network {
            armed && !cooling
        } else {
            armed && channel != 0 && mouse_here && here != Some(channel) && !cooling
        };
        if will_switch {
            if !counting_down {
                if let Some(display) = displays.iter().find(|d| d.contains(x, y)) {
                    overlay::edge_begin(&cfg.edges, edge, display, since.elapsed(), dwell, false);
                    counting_down = true;
                }
            }
            // Waiting at an edge that will switch: the mouse pulses so the hand knows.
            if cfg.haptics.on_edge {
                let progress = since.elapsed().as_secs_f64() / dwell.as_secs_f64();
                let (gap, wave) =
                    if cfg.haptics.progressive { rising_pulse(progress) } else { (HAPTIC_TICK, cfg.haptics.edge_waveform) };
                if last_tick.is_none_or(|t| t.elapsed() >= gap) {
                    last_tick = Some(Instant::now());
                    shared.request_haptic(wave);
                }
            }
        } else {
            end_countdown(&mut counting_down);
        }
        if !armed || since.elapsed() < dwell {
            continue;
        }
        // One decision per visit to the edge.
        armed = false;
        end_countdown(&mut counting_down);
        if network {
            if cooling {
                continue;
            }
            log::info!("{} edge: crossing to the network computer", edge.name());
            last_switch = Some(Instant::now());
            // No channel switch follows, so nothing else plays the last pulse.
            if cfg.haptics.on_edge && cfg.haptics.progressive {
                shared.request_haptic(waveform::COMPLETED);
                std::thread::sleep(FINAL_PULSE);
            }
            crate::net::cross(edge, x, y);
            continue;
        }
        if channel == 0 {
            continue;
        }
        if !mouse_here {
            log::debug!("{} edge: the master mouse is not here", edge.name());
            continue;
        }
        if here == Some(channel) {
            log::debug!("{} edge: channel {channel} is this computer", edge.name());
            continue;
        }
        if cooling {
            continue;
        }
        log::info!("{} edge: channel {channel}", edge.name());
        last_switch = Some(Instant::now());
        // The rising pulses end in one last pulse. With pulses on switching, the
        // engine's own switch pulse is that one.
        if cfg.haptics.on_edge && cfg.haptics.progressive && !cfg.haptics.on_switch {
            shared.request_haptic(waveform::COMPLETED);
            std::thread::sleep(FINAL_PULSE);
        }
        shared.request_switch(channel);
    }
    end_countdown(&mut counting_down);
}

/// The cursor and the displays, in Windows' virtual-screen coordinates (origin at the
/// top left of the primary display, y down). None of this needs a permission.
pub(crate) mod platform {
    use super::Rect;
    use crate::config::ShakeKey;
    use windows_sys::Win32::Foundation::{LPARAM, POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

    pub fn available() -> bool {
        true
    }

    /// Where the cursor is right now.
    pub fn cursor() -> Option<(f64, f64)> {
        let mut p = POINT { x: 0, y: 0 };
        // Safety: a plain out-parameter.
        (unsafe { GetCursorPos(&mut p) } != 0).then(|| (f64::from(p.x), f64::from(p.y)))
    }

    unsafe extern "system" fn visit(_: HMONITOR, _: HDC, rect: *mut RECT, data: LPARAM) -> i32 {
        // Safety: `data` is the Vec passed by `displays`; `rect` is valid for this call.
        let out = unsafe { &mut *(data as *mut Vec<Rect>) };
        let r = unsafe { *rect };
        out.push(Rect {
            x: f64::from(r.left),
            y: f64::from(r.top),
            w: f64::from(r.right - r.left),
            h: f64::from(r.bottom - r.top),
        });
        1
    }

    pub fn displays() -> Vec<Rect> {
        let mut out: Vec<Rect> = Vec::new();
        unsafe { EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), Some(visit), &mut out as *mut Vec<Rect> as LPARAM) };
        out
    }

    /// Whether a key is held right now (either Control, Alt or Shift key).
    fn down(vk: u16) -> bool {
        // Safety: reads the key state; takes and keeps nothing. The top bit means "down".
        (unsafe { GetAsyncKeyState(i32::from(vk)) } as u16) & 0x8000 != 0
    }

    /// Whether a Control key is held right now.
    pub fn control_down() -> bool {
        down(VK_CONTROL)
    }

    /// Whether the key a shake must be made with is held (true when none is asked for).
    pub fn key_down(key: ShakeKey) -> bool {
        match key {
            ShakeKey::None => true,
            ShakeKey::Ctrl => down(VK_CONTROL),
            ShakeKey::Alt => down(VK_MENU),
            ShakeKey::Shift => down(VK_SHIFT),
        }
    }

    /// Moves the cursor there.
    pub fn place(x: f64, y: f64) {
        unsafe { SetCursorPos(x as i32, y as i32) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_side_by_side() -> Vec<Rect> {
        // Main display 1920×1080 with a second one to its right, a little higher.
        vec![Rect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 }, Rect { x: 1920.0, y: -200.0, w: 2560.0, h: 1440.0 }]
    }

    #[test]
    fn outer_edges_only() {
        let d = two_side_by_side();
        assert_eq!(edge_at(&d, 0.0, 500.0, 80.0), Some(Edge::Left));
        assert_eq!(edge_at(&d, 1919.0, 500.0, 80.0), None, "the second display continues there");
        assert_eq!(edge_at(&d, 1919.0, 1000.0, 80.0), None, "the join runs the whole height of the first display");
        assert_eq!(edge_at(&d, 4479.0, 500.0, 80.0), Some(Edge::Right));
        assert_eq!(edge_at(&d, 960.0, 0.0, 80.0), Some(Edge::Top));
        assert_eq!(edge_at(&d, 960.0, 1079.0, 80.0), Some(Edge::Bottom));
        assert_eq!(edge_at(&d, 3000.0, -200.0, 80.0), Some(Edge::Top));
        assert_eq!(edge_at(&d, 500.0, 500.0, 80.0), None, "the middle of the screen");
    }

    #[test]
    fn corners_do_not_count() {
        let d = two_side_by_side();
        assert_eq!(edge_at(&d, 0.0, 10.0, 80.0), None);
        assert_eq!(edge_at(&d, 0.0, 1075.0, 80.0), None);
        assert_eq!(edge_at(&d, 10.0, 0.0, 80.0), None);
        assert_eq!(edge_at(&d, 0.0, 80.0, 80.0), Some(Edge::Left));
    }

    #[test]
    fn cursor_outside_every_display() {
        assert_eq!(edge_at(&two_side_by_side(), -5.0, 500.0, 80.0), None);
        assert_eq!(edge_at(&[], 0.0, 0.0, 80.0), None);
    }

    #[test]
    fn landing_spots() {
        let d = two_side_by_side();
        assert_eq!(landing(&d, Edge::Left), Some((24.0, 540.0)));
        assert_eq!(landing(&d, Edge::Right), Some((4480.0 - 24.0, 520.0)));
        assert_eq!(landing(&d, Edge::Top), Some((3200.0, -176.0)));
        // The second display reaches lower (-200 + 1440 = 1240), so the bottom edge is its.
        assert_eq!(landing(&d, Edge::Bottom), Some((3200.0, 1216.0)));
        assert_eq!(landing(&[], Edge::Left), None);
    }

    /// Reads this Mac's cursor and displays: `cargo test platform -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn platform_reads() {
        let displays = platform::displays();
        let cursor = platform::cursor();
        println!("displays: {displays:?}\ncursor: {cursor:?}");
        assert!(!displays.is_empty());
        assert!(cursor.is_some());
        let (x, y) = cursor.unwrap();
        println!("edge now: {:?}", edge_at(&displays, x, y, 80.0));
    }

    #[test]
    fn pulses_quicken_and_sharpen_towards_the_switch() {
        assert_eq!(rising_pulse(0.0), (Duration::from_millis(180), waveform::SUBTLE_COLLISION));
        assert_eq!(rising_pulse(0.39), (Duration::from_millis(180), waveform::SUBTLE_COLLISION));
        assert_eq!(rising_pulse(0.40), (Duration::from_millis(110), waveform::DAMP_COLLISION));
        assert_eq!(rising_pulse(0.74), (Duration::from_millis(110), waveform::DAMP_COLLISION));
        assert_eq!(rising_pulse(0.75), (Duration::from_millis(60), waveform::SHARP_COLLISION));
        assert_eq!(rising_pulse(1.2), (Duration::from_millis(60), waveform::SHARP_COLLISION));
    }

    /// Feeds x positions, one poll apart, with the key held where `held` says; returns
    /// at which positions a shake lasting `time` was noticed.
    fn shake_with(xs: &[f64], held: impl Fn(usize) -> bool, time: Duration) -> Vec<usize> {
        let mut s = Shake::default();
        let start = Instant::now();
        xs.iter()
            .enumerate()
            .filter(|(i, x)| s.update((**x, 300.0), held(*i), time, start + POLL * *i as u32))
            .map(|(i, _)| i)
            .collect()
    }

    fn shake_on(xs: &[f64]) -> Vec<usize> {
        shake_with(xs, |_| true, Duration::from_millis(1000))
    }

    /// Back and forth by 300 points, three polls (75 ms) each way, for `polls` polls.
    fn shaking(polls: usize) -> Vec<f64> {
        (0..polls).map(|i| [0.0, 100.0, 200.0, 300.0, 200.0, 100.0][i % 6]).collect()
    }

    #[test]
    fn a_shake_counts_once_it_has_lasted_the_time() {
        // Four seconds of shaking: noticed after about a second, then rests two seconds.
        let noticed = shake_on(&shaking(160));
        assert!(!noticed.is_empty(), "{noticed:?}");
        let first = POLL * noticed[0] as u32;
        assert!(first >= Duration::from_millis(800) && first < Duration::from_millis(1300), "{first:?}");
        assert_eq!(noticed.len(), 2, "{noticed:?}");
        assert!(POLL * (noticed[1] - noticed[0]) as u32 >= SHAKE_REST);
        // The time asked for decides when.
        let at = |ms: u64| shake_with(&shaking(160), |_| true, Duration::from_millis(ms))[0] as u32 * POLL;
        assert!(at(500) < at(1000) && at(1000) < at(1500) && at(1500) < at(2000));
        assert!(at(2000) >= Duration::from_millis(1800));
    }

    #[test]
    fn a_short_shake_is_not_enough() {
        // Half a second of shaking, then still: never a full second of it.
        let mut xs = shaking(20);
        xs.extend(std::iter::repeat_n(100.0, 100));
        assert!(shake_on(&xs).is_empty());
        // Two short shakes with a pause between do not add up.
        let mut xs = shaking(30);
        xs.extend(std::iter::repeat_n(100.0, 20));
        xs.extend(shaking(30));
        assert!(shake_on(&xs).is_empty(), "{:?}", shake_on(&xs));
    }

    #[test]
    fn ordinary_movement_is_not_a_shake() {
        let straight: Vec<f64> = (0..60).map(|i| i as f64 * 40.0).collect();
        assert!(shake_on(&straight).is_empty());
        // Small wobbles never travel far enough to count.
        let wobble: Vec<f64> = (0..200).map(|i| if i % 2 == 0 { 0.0 } else { 20.0 }).collect();
        assert!(shake_on(&wobble).is_empty());
        // Far enough, but so slowly the turns are too far apart.
        let slow: Vec<f64> = (0..400).map(|i| [0.0, 5.0, 10.0, 15.0][i % 4] + ((i / 40) % 2) as f64 * 200.0).collect();
        assert!(shake_on(&slow).is_empty());
        // Moving back and forth across the screen at an easy pace (a turn every 600 ms).
        let sweeping: Vec<f64> = (0..400).map(|i| ((i % 48) as f64 - 24.0).abs() * 40.0).collect();
        assert!(shake_on(&sweeping).is_empty());
    }

    #[test]
    fn with_a_key_only_while_it_is_held() {
        let xs = shaking(160);
        let second = Duration::from_millis(1000);
        // Never held: nothing.
        assert!(shake_with(&xs, |_| false, second).is_empty());
        // Held from 1.5 s on: noticed about a second after that.
        let noticed = shake_with(&xs, |i| i >= 60, second);
        assert!(!noticed.is_empty());
        let at = POLL * noticed[0] as u32;
        assert!(at >= Duration::from_millis(2300) && at < Duration::from_millis(2800), "{at:?}");
        // Let go for a moment early on: the time starts again.
        let noticed = shake_with(&xs, |i| !(30..32).contains(&i), second);
        let at = POLL * noticed[0] as u32;
        assert!(at >= Duration::from_millis(1600), "{at:?}");
    }

    /// A hand-made recording of a real shake: `hz` turns-and-back per second of `amp`
    /// points each way along `dir`, the amplitude building up over the first 150 ms,
    /// samples at uneven intervals of `step_ms` (the poll plus the scheduler's slop) and
    /// a couple of points of sensor noise. Returns (time since the start, position).
    fn recorded_shake(hz: f64, amp: f64, secs: f64, step_ms: (u64, u64), dir: (f64, f64), seed: u64) -> Vec<(Duration, (f64, f64))> {
        let mut state = seed;
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as f64 / (1u64 << 31) as f64
        };
        let (mut t, mut out) = (0.0_f64, Vec::new());
        while t <= secs {
            let ramp = (t / 0.15).min(1.0);
            let wave = (t * hz * std::f64::consts::TAU).sin() * amp * ramp;
            let (nx, ny) = ((next() - 0.5) * 4.0, (next() - 0.5) * 4.0);
            out.push((Duration::from_secs_f64(t), (800.0 + dir.0 * wave + nx, 500.0 + dir.1 * wave + ny)));
            t += (step_ms.0 as f64 + next() * (step_ms.1 - step_ms.0) as f64) / 1000.0;
        }
        out
    }

    /// When (since the start) a recording was noticed as a shake lasting `time`.
    fn noticed_at(recording: &[(Duration, (f64, f64))], time: Duration) -> Vec<Duration> {
        let (mut s, start) = (Shake::default(), Instant::now());
        recording.iter().filter(|(t, at)| s.update(*at, true, time, start + *t)).map(|(t, _)| *t).collect()
    }

    #[test]
    fn a_fast_second_of_shaking_is_noticed_as_recorded() {
        let second = Duration::from_millis(1000);
        // Quick, short shakes by hand: 6-9 turns a second over 30-120 points, along x,
        // along y and diagonally, read every 10-12 ms (while watching) or 25-35 ms.
        for (hz, amp) in [(6.0, 60.0), (7.5, 40.0), (9.0, 30.0), (5.0, 110.0)] {
            for dir in [(1.0, 0.0), (0.0, 1.0), (0.8, 0.6)] {
                for step in [(10, 12), (20, 30), (25, 35)] {
                    for seed in 1..=4 {
                        // A second of shaking, 150 ms of it getting up to speed.
                        let rec = recorded_shake(hz, amp, 1.15, step, dir, seed);
                        let got = noticed_at(&rec, second);
                        assert!(!got.is_empty(), "{hz} Hz +-{amp} dir {dir:?} step {step:?} seed {seed}: not noticed in 1 s");
                        assert!(got[0] >= Duration::from_millis(800) && got[0] <= Duration::from_millis(1150), "{got:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn recorded_non_shakes_stay_quiet() {
        let second = Duration::from_millis(1000);
        // A slow, wide sweep (about 2 turns a second) and plain jitter of a resting hand.
        for (hz, amp) in [(1.0, 400.0), (1.2, 250.0), (7.0, 6.0), (10.0, 12.0)] {
            let rec = recorded_shake(hz, amp, 6.0, (20, 30), (1.0, 0.0), 9);
            assert!(noticed_at(&rec, second).is_empty(), "{hz} Hz +-{amp}");
        }
        // Half a second of shaking is not a second.
        let rec = recorded_shake(7.5, 40.0, 0.5, (10, 12), (1.0, 0.0), 3);
        assert!(noticed_at(&rec, second).is_empty());
    }

    #[test]
    fn last_switch_text() {
        assert_eq!(last_channel("1 → 2"), Some(2));
        assert_eq!(last_channel("→ 3"), Some(3));
        assert_eq!(last_channel(""), None);
    }
}

#[cfg(test)]
mod owner_tests {
    use super::*;
    #[test]
    fn watcher_has_at_most_one_owner() {
        for enabled in [false, true] {
            for network in [false, true] {
                for edges in [false, true] {
                    let mut c = crate::config::Config::default();
                    c.enabled = enabled; c.network.on = network; c.edges.on = edges;
                    c.edges.right = 2; c.network.notice_accepted = true; c.network.cross_delay_ms = 500;
                    assert_eq!(edge_activity(&c), (enabled && edges, network && !edges));
                    c.edges.right = 0;
                    assert!(!edge_activity(&c).0);
                    c.network.cross_delay_ms = 0;
                    assert!(!edge_activity(&c).1);
                }
            }
        }
    }
}
