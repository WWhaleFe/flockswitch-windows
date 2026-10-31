//! The settings window. It runs as its own process (`flockswitch settings`) so the tray
//! app keeps its own event loop; changes are written to config.toml, which the running
//! engine reloads within a second. Every change applies immediately and the window
//! stays open, unlike a native menu that closes after each click.

use crate::config::{Config, Gesture, BUTTON_GESTURE, BUTTON_MIDDLE, BUTTON_WHEEL_MODE};
use crate::i18n::{texts, Texts};
use crate::shared::{self, DeviceInfo, StatusFile};
use anyhow::{anyhow, Result};
use auto_launch::AutoLaunch;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Margin, Pos2, Rect, Response, RichText, Role,
    Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const REFRESH: Duration = Duration::from_secs(1);

/// The app icon, scaled by build.rs (WINDOW_ICON there).
const ICON_SIDE: u32 = 96;
const ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/window-icon.rgba"));

/// Opens the window. `lock` keeps a second window from opening; a connection to it
/// means "Settings…" was chosen again, so the window comes back to the front.
pub fn run(lock: std::net::TcpListener) -> Result<()> {
    if let Ok(cfg) = Config::load_or_create() {
        crate::i18n::set_language(&cfg.language);
    }
    let t = texts();
    let icon = egui::IconData { rgba: ICON_RGBA.to_vec(), width: ICON_SIDE, height: ICON_SIDE };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(t.settings_title)
            .with_icon(icon)
            .with_inner_size([520.0, 720.0])
            .with_min_inner_size([400.0, 360.0])
            .with_active(true),
        renderer: renderer(),
        ..Default::default()
    };
    log::info!("settings window: {:?} renderer", options.renderer);
    eframe::run_native(
        "flockswitch-settings",
        options,
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            install_style(&cc.egui_ctx);
            let raise = Arc::new(AtomicBool::new(false));
            let (ctx, flag) = (cc.egui_ctx.clone(), raise.clone());
            std::thread::spawn(move || {
                for _ in lock.incoming() {
                    flag.store(true, Ordering::Relaxed);
                    ctx.request_repaint();
                }
            });
            Ok(Box::new(SettingsApp::new(&cc.egui_ctx, raise)))
        }),
    )
    .map_err(|e| anyhow!("cannot open the settings window: {e}"))
}

/// Environment variable that makes a relaunched settings window use OpenGL.
pub const RENDERER_ENV: &str = "FLOCKSWITCH_RENDERER";

/// Direct3D through wgpu, which also works where only OpenGL 1.1 exists; OpenGL when
/// relaunched as a fallback.
fn renderer() -> eframe::Renderer {
    if cfg!(windows) && std::env::var(RENDERER_ENV).as_deref() != Ok("glow") {
        #[cfg(windows)]
        return eframe::Renderer::Wgpu;
    }
    eframe::Renderer::Glow
}

/// A system font file, memory-mapped rather than read: mapped pages are read only as
/// glyphs are drawn and do not count against the app's memory. Falls back to reading.
fn font_file(path: &str) -> Option<egui::epaint::text::Blob> {
    let file = std::fs::File::open(path).ok()?;
    // Safety: system font files are not modified while we run.
    match unsafe { memmap2::Mmap::map(&file) } {
        Ok(map) => Some(std::sync::Arc::new(map)),
        Err(_) => std::fs::read(path).ok().map(|b| std::sync::Arc::new(b) as egui::epaint::text::Blob),
    }
}

/// Uses the system UI font for everything (egui's own font has no Hangul, and mixing
/// it with a Korean fallback looks uneven), plus its bold face for titles.
fn install_fonts(ctx: &egui::Context) {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    // (file, regular face index, bold file, bold face index)
    // Malgun Gothic, the Windows Korean UI font.
    let candidates = [(format!("{windir}\\Fonts\\malgun.ttf"), 0, Some(format!("{windir}\\Fonts\\malgunbd.ttf")), 0)];
    let mut fonts = egui::FontDefinitions::default();
    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let found = candidates.iter().find_map(|(path, index, bold_path, bold_index)| {
        let file = font_file(path)?;
        let bold = match bold_path {
            Some(b) => font_file(b).map(|f| (f, *bold_index)),
            // Same collection file: the bold face, if the file has one (shared, not copied).
            None if *bold_index > 0 => Some((file.clone(), *bold_index)),
            None => None,
        };
        Some(((file, *index), bold))
    });

    let mut bold_family = Vec::new();
    if let Some(((file, index), bold)) = found {
        fonts.font_data.insert("system".into(), std::sync::Arc::new(egui::FontData::from_blob(file, index)));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().insert(0, "system".into());
        }
        bold_family.push(if let Some((file, index)) = bold {
            fonts.font_data.insert("system-bold".into(), std::sync::Arc::new(egui::FontData::from_blob(file, index)));
            "system-bold".to_string()
        } else {
            "system".to_string()
        });
    }
    bold_family.extend(defaults);
    fonts.families.insert(FontFamily::Name("bold".into()), bold_family);
    ctx.set_fonts(fonts);
}

fn install_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.scroll.bar_width = 6.0;
        for (text_style, size) in [
            (egui::TextStyle::Body, 14.0),
            (egui::TextStyle::Button, 14.0),
            (egui::TextStyle::Small, 12.0),
            (egui::TextStyle::Heading, 20.0),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style) {
                font.size = size;
            }
        }
    });
    for (theme, dark) in [(egui::Theme::Light, false), (egui::Theme::Dark, true)] {
        let p = Palette::new(dark);
        let mut visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        visuals.panel_fill = p.bg;
        visuals.window_fill = p.bg;
        visuals.override_text_color = Some(p.text);
        visuals.selection.bg_fill = p.accent.gamma_multiply(0.35);
        ctx.set_visuals_of(theme, visuals);
    }
}

/// Colours for the current light/dark appearance. The accent matches the app icon.
#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    card: Color32,
    border: Color32,
    track: Color32,
    hover: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    green: Color32,
    amber: Color32,
    red: Color32,
}

impl Palette {
    fn new(dark: bool) -> Palette {
        if dark {
            Palette {
                bg: Color32::from_rgb(0x1C, 0x1C, 0x1F),
                card: Color32::from_rgb(0x2A, 0x2A, 0x2E),
                border: Color32::from_rgb(0x3A, 0x3A, 0x40),
                track: Color32::from_rgb(0x48, 0x48, 0x50),
                hover: Color32::from_rgb(0x33, 0x33, 0x39),
                text: Color32::from_rgb(0xF2, 0xF2, 0xF5),
                muted: Color32::from_rgb(0xA8, 0xA8, 0xB0),
                accent: Color32::from_rgb(0x6E, 0x7B, 0xFF),
                green: Color32::from_rgb(0x30, 0xD1, 0x58),
                amber: Color32::from_rgb(0xFF, 0xB3, 0x40),
                red: Color32::from_rgb(0xFF, 0x5A, 0x50),
            }
        } else {
            Palette {
                bg: Color32::from_rgb(0xF2, 0xF2, 0xF6),
                card: Color32::WHITE,
                border: Color32::from_rgb(0xE2, 0xE2, 0xE8),
                track: Color32::from_rgb(0xD5, 0xD5, 0xDC),
                hover: Color32::from_rgb(0xF4, 0xF4, 0xF8),
                text: Color32::from_rgb(0x1C, 0x1C, 0x1E),
                muted: Color32::from_rgb(0x60, 0x60, 0x68),
                accent: Color32::from_rgb(0x46, 0x55, 0xE8),
                green: Color32::from_rgb(0x1F, 0xA8, 0x45),
                amber: Color32::from_rgb(0xC2, 0x6A, 0x00),
                red: Color32::from_rgb(0xD7, 0x2C, 0x24),
            }
        }
    }
}

fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

/// The pages of the window, one tab each.
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Switching,
    Devices,
    General,
}

/// One channel list: a button's press, or one gesture direction.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Slot {
    Press(u16),
    Move(u16, Gesture),
}

fn slot_channels(c: &Config, slot: Slot) -> Vec<u8> {
    match slot {
        Slot::Press(cid) => c.button(cid).channels.clone(),
        Slot::Move(cid, g) => c.button(cid).moves.get(g).map(|m| m.channels.clone()).unwrap_or_default(),
    }
}

fn slot_channels_mut(c: &mut Config, slot: Slot) -> Option<&mut Vec<u8>> {
    match slot {
        Slot::Press(cid) => Some(&mut c.button_mut(cid).channels),
        Slot::Move(cid, g) => Some(&mut c.button_mut(cid).moves.get_mut(g)?.channels),
    }
}

#[derive(PartialEq)]
enum Uninstall {
    Idle,
    Confirming,
    /// Waiting for the tray app to hand the button back and quit.
    Waiting(Instant),
    Done,
}

struct SettingsApp {
    tab: Tab,
    /// The button whose settings are shown, and for the gesture button which gesture.
    button_tab: u16,
    gesture_tab: Gesture,
    uninstall: Uninstall,
    cfg: Config,
    devices: Vec<DeviceInfo>,
    status: StatusFile,
    autostart: Option<AutoLaunch>,
    autostart_on: bool,
    error: Option<String>,
    last_refresh: Instant,
    /// Frames left in which to bring the window to the front (see `bring_to_front`).
    raise_frames: u8,
    /// Window title last set, to follow a language change.
    title: &'static str,
    raise: Arc<AtomicBool>,
    icon: egui::TextureHandle,
}

impl SettingsApp {
    fn new(ctx: &egui::Context, raise: Arc<AtomicBool>) -> SettingsApp {
        let autostart = crate::tray::autostart();
        let autostart_on = autostart.as_ref().is_some_and(|a| a.is_enabled().unwrap_or(false));
        let image = egui::ColorImage::from_rgba_unmultiplied([ICON_SIDE as usize; 2], ICON_RGBA);
        let mut app = SettingsApp {
            tab: Tab::Switching,
            button_tab: BUTTON_WHEEL_MODE,
            gesture_tab: Gesture::Press,
            uninstall: Uninstall::Idle,
            cfg: Config::load_or_create().unwrap_or_default(),
            devices: Vec::new(),
            status: StatusFile::default(),
            autostart,
            autostart_on,
            error: None,
            last_refresh: Instant::now() - REFRESH,
            // Started from the tray app: come to the front once at start.
            raise_frames: 2,
            raise,
            title: texts().settings_title,
            icon: ctx.load_texture("app-icon", image, egui::TextureOptions::LINEAR),
        };
        app.refresh();
        app
    }

    /// Re-reads settings, the remembered devices and what the engine sees, so the window
    /// also reflects changes made from the tray menu or by switching computers.
    fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        if let Ok(cfg) = Config::load_or_create() {
            crate::i18n::set_language(&cfg.language);
            self.cfg = cfg;
        }
        self.status = shared::load_status_file();
        self.devices = shared::load_catalog();
        for d in &mut self.devices {
            d.online = self.status.online.iter().any(|k| k == &d.key);
        }
    }

    /// Applies one change on top of the file's current content, validates, saves.
    fn apply(&mut self, change: impl FnOnce(&mut Config)) {
        let mut next = Config::load_or_create().unwrap_or_else(|_| self.cfg.clone());
        change(&mut next);
        match next.validate().and_then(|_| next.save()) {
            Ok(()) => {
                crate::i18n::set_language(&next.language);
                self.cfg = next;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("{e:#}")),
        }
    }

    /// The master mouse's key. With "auto" and the mouse away (so the engine has not
    /// picked it yet), assume the first known mouse with the button, as auto would.
    fn master_key(&self) -> Option<String> {
        if self.cfg.trigger_device.eq_ignore_ascii_case("auto") {
            self.status
                .trigger_key
                .clone()
                .or_else(|| self.devices.iter().find(|d| d.master).map(|d| d.key.clone()))
        } else {
            Some(self.cfg.trigger_device.clone())
        }
    }
}

impl eframe::App for SettingsApp {
    /// Runs even while the window is minimized or fully covered (when `ui` does not),
    /// so "Settings…" can still bring it back.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.bring_to_front(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t = texts();
        if self.title != t.settings_title {
            self.title = t.settings_title;
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(t.settings_title.into()));
        }
        let p = Palette::new(ui.visuals().dark_mode);
        let panel = egui::Frame::new().fill(p.bg).inner_margin(Margin::symmetric(18, 12));
        // After uninstalling, show only the result: re-reading the settings now would
        // create the deleted settings folder again.
        if self.uninstall_progress() {
            ctx.request_repaint_after(Duration::from_millis(300));
            egui::CentralPanel::default().frame(panel).show(ui, |ui| self.uninstall_result(ui, t, p));
            return;
        }
        if self.last_refresh.elapsed() >= REFRESH {
            self.refresh();
        }
        ctx.request_repaint_after(REFRESH);

        // The header and the tabs stay in place; only the page below them scrolls.
        egui::CentralPanel::default().frame(panel).show(ui, |ui| {
            self.header(ui, t, p);
            ui.add_space(12.0);
            let tabs = [
                (Tab::Switching, t.section_switching),
                (Tab::Devices, t.section_devices),
                (Tab::General, t.section_general),
            ];
            let labels: Vec<String> = tabs.iter().map(|(_, l)| l.to_string()).collect();
            if let Some(i) = segments(ui, p, "tabs", &labels, Role::Tab, SEG_NORMAL, |i| tabs[i].0 == self.tab) {
                self.tab = tabs[i].0;
            }
            // Pages are short enough to fit; if one grows (many devices, a long error),
            // the window grows with it instead of scrolling.
            let page = egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                {
                    if let Some(err) = self.error.clone() {
                        ui.add_space(10.0);
                        egui::Frame::new()
                            .fill(p.red.gamma_multiply(0.12))
                            .stroke(Stroke::new(1.0, p.red.gamma_multiply(0.6)))
                            .corner_radius(8)
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new(err).color(p.red));
                            });
                    }
                    match self.tab {
                        Tab::Switching => self.switching_section(ui, t, p),
                        Tab::Devices => {
                            self.master_section(ui, t, p);
                            self.group_section(ui, t, p);
                        }
                        Tab::General => {
                            notice(ui, p, t);
                            self.general_section(ui, t, p);
                        }
                    }

                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format!("{} · flockswitch {}", t.applies_immediately, env!("CARGO_PKG_VERSION")))
                                .size(12.0)
                                .color(p.muted),
                        );
                    });
                }
            });
            let overflow = page.content_size.y - page.inner_rect.height();
            if overflow > 1.0 {
                let (size, monitor) = ctx.input(|i| (i.viewport().inner_rect.map(|r| r.size()), i.viewport().monitor_size));
                if let Some(size) = size {
                    let max = monitor.map_or(f32::INFINITY, |m| m.y - 80.0);
                    let height = (size.y + overflow).min(max);
                    if height > size.y + 1.0 {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(size.x, height)));
                    }
                }
            }
        });
    }
}

impl SettingsApp {
    /// Un-minimizes and focuses the window when it starts and whenever "Settings…" is
    /// chosen again. Repeated on the next frame: un-minimizing takes effect first.
    fn bring_to_front(&mut self, ctx: &egui::Context) {
        if self.raise.swap(false, Ordering::Relaxed) {
            self.raise_frames = 2;
        }
        if self.raise_frames == 0 {
            return;
        }
        self.raise_frames -= 1;
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint_after(Duration::from_millis(100));
    }

    /// App icon, name, what the engine is doing and the last switch.
    fn header(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 48.0), Sense::hover());
        let painter = ui.painter();
        let icon = Rect::from_min_size(rect.min + Vec2::new(0.0, 2.0), Vec2::splat(44.0));
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        painter.image(self.icon.id(), icon, uv, Color32::WHITE);

        let x = icon.right() + 12.0;
        painter.text(Pos2::new(x, rect.top() + 3.0), Align2::LEFT_TOP, "flockswitch", bold(19.0), p.text);

        let (color, line) = if !self.cfg.enabled {
            (p.muted, t.paused.to_string())
        } else if let Some(trigger) = &self.status.trigger {
            (p.green, format!("{} · {trigger}", t.running))
        } else {
            (p.amber, t.searching.to_string())
        };
        let y = rect.top() + 35.0;
        painter.circle_filled(Pos2::new(x + 5.0, y), 4.5, color);
        let status = painter.text(Pos2::new(x + 16.0, y), Align2::LEFT_CENTER, line, FontId::proportional(14.0), p.text);

        if let Some(last) = &self.status.last_switch {
            let label = format!("{} · {} {last}", t.last_switch, t.channel);
            let galley = painter.layout_no_wrap(label, FontId::proportional(12.0), p.muted);
            // On the same line when it fits, otherwise left out (the tray menu shows it too).
            if status.right() + 16.0 + galley.size().x + 16.0 <= rect.right() {
                let pill = Rect::from_min_size(
                    Pos2::new(rect.right() - galley.size().x - 16.0, y - 11.0),
                    Vec2::new(galley.size().x + 16.0, 22.0),
                );
                painter.rect_filled(pill, 11, p.card);
                painter.rect_stroke(pill, 11, Stroke::new(1.0, p.border), StrokeKind::Inside);
                painter.galley(pill.center() - galley.size() / 2.0, galley, p.muted);
            }
        }
    }

    /// The master switch, then the settings of one button at a time.
    fn switching_section(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        ui.add_space(12.0);
        info_box(ui, p, t.switching_help);
        ui.add_space(10.0);
        card(ui, p, |ui| {
            let mut enabled = self.cfg.enabled;
            if switch_row(ui, p, t.enabled, None, &mut enabled) {
                self.apply(|c| c.enabled = enabled);
            }
        });

        let buttons = [
            (BUTTON_WHEEL_MODE, t.button_wheel_mode_short, t.button_wheel_mode),
            (BUTTON_GESTURE, t.button_gesture_short, t.button_gesture),
            (BUTTON_MIDDLE, t.button_middle_short, t.button_middle),
        ];
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new(t.section_buttons).font(bold(13.0)).color(p.muted));
        });
        ui.add_space(4.0);
        let labels: Vec<String> = buttons
            .iter()
            .map(|(cid, short, _)| if self.cfg.button(*cid).on { format!("{short} ✓") } else { short.to_string() })
            .collect();
        if let Some(i) = segments(ui, p, "button-tabs", &labels, Role::Tab, SEG_NORMAL, |i| buttons[i].0 == self.button_tab) {
            self.button_tab = buttons[i].0;
        }
        let (cid, _, label) = buttons.into_iter().find(|b| b.0 == self.button_tab).unwrap_or(buttons[0]);

        ui.add_space(8.0);
        card(ui, p, |ui| {
            // Only known while the master mouse is on this computer.
            let missing = self.status.trigger.is_some() && !self.status.buttons.contains(&cid);
            let mut on = self.cfg.button(cid).on;
            if switch_row(ui, p, label, missing.then_some(t.not_on_mouse), &mut on) {
                if !on && self.cfg.triggers() == [cid] {
                    self.error = Some(t.need_one_button.into());
                } else {
                    self.apply(|c| c.button_mut(cid).on = on);
                }
            }
            if !self.cfg.button(cid).on {
                return;
            }
            divider(ui, p);
            if cid == BUTTON_GESTURE {
                self.gesture_rows(ui, t, p, cid);
            } else {
                self.action_rows(ui, t, p, Slot::Press(cid));
            }
            if cid == BUTTON_MIDDLE {
                hint(ui, p, t.buttons_hint);
            }
            ui.add_space(10.0);
        });
    }

    /// Press and the four hold-and-move gestures, one at a time.
    fn gesture_rows(&mut self, ui: &mut Ui, t: &Texts, p: Palette, cid: u16) {
        let gestures = [
            (Gesture::Press, t.gesture_press),
            (Gesture::Up, "↑"),
            (Gesture::Down, "↓"),
            (Gesture::Left, "←"),
            (Gesture::Right, "→"),
        ];
        let moves = self.cfg.button(cid).moves.clone();
        let labels: Vec<String> = gestures
            .iter()
            .map(|(g, l)| if moves.get(*g).is_some_and(|m| m.on) { format!("{l} ✓") } else { l.to_string() })
            .collect();
        ui.add_space(10.0);
        if let Some(i) = segments(ui, p, "gesture-tabs", &labels, Role::Tab, SEG_SMALL, |i| gestures[i].0 == self.gesture_tab) {
            self.gesture_tab = gestures[i].0;
        }
        hint(ui, p, t.gesture_hint);
        let g = self.gesture_tab;
        if g == Gesture::Press {
            self.action_rows(ui, t, p, Slot::Press(cid));
            return;
        }
        ui.add_space(4.0);
        let mut on = moves.get(g).is_some_and(|m| m.on);
        if switch_row(ui, p, t.gesture_on, None, &mut on) {
            self.apply(|c| {
                if let Some(m) = c.button_mut(cid).moves.get_mut(g) {
                    m.on = on;
                }
            });
        }
        if on {
            self.action_rows(ui, t, p, Slot::Move(cid, g));
        }
    }

    /// 사용할 채널 of one slot: toggles, several can be on.
    fn action_rows(&mut self, ui: &mut Ui, t: &Texts, p: Palette, slot: Slot) {
        let channels = slot_channels(&self.cfg, slot);
        let numbers: Vec<String> = (1..=3).map(|n| n.to_string()).collect();
        let seg_width = (ui.available_width() * 0.5).min(210.0);
        let mut clicked = None;
        ui.add_space(2.0);
        labeled_row(ui, p, t.channels, seg_width, ROW_SMALL, |ui| {
            clicked = segments(ui, p, &format!("channels-{slot:?}"), &numbers, Role::CheckBox, SEG_SMALL, |i| {
                channels.contains(&(i as u8 + 1))
            })
            .map(|i| i as u8 + 1);
            false
        });
        let Some(n) = clicked else { return };
        let was_on = channels.contains(&n);
        if was_on && channels.len() == 1 {
            self.error = Some(t.need_one_channel.into());
            return;
        }
        self.apply(|c| {
            if let Some(channels) = slot_channels_mut(c, slot) {
                if was_on {
                    channels.retain(|&x| x != n);
                } else {
                    channels.push(n);
                    channels.sort_unstable();
                    channels.dedup();
                }
            }
        });
    }

    fn master_section(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        section(ui, p, t.master, |ui| {
            let auto = self.cfg.trigger_device.eq_ignore_ascii_case("auto");
            let now = self.status.trigger.as_ref().map(|name| format!("{} {name}", t.current));
            let row = Row { title: t.master_auto, subtitle: now.as_deref(), badge: None };
            if choice_row(ui, p, Mark::Radio, auto, &row).clicked() && !auto {
                self.apply(|c| c.trigger_device = "auto".into());
            }
            let masters: Vec<DeviceInfo> = self.devices.iter().filter(|d| d.master).cloned().collect();
            if masters.is_empty() {
                divider(ui, p);
                empty(ui, p, t.no_master);
            }
            for d in masters {
                divider(ui, p);
                let chosen = !auto && self.cfg.trigger_device.eq_ignore_ascii_case(&d.key);
                let subtitle = device_subtitle(&d, t);
                let row = Row { title: &d.name, subtitle: Some(&subtitle), badge: away_badge(&d, t, p) };
                if choice_row(ui, p, Mark::Radio, chosen, &row).clicked() && !chosen {
                    let key = d.key.clone();
                    self.apply(|c| c.trigger_device = key);
                }
            }
        });
    }

    fn group_section(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        section(ui, p, t.follow, |ui| {
            ui.add_space(4.0);
            hint(ui, p, t.follow_hint);
            let master = self.master_key();
            let others: Vec<DeviceInfo> = self
                .devices
                .iter()
                .filter(|d| !master.as_ref().is_some_and(|m| m.eq_ignore_ascii_case(&d.key)))
                .cloned()
                .collect();
            let keys: Vec<String> = others.iter().map(|d| d.key.clone()).collect();

            let all = self.cfg.follows_all();
            let row = Row { title: t.follow_all, subtitle: None, badge: None };
            if choice_row(ui, p, Mark::Check, all, &row).clicked() {
                self.apply(|c| c.follow = if all { vec![] } else { vec!["all".into()] });
            }
            if others.is_empty() {
                divider(ui, p);
                empty(ui, p, t.no_devices);
            }
            for d in &others {
                divider(ui, p);
                let on = all || self.cfg.follow.iter().any(|f| f.eq_ignore_ascii_case(&d.key));
                let subtitle = device_subtitle(d, t);
                let row = Row { title: &d.name, subtitle: Some(&subtitle), badge: away_badge(d, t, p) };
                if choice_row(ui, p, Mark::Check, on, &row).clicked() {
                    let key = d.key.clone();
                    let keys = keys.clone();
                    self.apply(|c| c.toggle_follow(&keys, &key));
                }
            }
        });
    }

    /// Language, start at login, and uninstall.
    fn general_section(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        ui.add_space(12.0);
        card(ui, p, |ui| {
            let mut language = self.cfg.language.clone();
            let choices = [("ko", "한국어"), ("en", "English")];
            let width = (ui.available_width() * 0.62).min(250.0);
            let mut chosen = language.as_str();
            if labeled_row(ui, p, t.language, width, ROW_NORMAL, |ui| segmented(ui, p, "language", &mut chosen, &choices, SEG_NORMAL)) {
                language = chosen.to_string();
                self.apply(|c| c.language = language);
            }
            divider(ui, p);
            if let Some(al) = &self.autostart {
                let mut on = self.autostart_on;
                if switch_row(ui, p, t.autostart, None, &mut on) {
                    let res = if on { al.enable() } else { al.disable() };
                    match res {
                        Ok(()) => self.autostart_on = on,
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                divider(ui, p);
            }
            self.uninstall_row(ui, t, p);
        });
    }

    /// What uninstalling does on the left, the button (then confirm/cancel) on the right.
    fn uninstall_row(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        let labels: &[(&str, ButtonKind)] = match self.uninstall {
            Uninstall::Confirming => &[(t.uninstall_confirm, ButtonKind::DangerFilled), (t.cancel, ButtonKind::Plain)],
            _ => &[(t.uninstall_button, ButtonKind::Danger)],
        };
        let buttons: f32 = labels.iter().map(|(l, k)| button_width(ui, l, *k)).sum::<f32>() + 8.0 * (labels.len() - 1) as f32;
        let width = ui.available_width();
        let text = ui.painter().layout(t.uninstall_explain.to_string(), FontId::proportional(12.5), p.muted, width - buttons - 16.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, (text.size().y + 20.0).max(48.0)), Sense::hover());
        ui.painter().galley(Pos2::new(rect.left(), rect.center().y - text.size().y / 2.0), text, p.muted);
        let area = Rect::from_min_max(
            Pos2::new(rect.right() - buttons, rect.center().y - 15.0),
            Pos2::new(rect.right(), rect.center().y + 15.0),
        );
        let mut clicked = None;
        ui.scope_builder(UiBuilder::new().max_rect(area), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.horizontal(|ui| {
                for (i, (label, kind)) in labels.iter().enumerate() {
                    if button(ui, p, label, *kind).clicked() {
                        clicked = Some(i);
                    }
                }
            });
        });
        match (&self.uninstall, clicked) {
            (Uninstall::Idle, Some(_)) => self.uninstall = Uninstall::Confirming,
            (Uninstall::Confirming, Some(0)) => self.start_uninstall(),
            (Uninstall::Confirming, Some(_)) => self.uninstall = Uninstall::Idle,
            _ => {}
        }
    }

    /// Asks the running tray app to uninstall (it owns the button), or does it here
    /// when no tray app is running.
    fn start_uninstall(&mut self) {
        if crate::main_instance_running() {
            match std::fs::write(shared::uninstall_request_path(), b"") {
                Ok(()) => self.uninstall = Uninstall::Waiting(Instant::now()),
                Err(e) => {
                    self.error = Some(e.to_string());
                    self.uninstall = Uninstall::Idle;
                }
            }
        } else {
            crate::uninstall();
            self.uninstall = Uninstall::Done;
        }
    }

    /// Advances the uninstall state; returns true once it has started.
    fn uninstall_progress(&mut self) -> bool {
        if let Uninstall::Waiting(since) = self.uninstall {
            if !crate::main_instance_running() {
                self.uninstall = Uninstall::Done;
            } else if since.elapsed() > Duration::from_secs(15) {
                self.error = Some("flockswitch did not respond; quit it from the menu and try again".into());
                let _ = std::fs::remove_file(shared::uninstall_request_path());
                self.uninstall = Uninstall::Idle;
            }
        }
        matches!(self.uninstall, Uninstall::Waiting(_) | Uninstall::Done)
    }

    fn uninstall_result(&mut self, ui: &mut Ui, t: &Texts, p: Palette) {
        self.header(ui, t, p);
        ui.add_space(16.0);
        card(ui, p, |ui| {
            ui.add_space(12.0);
            if self.uninstall == Uninstall::Done {
                ui.label(t.uninstall_done);
                ui.add_space(8.0);
                if button(ui, p, t.close, ButtonKind::Plain).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else {
                ui.label(t.uninstall_waiting);
            }
            ui.add_space(12.0);
        });
    }
}

fn device_subtitle(d: &DeviceInfo, t: &Texts) -> String {
    let kind = match d.kind.as_str() {
        "mouse" => t.kind_mouse,
        "keyboard" => t.kind_keyboard,
        _ => t.kind_other,
    };
    format!("{kind} · {}", d.key)
}

fn away_badge<'a>(d: &DeviceInfo, t: &'a Texts, p: Palette) -> Option<(&'a str, Color32)> {
    (!d.online).then_some((t.away, p.amber))
}

// ---- Building blocks -------------------------------------------------------------

/// A titled card: the title sits above a rounded panel, as in the system settings.
fn section(ui: &mut Ui, p: Palette, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(title).font(bold(13.0)).color(p.muted));
    });
    ui.add_space(2.0);
    card(ui, p, add);
}

fn card(ui: &mut Ui, p: Palette, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(12)
        .inner_margin(Margin::symmetric(18, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui);
        });
}

/// What the app expects before it is set up: Logi Options+ with the buttons on "No action".
fn notice(ui: &mut Ui, p: Palette, t: &Texts) {
    ui.add_space(12.0);
    egui::Frame::new()
        .fill(p.accent.gamma_multiply(0.10))
        .stroke(Stroke::new(1.0, p.accent.gamma_multiply(0.45)))
        .corner_radius(12)
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(RichText::new(format!("ⓘ  {}", t.notice_title)).font(bold(14.0)).color(p.accent));
            ui.label(RichText::new(t.notice).size(12.5).color(p.text));
        });
}

/// A quiet box of explanation text.
fn info_box(ui: &mut Ui, p: Palette, text: &str) {
    egui::Frame::new()
        .fill(p.card.gamma_multiply(0.5))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).size(12.5).color(p.muted));
        });
}

/// A thin line between rows, edge to edge inside the card.
fn divider(ui: &mut Ui, p: Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0, p.border));
}

fn hint(ui: &mut Ui, p: Palette, text: &str) {
    ui.add_space(2.0);
    ui.label(RichText::new(text).size(12.5).color(p.muted));
}

fn empty(ui: &mut Ui, p: Palette, text: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(text).size(13.0).color(p.muted));
    ui.add_space(8.0);
}

/// A row with a label and an on/off switch on the right. Returns true when flipped.
/// A `note` shows in smaller text under the label.
fn switch_row(ui: &mut Ui, p: Palette, label: &str, note: Option<&str>, on: &mut bool) -> bool {
    let h = if note.is_some() { 50.0 } else { 40.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| WidgetInfo::selected(Role::Switch, true, *on, label));
    if resp.clicked() {
        *on = !*on;
    }
    let painter = ui.painter();
    let left = Pos2::new(rect.left(), rect.center().y);
    match note {
        Some(note) => {
            painter.text(left - Vec2::new(0.0, 1.0), Align2::LEFT_BOTTOM, label, FontId::proportional(14.5), p.text);
            painter.text(left + Vec2::new(0.0, 3.0), Align2::LEFT_TOP, note, FontId::proportional(12.5), p.amber);
        }
        None => {
            painter.text(left, Align2::LEFT_CENTER, label, FontId::proportional(14.5), p.text);
        }
    }

    let how_on = ui.ctx().animate_bool_responsive(resp.id, *on);
    let track = Rect::from_center_size(Pos2::new(rect.right() - 20.0, rect.center().y), Vec2::new(40.0, 24.0));
    painter.rect_filled(track, 12, p.track.lerp_to_gamma(p.accent, how_on));
    let x = egui::lerp(track.left() + 12.0..=track.right() - 12.0, how_on);
    painter.circle_filled(Pos2::new(x, track.center().y), 9.5, Color32::WHITE);
    resp.clicked()
}

const ROW_NORMAL: f32 = 52.0;
const ROW_SMALL: f32 = 42.0;

/// A row `height` tall with a bold label on the left and a control `width` wide on the
/// right.
fn labeled_row(
    ui: &mut Ui,
    p: Palette,
    label: &str,
    width: f32,
    height: f32,
    control: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let size = if height < ROW_NORMAL { 13.5 } else { 14.0 };
    ui.painter().text(Pos2::new(rect.left(), rect.center().y), Align2::LEFT_CENTER, label, bold(size), p.text);
    let area = Rect::from_min_max(
        Pos2::new(rect.right() - width, rect.center().y - 15.0),
        Pos2::new(rect.right(), rect.center().y + 15.0),
    );
    ui.scope_builder(UiBuilder::new().max_rect(area), control).inner
}

/// Equal-width buttons in one row. `selected(i)` says which are on; returns the index
/// clicked. One-choice rows (mode, tabs, language) draw them on a shared bar; toggle
/// rows (channels) as separate outlined buttons in the same positions, so the rows
/// line up.
/// Height and text size of a segment bar.
#[derive(Clone, Copy)]
struct SegSize {
    height: f32,
    text: f32,
}
const SEG_NORMAL: SegSize = SegSize { height: 30.0, text: 14.0 };
const SEG_SMALL: SegSize = SegSize { height: 26.0, text: 13.0 };

fn segments(
    ui: &mut Ui,
    p: Palette,
    name: &str,
    labels: &[String],
    role: Role,
    size: SegSize,
    selected: impl Fn(usize) -> bool,
) -> Option<usize> {
    let toggles = role == Role::CheckBox;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), size.height), Sense::hover());
    if !toggles {
        ui.painter().rect_filled(rect, 8, p.track.gamma_multiply(0.55));
    }
    let w = rect.width() / labels.len() as f32;
    let mut clicked = None;
    for (i, label) in labels.iter().enumerate() {
        let seg = Rect::from_min_size(Pos2::new(rect.left() + w * i as f32, rect.top()), Vec2::new(w, rect.height()))
            .shrink(3.0);
        // Named per bar: the rows sit in look-alike child areas, so an id derived from
        // the area would be shared and one bar's click would also hit the other.
        let id = egui::Id::unique(("segment", name, i));
        let resp = ui.interact(seg, id, Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let on = selected(i);
        resp.widget_info(|| WidgetInfo::selected(role, true, on, label));
        if resp.clicked() {
            clicked = Some(i);
        }
        let painter = ui.painter();
        let (fill, color) = if on {
            (p.accent, Color32::WHITE)
        } else if resp.hovered() {
            (if toggles { p.hover } else { p.card.gamma_multiply(0.6) }, p.text)
        } else {
            (if toggles { p.card } else { Color32::TRANSPARENT }, p.text)
        };
        painter.rect_filled(seg, 7, fill);
        if toggles && !on {
            painter.rect_stroke(seg, 7, Stroke::new(1.0, p.track), StrokeKind::Inside);
        }
        let font = if on { bold(size.text) } else { FontId::proportional(size.text) };
        // A trailing " ✓" (button or gesture is on) is drawn, not taken from the font:
        // Windows' Korean font has no check mark.
        let (text, mark) = match label.strip_suffix(" ✓") {
            Some(text) => (text, true),
            None => (label.as_str(), false),
        };
        let galley = painter.layout_no_wrap(text.to_string(), font, color);
        let mark_width = if mark { size.text * 0.9 } else { 0.0 };
        let left = seg.center().x - (galley.size().x + mark_width) / 2.0;
        let text_right = left + galley.size().x;
        painter.galley(Pos2::new(left, seg.center().y - galley.size().y / 2.0), galley, color);
        if mark {
            let u = size.text / 14.0;
            let (x, y) = (text_right + 4.0 * u, seg.center().y);
            let tick = Stroke::new(1.6 * u, color);
            painter.line_segment([Pos2::new(x, y), Pos2::new(x + 3.0 * u, y + 3.0 * u)], tick);
            painter.line_segment([Pos2::new(x + 3.0 * u, y + 3.0 * u), Pos2::new(x + 8.0 * u, y - 3.5 * u)], tick);
        }
    }
    clicked
}

/// One-of-several choice on a segment bar. Returns true on change.
fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    p: Palette,
    name: &str,
    value: &mut T,
    options: &[(T, &str)],
    size: SegSize,
) -> bool {
    let labels: Vec<String> = options.iter().map(|(_, l)| l.to_string()).collect();
    match segments(ui, p, name, &labels, Role::RadioButton, size, |i| options[i].0 == *value) {
        Some(i) if options[i].0 != *value => {
            *value = options[i].0;
            true
        }
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Radio,
    Check,
}

struct Row<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    badge: Option<(&'a str, Color32)>,
}

/// A selectable list row: radio/check mark, title, optional second line and badge.
fn choice_row(ui: &mut Ui, p: Palette, mark: Mark, selected: bool, row: &Row) -> Response {
    let h = if row.subtitle.is_some() { 46.0 } else { 38.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let role = if mark == Mark::Radio { Role::RadioButton } else { Role::CheckBox };
    resp.widget_info(|| WidgetInfo::selected(role, true, selected, row.title));
    let painter = ui.painter();
    if resp.hovered() {
        painter.rect_filled(rect.expand2(Vec2::new(8.0, -3.0)), 8, p.hover);
    }

    let c = Pos2::new(rect.left() + 10.0, rect.center().y);
    match (mark, selected) {
        (Mark::Radio, true) => {
            painter.circle_filled(c, 10.0, p.accent);
            painter.circle_filled(c, 4.0, Color32::WHITE);
        }
        (Mark::Radio, false) => {
            painter.circle_stroke(c, 9.25, Stroke::new(1.5, p.track));
        }
        (Mark::Check, true) => {
            let r = Rect::from_center_size(c, Vec2::splat(20.0));
            painter.rect_filled(r, 6, p.accent);
            let tick = Stroke::new(2.2, Color32::WHITE);
            painter.line_segment([c + Vec2::new(-5.0, 0.5), c + Vec2::new(-1.5, 4.0)], tick);
            painter.line_segment([c + Vec2::new(-1.5, 4.0), c + Vec2::new(5.5, -3.5)], tick);
        }
        (Mark::Check, false) => {
            let r = Rect::from_center_size(c, Vec2::splat(18.5));
            painter.rect_stroke(r, 5, Stroke::new(1.5, p.track), StrokeKind::Inside);
        }
    }

    let x = rect.left() + 32.0;
    let mut right = rect.right();
    if let Some((text, color)) = row.badge {
        let galley = painter.layout_no_wrap(text.to_string(), FontId::proportional(12.0), color);
        let pill = Rect::from_min_size(
            Pos2::new(right - galley.size().x - 16.0, rect.center().y - 11.0),
            Vec2::new(galley.size().x + 16.0, 22.0),
        );
        painter.rect_filled(pill, 11, color.gamma_multiply(0.14));
        painter.galley(pill.center() - galley.size() / 2.0, galley, color);
        right = pill.left() - 8.0;
    }
    let clip = Rect::from_min_max(rect.min, Pos2::new(right, rect.max.y));
    let painter = painter.with_clip_rect(clip.intersect(painter.clip_rect()));
    match row.subtitle {
        Some(sub) => {
            painter.text(Pos2::new(x, rect.center().y - 1.0), Align2::LEFT_BOTTOM, row.title, FontId::proportional(14.5), p.text);
            painter.text(Pos2::new(x, rect.center().y + 3.0), Align2::LEFT_TOP, sub, FontId::proportional(12.5), p.muted);
        }
        None => {
            painter.text(Pos2::new(x, rect.center().y), Align2::LEFT_CENTER, row.title, FontId::proportional(14.5), p.text);
        }
    }
    resp
}

#[derive(Clone, Copy, PartialEq)]
enum ButtonKind {
    Plain,
    Danger,
    DangerFilled,
}

fn button_font(kind: ButtonKind) -> FontId {
    if kind == ButtonKind::DangerFilled { bold(14.0) } else { FontId::proportional(14.0) }
}

fn button_width(ui: &Ui, label: &str, kind: ButtonKind) -> f32 {
    ui.painter().layout_no_wrap(label.to_string(), button_font(kind), Color32::WHITE).size().x + 28.0
}

fn button(ui: &mut Ui, p: Palette, label: &str, kind: ButtonKind) -> Response {
    let font = button_font(kind);
    let size = Vec2::new(button_width(ui, label, kind), 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let hovered = resp.hovered();
    let (fill, stroke, color) = match kind {
        ButtonKind::Plain => (if hovered { p.hover } else { p.card }, p.track, p.text),
        ButtonKind::Danger => (if hovered { p.red.gamma_multiply(0.10) } else { p.card }, p.red.gamma_multiply(0.7), p.red),
        ButtonKind::DangerFilled => (if hovered { p.red.gamma_multiply(0.85) } else { p.red }, p.red, Color32::WHITE),
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(9), fill);
    painter.rect_stroke(rect, 9, Stroke::new(1.0, stroke), StrokeKind::Inside);
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, color);
    resp
}
