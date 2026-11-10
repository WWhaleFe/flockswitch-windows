//! Settings backups: the settings written as a TOML file into a folder the user can see
//! and copy from (Documents\flockswitch), to load again later or on another computer.
//! A backup is config.toml with a comment header, so the macOS and Linux versions read
//! it too; keys only another system has are passed over (serde ignores unknown keys).
//!
//! The Store version runs with full trust, so Documents is the real Documents folder
//! there as well (only its AppData is kept apart), and the same path works.

use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Top-level keys of a settings file, including the ones older versions wrote. A TOML
/// file with none of them is something else, not a settings file with every value left
/// at its default.
const KNOWN_KEYS: &[&str] = &[
    "enabled",
    "trigger_device",
    "follow",
    "language",
    "check_updates",
    "auto_update",
    "buttons",
    "hotkeys",
    "edges",
    "haptics",
    "locate",
    "mode",
    "channels",
    "target",
    "triggers",
    "trigger_cid",
];

/// A settings file is a few kilobytes; anything far larger is not one.
const MAX_SIZE: u64 = 1024 * 1024;

/// At most this many .toml files are kept in the folder: at this many, Save writes
/// nothing until some are renamed away or deleted in Manage.
pub const MAX_BACKUPS: usize = 10;

/// Longest name (without .toml) a backup can be renamed to, in characters.
pub const MAX_NAME: usize = 60;

/// Documents\flockswitch.
pub fn folder() -> PathBuf {
    dirs::document_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Documents")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("flockswitch")
}

/// A backup found in the folder.
#[derive(Debug, Clone)]
pub struct Backup {
    pub path: PathBuf,
    pub name: String,
    pub modified: Option<SystemTime>,
}

impl Backup {
    /// The name without ".toml", as the list shows it.
    pub fn stem(&self) -> &str {
        strip_toml(&self.name)
    }
}

fn strip_toml(name: &str) -> &str {
    let cut = name.len().saturating_sub(5);
    match name.get(cut..) {
        Some(ext) if ext.eq_ignore_ascii_case(".toml") => &name[..cut],
        _ => name,
    }
}

/// A moment in local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Stamp {
    /// Now, in this computer's time zone.
    pub fn now() -> Stamp {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::SYSTEMTIME;
            use windows_sys::Win32::System::SystemInformation::GetLocalTime;
            // Safety: a plain struct, filled in by the call.
            let mut t: SYSTEMTIME = unsafe { std::mem::zeroed() };
            unsafe { GetLocalTime(&mut t) };
            let SYSTEMTIME { wYear, wMonth, wDay, wHour, wMinute, wSecond, .. } = t;
            Stamp { year: wYear, month: wMonth as u8, day: wDay as u8, hour: wHour as u8, minute: wMinute as u8, second: wSecond as u8 }
        }
        #[cfg(not(windows))]
        {
            let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            Stamp::from_unix(secs)
        }
    }

    /// When a file was last changed, in this computer's time zone.
    pub fn local(time: SystemTime) -> Stamp {
        let secs = time.duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let utc = Stamp::from_unix(secs);
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::SYSTEMTIME;
            use windows_sys::Win32::System::Time::SystemTimeToTzSpecificLocalTime;
            let from = SYSTEMTIME {
                wYear: utc.year,
                wMonth: utc.month.into(),
                wDayOfWeek: 0,
                wDay: utc.day.into(),
                wHour: utc.hour.into(),
                wMinute: utc.minute.into(),
                wSecond: utc.second.into(),
                wMilliseconds: 0,
            };
            // Safety: plain structs; a null time zone means the current one.
            let mut t: SYSTEMTIME = unsafe { std::mem::zeroed() };
            if unsafe { SystemTimeToTzSpecificLocalTime(std::ptr::null(), &from, &mut t) } != 0 {
                let SYSTEMTIME { wYear, wMonth, wDay, wHour, wMinute, wSecond, .. } = t;
                return Stamp { year: wYear, month: wMonth as u8, day: wDay as u8, hour: wHour as u8, minute: wMinute as u8, second: wSecond as u8 };
            }
        }
        utc
    }

    /// 2026-11-10 14:03.
    pub fn short(&self) -> String {
        format!("{:04}-{:02}-{:02} {:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute)
    }

    /// A time in seconds since 1970 (UTC), as a date and time.
    #[cfg_attr(all(windows, not(test)), allow(dead_code))]
    pub fn from_unix(secs: u64) -> Stamp {
        let days = (secs / 86_400) as i64;
        let rest = secs % 86_400;
        // Howard Hinnant's civil_from_days.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as u16;
        Stamp { year, month, day, hour: (rest / 3600) as u8, minute: (rest / 60 % 60) as u8, second: (rest % 60) as u8 }
    }

    /// 20261110-140322, for file names.
    pub fn compact(&self) -> String {
        format!("{:04}{:02}{:02}-{:02}{:02}{:02}", self.year, self.month, self.day, self.hour, self.minute, self.second)
    }

    /// 2026-11-10 14:03:22.
    pub fn readable(&self) -> String {
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute, self.second)
    }
}

/// This computer's name, as it can go into a file name.
pub fn computer_name() -> String {
    let raw = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_default();
    let clean: String = raw.trim().chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    let clean = clean.trim_matches('-');
    if clean.is_empty() { "pc".into() } else { clean.into() }
}

/// flockswitch-settings-<computer>-<YYYYMMDD-HHMMSS>.toml
pub fn file_name(computer: &str, stamp: &Stamp) -> String {
    format!("flockswitch-settings-{computer}-{}.toml", stamp.compact())
}

/// "DESK-1 · 2026-11-10 09:05" for a file named as `file_name` names it.
pub fn describe(name: &str) -> Option<String> {
    let stem = name.strip_prefix("flockswitch-settings-")?.strip_suffix(".toml")?;
    let mut parts = stem.rsplitn(3, '-');
    let (time, date, computer) = (parts.next()?, parts.next()?, parts.next()?);
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(date, 8) || !digits(time, 6) || computer.is_empty() {
        return None;
    }
    Some(format!("{computer} · {}-{}-{} {}:{}", &date[..4], &date[4..6], &date[6..], &time[..2], &time[2..4]))
}

/// The settings as a backup file: a comment header, then the settings file itself.
pub fn to_text(cfg: &Config, computer: &str, stamp: &Stamp) -> Result<String> {
    let body = toml::to_string_pretty(cfg)?;
    Ok(format!(
        "# flockswitch settings backup\n\
         # app: flockswitch {} for Windows\n\
         # os: Windows\n\
         # computer: {computer}\n\
         # saved: {}\n\
         # Load it in flockswitch: Settings > General > Settings backup (Windows, macOS or Linux).\n\
         # Channels are set per computer: check them after loading on another computer.\n\n{body}",
        env!("CARGO_PKG_VERSION"),
        stamp.readable(),
    ))
}

/// What Save did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved {
    /// The new file.
    File(PathBuf),
    /// Nothing: the folder already holds `MAX_BACKUPS` files.
    Full,
}

/// Writes the settings into the folder, unless it is full.
pub fn save(cfg: &Config) -> Result<Saved> {
    save_in(&folder(), cfg, &computer_name(), &Stamp::now())
}

pub fn save_in(dir: &Path, cfg: &Config, computer: &str, stamp: &Stamp) -> Result<Saved> {
    if list_in(dir).len() >= MAX_BACKUPS {
        return Ok(Saved::Full);
    }
    std::fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    let text = to_text(cfg, computer, stamp)?;
    // Never over another file: a second save within the same second gets "-2".
    let base = file_name(computer, stamp);
    let stem = strip_toml(&base);
    let mut n = 1;
    loop {
        let name = if n == 1 { base.clone() } else { format!("{stem}-{n}.toml") };
        let path = dir.join(name);
        match std::fs::File::options().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(text.as_bytes()).with_context(|| format!("cannot write {}", path.display()))?;
                log::info!("settings saved to {}", path.display());
                return Ok(Saved::File(path));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && n < 100 => n += 1,
            Err(e) => return Err(e).with_context(|| format!("cannot write {}", path.display())),
        }
    }
}

/// Why a name cannot be a backup's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong,
    /// One of / \ : * ? " < > | or a control character.
    BadChar(char),
    /// Ends with a dot or a space, which Windows takes off.
    BadEnd,
    /// CON, PRN, AUX, NUL, COM0-9, LPT0-9 (with or without an extension): names Windows
    /// keeps for devices.
    Reserved,
    /// Another file in the folder has it.
    Taken,
}

/// The file name (with .toml) a backup gets when renamed to `input`; ".toml" typed at the
/// end is allowed, and spaces around the name are taken off.
pub fn check_name(input: &str) -> Result<String, NameError> {
    let stem = strip_toml(input.trim()).trim();
    if stem.is_empty() {
        return Err(NameError::Empty);
    }
    if stem.chars().count() > MAX_NAME {
        return Err(NameError::TooLong);
    }
    if let Some(c) = stem.chars().find(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()) {
        return Err(NameError::BadChar(c));
    }
    if stem.ends_with('.') || stem.ends_with(' ') {
        return Err(NameError::BadEnd);
    }
    // "CON" and "con.backup" alike: Windows looks at the part before the first dot.
    let device = stem.split('.').next().unwrap_or(stem).trim_end().to_ascii_uppercase();
    let numbered = |prefix: &str| {
        device.strip_prefix(prefix).is_some_and(|n| matches!(n, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
    };
    if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&device.as_str()) || numbered("COM") || numbered("LPT") {
        return Err(NameError::Reserved);
    }
    Ok(format!("{stem}.toml"))
}

/// Gives a backup a new name (`input`, see `check_name`) in the same folder, never over
/// another file. Returns the new path.
pub fn rename(path: &Path, input: &str) -> Result<PathBuf, RenameError> {
    let name = check_name(input).map_err(RenameError::Name)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let target = dir.join(&name);
    let current = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if current == name {
        return Ok(target);
    }
    // Only the case changes: the same file on Windows, so not "taken".
    let same_file = current.eq_ignore_ascii_case(&name);
    if !same_file && (target.exists() || list_in(dir).iter().any(|b| b.name.eq_ignore_ascii_case(&name))) {
        return Err(RenameError::Name(NameError::Taken));
    }
    std::fs::rename(path, &target).map_err(|e| RenameError::Io(e.to_string()))?;
    log::info!("settings backup {} renamed to {name}", path.display());
    Ok(target)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    Name(NameError),
    /// The system's own message.
    Io(String),
}

/// What Delete did with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deleted {
    RecycleBin,
    /// The Recycle Bin could not take it: deleted for good.
    Permanently,
}

/// Moves a backup to the Recycle Bin; where that does not work, deletes it.
pub fn delete(path: &Path) -> Result<Deleted> {
    #[cfg(windows)]
    match recycle(path) {
        Recycle::Done => {
            log::info!("settings backup {} moved to the Recycle Bin", path.display());
            return Ok(Deleted::RecycleBin);
        }
        Recycle::Declined => bail!("not deleted"),
        Recycle::Failed => {}
    }
    std::fs::remove_file(path).with_context(|| format!("cannot delete {}", path.display()))?;
    log::info!("settings backup {} deleted", path.display());
    Ok(Deleted::Permanently)
}

#[cfg(windows)]
enum Recycle {
    Done,
    /// The shell asked before deleting for good, and was told no.
    Declined,
    /// The shell could not do it; deleting the file directly may still work.
    Failed,
}

/// The shell's delete with undo, as Explorer does it. Where the file cannot be recycled
/// (no Recycle Bin on that drive, or it is too large for it) the shell asks before
/// deleting it for good.
#[cfg(windows)]
fn recycle(path: &Path) -> Recycle {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FOF_WANTNUKEWARNING, FO_DELETE,
        SHFILEOPSTRUCTW,
    };
    let Ok(full) = std::fs::canonicalize(path) else { return Recycle::Failed };
    // The shell wants a plain path (no \\?\ prefix), ended by two zeros.
    let text = full.to_string_lossy();
    let plain = text.strip_prefix(r"\\?\").unwrap_or(&text);
    let from: Vec<u16> = std::ffi::OsStr::new(plain).encode_wide().chain([0, 0]).collect();
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: from.as_ptr(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT | FOF_WANTNUKEWARNING) as u16,
        ..Default::default()
    };
    // Safety: `from` outlives the call and is double-zero terminated as required.
    let result = unsafe { SHFileOperationW(&mut op) };
    if op.fAnyOperationsAborted != 0 {
        return Recycle::Declined;
    }
    if result != 0 || path.exists() {
        log::info!("recycling {plain}: shell result {result:#x}");
        return Recycle::Failed;
    }
    Recycle::Done
}

/// The .toml files in the folder, newest first.
pub fn list() -> Vec<Backup> {
    list_in(&folder())
}

pub fn list_in(dir: &Path) -> Vec<Backup> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<Backup> = entries
        .flatten()
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("toml")))
        .map(|e| Backup {
            name: e.file_name().to_string_lossy().into_owned(),
            modified: e.metadata().and_then(|m| m.modified()).ok(),
            path: e.path(),
        })
        .collect();
    found.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| b.name.cmp(&a.name)));
    found
}

/// Reads a backup through the same checks as config.toml. Keys this version does not
/// know are passed over; a file that is not TOML, is not a flockswitch settings file,
/// or has a value this version cannot use is refused.
pub fn read(text: &str) -> Result<Config> {
    let table: toml::Table = text.parse().context("this is not a settings file (not TOML)")?;
    if !table.keys().any(|k| KNOWN_KEYS.contains(&k.as_str())) {
        bail!("this is not a flockswitch settings file");
    }
    let mut cfg = Config::parse(text)?;
    for (slot, key) in fit_shortcuts(&mut cfg) {
        log::info!("loading settings: shortcut '{key}' (slot {slot}) cannot be used on Windows; left empty");
    }
    cfg.validate()?;
    Ok(cfg)
}

/// Empties the shortcuts Windows cannot use, so a file from another system still loads:
/// ones its parser does not read, ones with the Windows key (⌘ on a Mac), and a second
/// use of the same combination. Returns the slots emptied, with what they held.
fn fit_shortcuts(cfg: &mut Config) -> Vec<(u8, String)> {
    let mut seen = Vec::new();
    let mut emptied = Vec::new();
    for slot in [1, 2, 3, crate::hotkey::LOCATE] {
        let key = cfg.shortcut(slot).to_string();
        if key.is_empty() {
            continue;
        }
        let windows_key = key.split('+').any(|k| ["cmd", "command", "super", "meta", "win"].contains(&k.trim().to_lowercase().as_str()));
        match crate::hotkey::parse(&key) {
            Ok(parsed) if !windows_key && !seen.contains(&parsed) => seen.push(parsed),
            _ => {
                cfg.set_shortcut(slot, "");
                emptied.push((slot, key));
            }
        }
    }
    cfg.hotkeys.keys.truncate(3);
    emptied
}

/// Reads a backup file.
pub fn load(path: &Path) -> Result<Config> {
    let size = std::fs::metadata(path).with_context(|| format!("cannot read {}", path.display()))?.len();
    if size > MAX_SIZE {
        bail!("this file is too large to be a settings file");
    }
    let text = std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    read(&text)
}

/// Opens the folder in Explorer, creating it first so it can be opened before the first
/// backup (and files from another computer copied in).
pub fn open_folder() -> Result<()> {
    let dir = folder();
    std::fs::create_dir_all(&dir).with_context(|| format!("cannot create {}", dir.display()))?;
    std::process::Command::new("explorer").arg(&dir).spawn().with_context(|| format!("cannot open {}", dir.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ShakeKey;

    fn same(a: &Config, b: &Config) -> bool {
        toml::to_string_pretty(a).unwrap() == toml::to_string_pretty(b).unwrap()
    }

    #[test]
    fn save_and_load_round_trip() {
        let mut c = Config::default();
        c.set("edges", "true").unwrap();
        c.set("edge.right", "2").unwrap();
        c.set("locate", "true").unwrap();
        c.set("locate.shake", "true").unwrap();
        c.set("locate.shake_time", "1500").unwrap();
        c.set("locate.shake_key", "ctrl").unwrap();
        c.set("hotkey.3", "ctrl+shift+f5").unwrap();
        c.set("gesture.channels", "3").unwrap();
        c.set("language", "en").unwrap();
        let stamp = Stamp { year: 2026, month: 11, day: 10, hour: 9, minute: 5, second: 3 };
        let text = to_text(&c, "DESK-1", &stamp).unwrap();
        assert!(text.starts_with("# flockswitch settings backup\n"), "{text}");
        assert!(text.contains(concat!("# app: flockswitch ", env!("CARGO_PKG_VERSION"), " for Windows")));
        assert!(text.contains("# os: Windows") && text.contains("# saved: 2026-11-10 09:05:03"));
        let back = read(&text).unwrap();
        assert!(same(&back, &c));
        assert_eq!(back.locate.shake_key, ShakeKey::Ctrl);
        assert_eq!(back.locate.shake_time_ms, 1500);

        // Through a file in a folder, listed newest first.
        let dir = std::env::temp_dir().join(format!("flockswitch-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let older = dir.join(file_name("DESK-1", &Stamp { hour: 8, ..stamp }));
        let newer = dir.join(file_name("DESK-1", &stamp));
        std::fs::write(&older, &text).unwrap();
        std::fs::write(&newer, &text).unwrap();
        std::fs::write(dir.join("notes.txt"), "not a backup").unwrap();
        let old_time = SystemTime::now() - std::time::Duration::from_secs(3600);
        std::fs::File::options().write(true).open(&older).unwrap().set_modified(old_time).unwrap();
        let listed = list_in(&dir);
        assert_eq!(listed.iter().map(|b| b.path.clone()).collect::<Vec<_>>(), vec![newer.clone(), older]);
        assert!(same(&load(&newer).unwrap(), &c));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn keys_from_another_system_are_passed_over() {
        let text = "# from a Mac\nenabled = false\nmac_only = 3\n\n[locate]\non = true\nshake_key = \"cmd\"\nmenu_bar_icon = true\n\n[dock]\nshow = false\n";
        let c = read(text).unwrap();
        assert!(!c.enabled && c.locate.on);
        assert_eq!(c.locate.shake_key, ShakeKey::None, "a key Windows has not got reads as none");
        assert_eq!(c.locate.shake_time_ms, 1000);
        // "option" is the Mac's name for Alt.
        assert_eq!(read("[locate]\nshake_key = \"option\"\n").unwrap().locate.shake_key, ShakeKey::Alt);
    }

    #[test]
    fn a_file_from_a_mac_loads() {
        // As the macOS version writes it: ⌘ shortcuts, a ⌘ shake key, keys of its own.
        let text = "# flockswitch settings backup\n# os: macOS\nenabled = true\nlanguage = \"ko\"\nlaunch_at_login = true\n\n\
                    [hotkeys]\non = true\nkeys = [\"cmd+alt+1\", \"ctrl+option+2\", \"super+3\"]\n\n\
                    [locate]\non = true\non_shake = true\nshake_time_ms = 1500\nshake_key = \"cmd\"\nhotkey = \"command+shift+l\"\n\n\
                    [edges]\non = true\nright = 2\nmenu_bar_margin = 24\n";
        let c = read(text).unwrap();
        assert_eq!(c.language, "ko");
        assert_eq!(c.hotkeys.keys, vec!["".to_string(), "ctrl+option+2".to_string(), "".to_string()]);
        assert_eq!(c.locate.hotkey, "");
        assert_eq!((c.locate.shake_key, c.locate.shake_time_ms), (ShakeKey::None, 1500));
        assert_eq!(c.edges.right, 2);
        for name in ["command", "super", "meta", "win", "something-new"] {
            let c = read(&format!("[locate]\nshake_key = \"{name}\"\n")).unwrap();
            assert_eq!(c.locate.shake_key, ShakeKey::None, "{name}");
        }
        // The same combination twice, or one Windows cannot read: the later one is emptied.
        let c = read("[hotkeys]\nkeys = [\"ctrl+alt+1\", \"ctrl+alt+1\", \"ctrl+alt+\"]\n").unwrap();
        assert_eq!(c.hotkeys.keys, vec!["ctrl+alt+1".to_string(), String::new(), String::new()]);
    }

    #[test]
    fn garbage_is_refused() {
        for bad in [
            "",
            "hello world",
            "\u{0}\u{1}binary",
            "[package]\nname = \"something\"\nversion = \"1.0.0\"\n",
            "enabled = \"yes\"",
            "[edges]\ndwell_ms = 1\n",
            "[edges]\nright = 9\n",
            "[edges]\nanim_style = \"sparkle\"\n",
        ] {
            assert!(read(bad).is_err(), "{bad:?} was accepted");
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("flockswitch-backup-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn at_most_ten_are_saved() {
        let dir = scratch("limit");
        let cfg = Config::default();
        let stamp = Stamp { year: 2026, month: 11, day: 10, hour: 9, minute: 5, second: 3 };
        std::fs::write(dir.join("notes.txt"), "not a backup").unwrap();
        let mut saved = Vec::new();
        for _ in 0..MAX_BACKUPS {
            // The same second every time: the files still never overwrite each other.
            match save_in(&dir, &cfg, "DESK-1", &stamp).unwrap() {
                Saved::File(path) => saved.push(path),
                Saved::Full => panic!("full after {}", saved.len()),
            }
        }
        assert_eq!(list_in(&dir).len(), MAX_BACKUPS);
        assert!(saved[1].ends_with("flockswitch-settings-DESK-1-20261110-090503-2.toml"), "{:?}", saved[1]);
        assert_eq!(save_in(&dir, &cfg, "DESK-1", &stamp).unwrap(), Saved::Full);
        assert_eq!(list_in(&dir).len(), MAX_BACKUPS, "nothing written when full");
        // One deleted: room for one more.
        delete(&saved[0]).unwrap();
        assert!(!saved[0].exists());
        assert_eq!(list_in(&dir).len(), MAX_BACKUPS - 1);
        assert!(matches!(save_in(&dir, &cfg, "DESK-1", &stamp).unwrap(), Saved::File(_)));
        assert_eq!(save_in(&dir, &cfg, "DESK-1", &stamp).unwrap(), Saved::Full);
        assert!(delete(&dir.join("gone.toml")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn names_a_backup_can_be_given() {
        assert_eq!(check_name("  my desk  "), Ok("my desk.toml".into()));
        assert_eq!(check_name("work.toml"), Ok("work.toml".into()));
        assert_eq!(check_name("work.TOML"), Ok("work.toml".into()));
        assert_eq!(check_name("집 컴퓨터 v2.1"), Ok("집 컴퓨터 v2.1.toml".into()));
        assert_eq!(check_name(""), Err(NameError::Empty));
        assert_eq!(check_name("   "), Err(NameError::Empty));
        assert_eq!(check_name(".toml"), Err(NameError::Empty));
        assert_eq!(check_name(&"가".repeat(MAX_NAME)), Ok(format!("{}.toml", "가".repeat(MAX_NAME))));
        assert_eq!(check_name(&"a".repeat(MAX_NAME + 1)), Err(NameError::TooLong));
        for c in ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\t'] {
            assert_eq!(check_name(&format!("a{c}b")), Err(NameError::BadChar(c)), "{c:?}");
        }
        assert_eq!(check_name("trailing."), Err(NameError::BadEnd));
        for reserved in ["CON", "con", "Nul", "aux", "PRN", "COM1", "com9", "LPT3", "lpt0", "COM¹", "nul.backup", "CON .x", "conout$"] {
            assert_eq!(check_name(reserved), Err(NameError::Reserved), "{reserved}");
        }
        for fine in ["CONSOLE", "COM10", "LPT", "nullable", "my con", "COMA"] {
            assert!(check_name(fine).is_ok(), "{fine}");
        }
    }

    #[test]
    fn renaming_keeps_toml_and_never_overwrites() {
        let dir = scratch("rename");
        let a = dir.join("a.toml");
        let b = dir.join("b.toml");
        std::fs::write(&a, "enabled = true\n").unwrap();
        std::fs::write(&b, "enabled = false\n").unwrap();
        assert_eq!(rename(&a, "b"), Err(RenameError::Name(NameError::Taken)));
        assert_eq!(rename(&a, "B.toml"), Err(RenameError::Name(NameError::Taken)), "another file, whatever the case");
        assert_eq!(rename(&a, "NUL"), Err(RenameError::Name(NameError::Reserved)));
        assert!(a.exists() && std::fs::read_to_string(&b).unwrap() == "enabled = false\n");
        let moved = rename(&a, "home office").unwrap();
        assert_eq!(moved, dir.join("home office.toml"));
        assert!(moved.exists() && !a.exists());
        assert_eq!(list_in(&dir).iter().map(|f| f.stem().to_string()).collect::<Vec<_>>().len(), 2);
        // The same name again, or only its case changed: fine.
        assert_eq!(rename(&moved, "home office"), Ok(moved.clone()));
        let cased = rename(&moved, "Home Office").unwrap();
        assert!(cased.exists());
        assert!(list_in(&dir).iter().any(|f| f.stem() == "Home Office"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stems_and_dates() {
        let b = Backup { path: PathBuf::from("x/work.TOML"), name: "work.TOML".into(), modified: None };
        assert_eq!(b.stem(), "work");
        assert_eq!(strip_toml("toml"), "toml");
        assert_eq!(strip_toml("a.toml.toml"), "a.toml");
        assert_eq!(strip_toml("한.toml"), "한");
        let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_794_301_503);
        let local = Stamp::local(t);
        assert_eq!((local.year, local.month), (2026, 11));
        assert_eq!(Stamp::from_unix(1_794_301_503).short(), "2026-11-10 09:05");
    }

    #[test]
    fn names_and_times() {
        let stamp = Stamp::from_unix(1_794_301_503);
        assert_eq!(stamp, Stamp { year: 2026, month: 11, day: 10, hour: 9, minute: 5, second: 3 });
        assert_eq!(file_name("DESK-1", &stamp), "flockswitch-settings-DESK-1-20261110-090503.toml");
        assert_eq!(Stamp::from_unix(951_868_799).readable(), "2000-02-29 23:59:59");
        assert_eq!(Stamp::from_unix(0).compact(), "19700101-000000");
        assert_eq!(describe("flockswitch-settings-DESK-1-20261110-090503.toml").as_deref(), Some("DESK-1 · 2026-11-10 09:05"));
        assert_eq!(describe("my-settings.toml"), None);
        assert_eq!(describe("flockswitch-settings-20261110-090503.toml"), None);
        let name = computer_name();
        assert!(!name.is_empty() && !name.contains(['/', '\\', ':', ' ']), "{name}");
    }
}
