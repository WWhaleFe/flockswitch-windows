//! Notification-area icon with the settings menu.
//! The engine runs on its own thread; the menu edits the shared config and the
//! engine picks the change up on its next loop.

use crate::engine::Engine;
use crate::i18n::{texts, Texts};
use crate::icon;
use crate::shared::{Shared, Status};
use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

#[derive(Clone)]
enum Action {
    Enabled,
    Settings,
    Quit,
}

struct Ui {
    shared: Arc<Shared>,
    tray: Option<TrayIcon>,
    actions: HashMap<MenuId, Action>,
    shown: (Status, u64),
    uninstall_requested: bool,
    shortcuts: Option<crate::hotkey::Shortcuts>,
}

pub fn run(shared: Arc<Shared>) -> Result<()> {
    // Ctrl+C, logout or shutdown: let the loop below hand the button back and exit.
    let s = shared.clone();
    ctrlc::set_handler(move || s.stop.store(true, std::sync::atomic::Ordering::SeqCst))?;
    let engine = spawn_engine(shared.clone());

    let event_loop = EventLoopBuilder::new().build();

    let mut ui = Ui {
        shared: shared.clone(),
        tray: None,
        actions: HashMap::new(),
        shown: (Status::default(), u64::MAX),
        uninstall_requested: false,
        shortcuts: None,
    };
    let mut engine = Some(engine);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(500));

        // The tray icon is created once the event loop is running.
        if let Event::NewEvents(StartCause::Init) = event {
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
                    shared.stop.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }
        // Uninstall asked for from the settings window or the uninstaller,
        // quit from the installer (before an upgrade).
        let request = crate::shared::uninstall_request_path();
        if request.exists() {
            let _ = std::fs::remove_file(&request);
            ui.uninstall_requested = true;
            shared.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let request = crate::shared::quit_request_path();
        if request.exists() {
            let _ = std::fs::remove_file(&request);
            shared.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let destroyed = matches!(event, Event::LoopDestroyed);
        if shared.stopped() || destroyed {
            shared.stop.store(true, std::sync::atomic::Ordering::SeqCst);
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

        // Redraw when the engine found/lost devices or switched.
        let now = (shared.status(), shared.version());
        if ui.tray.is_some() && now != ui.shown {
            ui.refresh();
        }
    });
}

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

/// Start at login through the Run registry key. Not for the Store version: its package
/// declares a startup task instead (a Run entry written from a package is not used).
pub fn autostart() -> Option<AutoLaunch> {
    if crate::store::packaged() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    AutoLaunchBuilder::new()
        .set_app_name("flockswitch")
        .set_app_path(&exe.to_string_lossy())
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .build()
        .ok()
}

/// White arrows on a blue tile (grey when paused), visible on light and dark taskbars.
fn tray_icon(enabled: bool) -> Result<Icon> {
    let tile = if enabled { [0x25, 0x63, 0xEB, 255] } else { [0x6B, 0x72, 0x80, 255] };
    Ok(Icon::from_rgba(icon::arrows(32, [255, 255, 255, 255], Some(tile)), 32, 32)?)
}

impl Ui {
    fn create_tray(&mut self) -> Result<()> {
        let menu = self.build_menu();
        let cfg = self.shared.config();
        let builder = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("flockswitch");
        let builder = builder.with_icon(tray_icon(cfg.enabled)?);
        let tray = builder.build()?;
        self.tray = Some(tray);
        self.shown = (self.shared.status(), self.shared.version());
        Ok(())
    }

    fn refresh(&mut self) {
        let menu = self.build_menu();
        let enabled = self.shared.config().enabled;
        if let Some(tray) = &self.tray {
            tray.set_menu(Some(Box::new(menu)));
            if let Ok(icon) = tray_icon(enabled) {
                let _ = tray.set_icon(Some(icon));
            }
        }
        self.shown = (self.shared.status(), self.shared.version());
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

    fn build_menu(&mut self) -> Menu {
        self.actions.clear();
        let t: &Texts = texts();
        let cfg = self.shared.config();
        let st = self.shared.status();
        let menu = Menu::new();

        let status = if !cfg.enabled {
            format!("○ {}", t.paused)
        } else if let Some(trigger) = &st.trigger {
            format!("● {} — {trigger}", t.running)
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
        let settings = self.item(t.settings, Action::Settings);
        let _ = menu.append(&settings);
        let _ = menu.append(&PredefinedMenuItem::separator());

        let _ = menu.append(&MenuItem::new(format!("flockswitch {}", env!("CARGO_PKG_VERSION")), false, None));
        let _ = menu.append(&PredefinedMenuItem::separator());
        let quit = self.item(t.quit, Action::Quit);
        let _ = menu.append(&quit);
        menu
    }

    /// Applies a menu action. Returns true when the app should quit.
    fn handle(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::Enabled => {
                if let Err(e) = self.shared.update_config(|c| c.enabled = !c.enabled) {
                    log::info!("setting not changed: {e:#}");
                }
            }
            Action::Settings => open_settings_window(),
        }
        self.refresh();
        false
    }
}

/// Starts the settings window as a separate process of this same program.
pub fn open_settings_window() {
    match std::env::current_exe() {
        Ok(exe) => {
            if let Err(e) = std::process::Command::new(exe).arg("settings").spawn() {
                log::warn!("cannot open the settings window: {e}");
            }
        }
        Err(e) => log::warn!("cannot open the settings window: {e}"),
    }
}

