//! flockswitch — use a spare Logitech mouse button as an Easy-Switch key that moves the
//! mouse and its companion keyboard to another computer together.

// No console window when started from Explorer or at login; command-line use attaches
// to the parent console instead (see `attach_console`).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod config;
mod device;
mod engine;
mod hidpp;
mod i18n;
mod icon;
mod settings_ui;
mod shared;
mod tray;

use anyhow::{bail, Result};
use config::Config;
use engine::Engine;
use shared::Shared;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

const USAGE: &str = "\
flockswitch — move your Logitech mouse and keyboard to another computer with one button

USAGE:
    flockswitch              start in the notification area (default)
    flockswitch settings     open the settings window
    flockswitch run          run without an icon, logging to the terminal
    flockswitch list         show devices, channels and the trigger button
    flockswitch switch <N>   move all following devices to channel N (1-3) now
    flockswitch config       print the config file path (created on first use)
    flockswitch quit         ask the running flockswitch to give the button back and quit
    flockswitch uninstall    remove start-at-login and all settings (also quits the running
                            flockswitch), then delete the program file/folder yourself
    flockswitch set <key> <value>
                            change a setting, e.g.
                              set buttons wheel_mode,gesture   buttons that switch (also: middle)
                              set channels 1,2      channels to cycle through (every button;
                                                    one channel = always go there)
                              set gesture.channels 3           one button: wheel_mode / gesture / middle
                              set follow all        devices that move along (all, none, or B366,MX Keys)
                              set trigger_device auto
                              set enabled true|false
    flockswitch help

Set RUST_LOG=debug for more detail.";

/// Port used only as a lock so two copies never react to the same button press.
const INSTANCE_PORT: u16 = 47_851;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    if !args.is_empty() {
        attach_console();
    }
    // The long-running modes take the single-instance lock before touching the log, so
    // a second copy does not wipe the running copy's log file.
    let tray_mode = args.is_empty() || args[0] == "tray";
    let _lock = if tray_mode || args[0] == "run" {
        match single_instance() {
            Some(lock) => Some(lock),
            None if tray_mode => match already_running() {
                Some(lock) => Some(lock),
                None => std::process::exit(0),
            },
            None => {
                eprintln!("error: {}", i18n::texts().already_running);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    Config::migrate_old_name();
    // The settings window has no console when started from the menu: log to its own
    // file so a window that fails to open (graphics drivers) leaves a trace.
    let log_file = if tray_mode {
        Some(log_path())
    } else if args.first().map(String::as_str) == Some("settings") {
        Some(log_path().with_file_name("settings.log"))
    } else {
        None
    };
    init_logging(log_file);
    // A panic (for example in a graphics driver) is written to the log, not only to a
    // console nobody sees.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        default_hook(info);
    }));
    if let Err(e) = real_main(&args) {
        log::error!("{e:#}");
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn real_main(args: &[String]) -> Result<()> {
    let cmd = args.first().map(String::as_str).unwrap_or("tray");
    match cmd {
        "tray" => tray::run(Arc::new(Shared::new(load_config()))),
        "run" => run_headless(),
        "settings" => {
            // One settings window at a time. Opening it again knocks on the open
            // window's port (nothing is sent), which brings that window to the front.
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], INSTANCE_PORT + 1));
            let fallback = std::env::var(settings_ui::RENDERER_ENV).is_ok();
            // A fallback relaunch waits for the failed window's process to let go.
            let tries = if fallback { 30 } else { 1 };
            let lock = (0..tries).find_map(|i| {
                if i > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                std::net::TcpListener::bind(addr).ok()
            });
            let Some(lock) = lock else {
                let _ = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1));
                return Ok(());
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| settings_ui::run(lock)))
                .unwrap_or_else(|_| Err(anyhow::anyhow!("the settings window crashed")));
            match result {
                // Windows: if Direct3D could not start, try once more with OpenGL.
                Err(e) if cfg!(windows) && !fallback => {
                    log::warn!("{e:#}; trying OpenGL");
                    let mut cmd = std::process::Command::new(std::env::current_exe()?);
                    cmd.args(args).env(settings_ui::RENDERER_ENV, "glow");
                    cmd.spawn()?;
                    Ok(())
                }
                // Nothing left to try: say so instead of failing silently.
                Err(e) => {
                    let t = i18n::texts();
                    let log = log_path().with_file_name("settings.log");
                    show_message(&format!("{}\n\n{e:#}\n\n{} {}", t.settings_failed, t.details_in, log.display()));
                    Err(e)
                }
                ok => ok,
            }
        }
        "list" => list(),
        "switch" => {
            let Some(n) = args.get(1).and_then(|s| s.parse::<u8>().ok()).filter(|n| (1..=3).contains(n)) else {
                bail!("usage: flockswitch switch <1|2|3>");
            };
            let mut engine = Engine::new(Arc::new(Shared::new(Config::load_or_create()?)))?;
            engine.scan()?;
            engine.switch_to(n)
        }
        "quit" => {
            if main_instance_running() {
                ask_running_copy(&shared::quit_request_path())?;
            }
            Ok(())
        }
        "uninstall" => {
            // A running tray app owns the button: it hands it back and uninstalls itself.
            if main_instance_running() {
                ask_running_copy(&shared::uninstall_request_path())?;
            } else {
                uninstall();
            }
            println!("Settings and start-at-login removed. Delete the flockswitch program to finish.");
            Ok(())
        }
        "config" => {
            Config::load_or_create()?;
            println!("{}", Config::path().display());
            Ok(())
        }
        "set" => {
            let (Some(key), Some(value)) = (args.get(1), args.get(2)) else {
                bail!("usage: flockswitch set <key> <value>  (see 'flockswitch help')");
            };
            let mut cfg = Config::load_or_create()?;
            cfg.set(key, value)?;
            cfg.save()?;
            println!("{key} = {value}  ({})", Config::path().display());
            Ok(())
        }
        "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(())
        }
        other => bail!("unknown command '{other}'\n\n{USAGE}"),
    }
}

fn run_headless() -> Result<()> {
    let cfg = load_config();
    if !cfg.enabled {
        log::info!("disabled in {}", Config::path().display());
    }
    let shared = Arc::new(Shared::new(cfg));
    let s = shared.clone();
    ctrlc::set_handler(move || s.stop.store(true, Ordering::SeqCst))?;
    Engine::new(shared)?.run()
}

fn list() -> Result<()> {
    let cfg = Config::load_or_create()?;
    let mut engine = Engine::new(Arc::new(Shared::new(cfg.clone())))?;
    engine.scan()?;
    engine.describe();
    println!();
    for d in engine.devices().to_vec() {
        let ep = engine.endpoint(d.endpoint);
        let host = d
            .host_state(ep)?
            .map(|s| format!("channel {}/{}", s.current + 1, s.count))
            .unwrap_or_else(|| "not on this computer".into());
        // The trigger buttons this device has; "*" marks ones diverted right now, "+xy"
        // ones whose movement during a hold is diverted too (gestures).
        let buttons: Vec<String> = config::BUTTONS
            .iter()
            .filter_map(|&cid| match d.control_reporting(ep, cid) {
                Ok(Some(r)) => Some(format!(
                    "0x{cid:04X}{}{}",
                    if r.diverted { "*" } else { "" },
                    if r.raw_xy { "+xy" } else { "" }
                )),
                _ => None,
            })
            .collect();
        let trig = if buttons.is_empty() { String::new() } else { format!(", buttons {}", buttons.join(" ")) };
        let kind = d.kind.map(|k| format!("{k:?}")).unwrap_or_else(|| "?".into());
        println!("  slot {:>3}  {:<28} {:<9} {host}{trig}", d.index, d.label(), kind);
    }
    Ok(())
}

/// Loads the settings. A file that cannot be read is kept as config.invalid.toml and
/// the defaults are used, so a typo never stops the app from starting.
fn load_config() -> Config {
    match Config::load_or_create() {
        Ok(cfg) => cfg,
        Err(e) => {
            let path = Config::path();
            let backup = path.with_file_name("config.invalid.toml");
            log::warn!("{e:#}; using defaults (your file is kept as {})", backup.display());
            let _ = std::fs::rename(&path, &backup);
            let cfg = Config::default();
            let _ = cfg.save();
            cfg
        }
    }
}

/// Started while the tray app already runs. Starting it again is how people look
/// for it, so the same version opens the settings window. A different version (for
/// example an older one still set to start at login) is asked to quit, and this copy
/// takes over, so the version the user just started is the one that runs.
fn already_running() -> Option<std::net::TcpListener> {
    let running = shared::load_status_file().version;
    if running == env!("CARGO_PKG_VERSION") {
        tray::open_settings_window();
        return None;
    }
    let request = shared::quit_request_path();
    if let Some(dir) = request.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&request, b"");
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Some(lock) = single_instance() {
            return Some(lock);
        }
    }
    let _ = std::fs::remove_file(&request);
    show_message(i18n::texts().old_copy_running);
    None
}

/// A message the user sees: a dialog on Windows (no console there), stderr elsewhere.
pub fn show_message(text: &str) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};
        let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let (text, title) = (wide(text), wide("flockswitch"));
        unsafe {
            MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONWARNING);
        }
    }
    #[cfg(not(windows))]
    eprintln!("{text}");
}

/// Leaves a request file for the running tray app and waits for it to quit.
fn ask_running_copy(request: &std::path::Path) -> Result<()> {
    if let Some(dir) = request.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(request, b"")?;
    for _ in 0..150 {
        if !main_instance_running() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = std::fs::remove_file(request);
    bail!("flockswitch did not quit; quit it from the menu (or stop `flockswitch run`) and try again")
}

/// True when another copy (the tray app or `run`) holds the instance lock.
pub fn main_instance_running() -> bool {
    std::net::TcpListener::bind(("127.0.0.1", INSTANCE_PORT)).is_err()
}

/// Holds a loopback port for the life of the process; a second copy fails to bind.
fn single_instance() -> Option<std::net::TcpListener> {
    std::net::TcpListener::bind(("127.0.0.1", INSTANCE_PORT)).ok()
}

/// Removes everything flockswitch created outside its own program file: the login
/// item and the settings folder (config, remembered devices, log). The trigger
/// button is already back to normal because the engine releases it when it stops.
pub fn uninstall() {
    if let Some(al) = tray::autostart() {
        if al.is_enabled().unwrap_or(false) {
            if let Err(e) = al.disable() {
                log::warn!("could not remove start at login: {e}");
            }
        }
    }
    if let Some(dir) = Config::path().parent() {
        // The log file may still be open (Windows refuses to delete it); remove what we can.
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let _ = std::fs::remove_file(e.path());
            }
        }
        let _ = std::fs::remove_dir(dir);
        // Windows keeps our own open log file until we exit; finish the job afterwards.
        #[cfg(windows)]
        if dir.exists() {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            // raw_arg: cmd.exe does not understand Rust's \" escaping of inner quotes.
            let script = format!("/C ping -n 3 127.0.0.1 >nul & rmdir /s /q \"{}\"", dir.display());
            let _ = std::process::Command::new("cmd")
                .raw_arg(script)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    }
    log::info!("uninstalled settings and start-at-login");
}

pub fn log_path() -> PathBuf {
    Config::path().with_file_name("flockswitch.log")
}

/// The tray app has no terminal, so it logs to a file next to the config.
fn init_logging(file: Option<PathBuf>) {
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder.format_timestamp_secs();
    if let Some(path) = file {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Start a fresh log each launch so it never grows without bound; a settings
        // window relaunched with OpenGL keeps the first attempt's lines.
        let append = std::env::var(settings_ui::RENDERER_ENV).is_ok();
        let file = std::fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&path);
        if let Ok(file) = file {
            builder.target(env_logger::Target::Pipe(Box::new(file)));
        }
    }
    builder.init();
}

/// Lets `flockswitch.exe list` etc. print to the PowerShell/cmd window it was typed in,
/// even though the program is built as a windowed (no console) app.
#[cfg(windows)]
fn attach_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
