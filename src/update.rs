//! New versions, looked for at start and once a day and installed by the app itself
//! when the settings say so (`check_updates`, `auto_update`).
//!
//! - The Microsoft Store version asks the Store (Windows.Services.Store) for a newer
//!   package and lets the Store install it quietly. Installing closes flockswitch, and
//!   Windows starts it again (see `store::restart_after_update`).
//! - The setup and portable versions read the latest release from GitHub's public API
//!   with Windows' own curl.exe (no account, nothing about the user is sent), https
//!   only. The file must match the SHA256SUMS.txt published with the release. Then the
//!   setup program runs silently (it asks this copy to quit and starts the new one), or
//!   the portable flockswitch.exe is swapped and the new one started, which takes over
//!   from this copy (see `already_running` in main.rs).

use crate::shared::Shared;
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

const RELEASES_API: &str = "https://api.github.com/repos/WWhaleFe/flockswitch-windows/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/WWhaleFe/flockswitch-windows/releases/latest";
/// The app's page in the Microsoft Store app.
pub const STORE_PAGE: &str = "ms-windows-store://pdp/?ProductId=9NN9ZLHLGSKN";
/// How often the running app looks for a new version.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// The first look waits until starting up (and a login's busy minute) is over.
const FIRST_CHECK: Duration = Duration::from_secs(60);
/// The checksum list published with every release (`sha256sum` output).
const SUMS: &str = "SHA256SUMS.txt";
/// The portable program's previous file, left next to it after a swap (a running
/// program can be renamed but not deleted) and removed at the next start.
const OLD_EXE: &str = "flockswitch.old.exe";

/// Where the app is with a new version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    /// A newer version, e.g. "1.7.0".
    Available(String),
    Installing(String),
    /// The check failed.
    Failed,
    /// Installing this version failed.
    InstallFailed(String),
}

/// The page that shows the new version: the Store's, or the GitHub release.
pub fn page() -> &'static str {
    if crate::store::packaged() {
        STORE_PAGE
    } else {
        RELEASES_PAGE
    }
}

/// A newer version than this one, if there is one.
pub fn find() -> Result<Option<String>> {
    if crate::store::packaged() {
        store::newer()
    } else {
        let r = github::latest()?;
        Ok(is_newer(&r.version, env!("CARGO_PKG_VERSION")).then_some(r.version))
    }
}

/// Installs the newest version and has it started. Returns once that is under way:
/// this copy is then asked to quit, or closed by Windows.
pub fn install() -> Result<()> {
    if crate::store::packaged() {
        store::install()
    } else {
        github::install()
    }
}

/// Looks for a new version and records the outcome (the tray menu shows it).
pub fn check_into(shared: &Shared) -> UpdateState {
    shared.set_status(|s| s.update = UpdateState::Checking);
    let next = match find() {
        Ok(Some(v)) => {
            log::info!("update check: {v} is available");
            UpdateState::Available(v)
        }
        Ok(None) => {
            log::info!("update check: up to date");
            UpdateState::UpToDate
        }
        Err(e) => {
            log::info!("update check failed: {e:#}");
            UpdateState::Failed
        }
    };
    shared.set_status(|s| s.update = next.clone());
    next
}

/// Installs `version`, recording how it went.
pub fn install_into(shared: &Shared, version: String) {
    shared.set_status(|s| s.update = UpdateState::Installing(version.clone()));
    match install() {
        Ok(()) => log::info!("update {version}: installing; flockswitch starts again"),
        Err(e) => {
            log::warn!("update {version} not installed: {e:#}");
            shared.set_status(|s| s.update = UpdateState::InstallFailed(version));
        }
    }
}

/// The tray app's background look for new versions: a minute after start, then once a
/// day, while the settings ask for it; what it finds is installed when they say so.
pub fn spawn_periodic(shared: Arc<Shared>) {
    std::thread::spawn(move || {
        let started = Instant::now();
        let mut last: Option<Instant> = None;
        while !shared.stopped() {
            std::thread::sleep(Duration::from_secs(1));
            let due = match last {
                None => started.elapsed() >= FIRST_CHECK,
                Some(at) => at.elapsed() >= CHECK_INTERVAL,
            };
            let cfg = shared.config();
            if !due || !cfg.check_updates || matches!(shared.status().update, UpdateState::Installing(_)) {
                continue;
            }
            last = Some(Instant::now());
            if let UpdateState::Available(v) = check_into(&shared) {
                if cfg.auto_update {
                    install_into(&shared, v);
                }
            }
        }
    });
}

/// Removes what an earlier update left behind: the previous portable program and the
/// downloaded files.
pub fn remove_leftovers() {
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        let _ = std::fs::remove_file(dir.join(OLD_EXE));
    }
    let _ = std::fs::remove_dir_all(download_dir());
}

fn download_dir() -> PathBuf {
    std::env::temp_dir().join("flockswitch-update")
}

/// "1.2.10" > "1.2.9"; parts that are not numbers count as 0.
pub fn is_newer(a: &str, b: &str) -> bool {
    let parts = |s: &str| -> Vec<u64> {
        s.split('.').map(|p| p.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0)).collect()
    };
    let (x, y) = (parts(a), parts(b));
    for i in 0..x.len().max(y.len()) {
        let (l, r) = (x.get(i).copied().unwrap_or(0), y.get(i).copied().unwrap_or(0));
        if l != r {
            return l > r;
        }
    }
    false
}

/// Opens a web link (or the Store app's) with the system's default handler.
pub fn open_url(url: &str) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        let (verb, target) = (wide("open"), wide(url));
        unsafe {
            ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
        }
    }
    #[cfg(not(windows))]
    log::info!("open {url}");
}

/// A console program run without flashing a console window.
fn hidden(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}

/// The release on GitHub and the file for this kind of install.
mod github {
    use super::*;

    pub struct Release {
        /// "1.7.0" (without the leading v).
        pub version: String,
        /// (file name, download URL, SHA256SUMS.txt URL); None when either is missing.
        pub asset: Option<(String, String, String)>,
    }

    #[derive(Deserialize)]
    struct GhRelease {
        tag_name: String,
        #[serde(default)]
        assets: Vec<GhAsset>,
    }

    #[derive(Deserialize)]
    struct GhAsset {
        name: String,
        browser_download_url: String,
    }

    /// Windows' curl.exe (Windows 10 1803 and later), quiet, following redirects,
    /// https only (redirects too).
    fn curl() -> Command {
        let mut c = hidden("curl.exe");
        c.args(["--silent", "--show-error", "--fail", "--location", "--proto", "=https", "--proto-redir", "=https"]);
        c.arg("--user-agent").arg(concat!("flockswitch/", env!("CARGO_PKG_VERSION")));
        c
    }

    fn run(mut c: Command) -> Result<Vec<u8>> {
        let out = c.output().context("cannot run curl.exe")?;
        if !out.status.success() {
            bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
        }
        Ok(out.stdout)
    }

    /// True when this copy was put in place by the setup program (its uninstaller sits
    /// next to it), so an update comes as the setup program too.
    fn installed() -> bool {
        std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("unins000.exe").exists())).unwrap_or(false)
    }

    pub fn latest() -> Result<Release> {
        let mut c = curl();
        c.args(["--max-time", "15", "--header", "Accept: application/vnd.github+json", RELEASES_API]);
        let r: GhRelease = serde_json::from_slice(&run(c)?).context("unexpected reply from GitHub")?;
        let version = r.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
        let names: Vec<&str> = r.assets.iter().map(|a| a.name.as_str()).collect();
        let url = |name: &str| r.assets.iter().find(|a| a.name == name).map(|a| a.browser_download_url.clone());
        let asset = pick_asset(&names, installed()).and_then(|name| Some((name.to_string(), url(name)?, url(SUMS)?)));
        Ok(Release { version, asset })
    }

    /// The setup program, or the portable zip.
    pub(super) fn pick_asset<'a>(names: &[&'a str], installed: bool) -> Option<&'a str> {
        names.iter().copied().find(|n| {
            if installed {
                n.starts_with("flockswitch-setup-") && n.ends_with(".exe")
            } else {
                n.contains("windows-portable") && n.ends_with(".zip")
            }
        })
    }

    pub fn install() -> Result<()> {
        let r = latest()?;
        if !is_newer(&r.version, env!("CARGO_PKG_VERSION")) {
            bail!("{} is not newer than this version", r.version);
        }
        let (name, url, sums) = r.asset.ok_or_else(|| anyhow!("the release {} has no file for this install, or no {SUMS}", r.version))?;
        let file = download(&name, &url, &sums)?;
        if installed() {
            // The setup program asks this copy to quit, replaces the files and starts the
            // new version quietly (see flockswitch.iss).
            Command::new(&file)
                .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/SP-"])
                .spawn()
                .with_context(|| format!("cannot start {}", file.display()))?;
            Ok(())
        } else {
            swap_portable(&file)
        }
    }

    /// Downloads a release file into the update folder; a file whose SHA-256 is not the
    /// one in the release's checksum list is deleted.
    fn download(name: &str, url: &str, sums_url: &str) -> Result<PathBuf> {
        let mut c = curl();
        c.args(["--max-time", "15", sums_url]);
        let sums = run(c)?;
        let want = expected_sha256(&String::from_utf8_lossy(&sums), name).ok_or_else(|| anyhow!("{name} is not in {SUMS}"))?;
        let dir = download_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(name);
        let mut c = curl();
        c.args(["--max-time", "600", "--output"]).arg(&path).arg(url);
        if let Err(e) = run(c) {
            let _ = std::fs::remove_file(&path);
            return Err(e);
        }
        match sha256(&path) {
            Ok(have) if have == want => Ok(path),
            Ok(have) => {
                let _ = std::fs::remove_file(&path);
                bail!("{name} does not match {SUMS} (got {have}, expected {want})")
            }
            Err(e) => {
                let _ = std::fs::remove_file(&path);
                Err(e)
            }
        }
    }

    /// Puts the new flockswitch.exe from the portable zip in place of this one and starts
    /// it; it asks this copy to quit and takes over. The folder's other files (readme,
    /// licence) are refreshed as well.
    fn swap_portable(zip: &Path) -> Result<()> {
        let exe = std::env::current_exe()?;
        let dir = exe.parent().ok_or_else(|| anyhow!("no folder for {}", exe.display()))?.to_path_buf();
        let unpacked = download_dir().join("unpacked");
        let _ = std::fs::remove_dir_all(&unpacked);
        std::fs::create_dir_all(&unpacked)?;
        // Windows' tar.exe (bsdtar) reads zip files.
        let out = hidden("tar.exe").arg("-xf").arg(zip).arg("-C").arg(&unpacked).output().context("cannot run tar.exe")?;
        if !out.status.success() {
            bail!("cannot unpack {}: {}", zip.display(), String::from_utf8_lossy(&out.stderr).trim());
        }
        let from = unpacked.join("flockswitch");
        let new_exe = from.join("flockswitch.exe");
        if !new_exe.is_file() {
            bail!("{} has no flockswitch\\flockswitch.exe", zip.display());
        }
        let old = dir.join(OLD_EXE);
        let _ = std::fs::remove_file(&old);
        std::fs::rename(&exe, &old).with_context(|| format!("cannot move {} aside", exe.display()))?;
        if let Err(e) = std::fs::copy(&new_exe, &exe) {
            let _ = std::fs::rename(&old, &exe);
            return Err(anyhow!(e).context(format!("cannot write {}", exe.display())));
        }
        for entry in std::fs::read_dir(&from)?.flatten() {
            let name = entry.file_name();
            if name != "flockswitch.exe" && entry.path().is_file() {
                let _ = std::fs::copy(entry.path(), dir.join(&name));
            }
        }
        Command::new(&exe)
            .args(["tray", crate::tray::LOGIN_ARG])
            .spawn()
            .with_context(|| format!("cannot start {}", exe.display()))?;
        Ok(())
    }

    /// SHA-256 of a file, lowercase hex, from Windows' certutil (older versions print
    /// the hash with spaces).
    fn sha256(path: &Path) -> Result<String> {
        let out = hidden("certutil.exe").arg("-hashfile").arg(path).arg("SHA256").output().context("cannot run certutil")?;
        if !out.status.success() {
            bail!("certutil: {}", String::from_utf8_lossy(&out.stdout).trim());
        }
        certutil_hash(&String::from_utf8_lossy(&out.stdout)).ok_or_else(|| anyhow!("unexpected certutil output"))
    }

    pub(super) fn certutil_hash(text: &str) -> Option<String> {
        text.lines().map(|l| l.replace(' ', "")).find(|l| l.len() == 64 && l.bytes().all(|b| b.is_ascii_hexdigit())).map(|l| l.to_ascii_lowercase())
    }

    /// The hash listed for `name` in `sha256sum` output ("<hex>  <name>", or "*<name>"
    /// in binary mode), lowercased.
    pub(super) fn expected_sha256(sums: &str, name: &str) -> Option<String> {
        sums.lines().find_map(|line| {
            let (hash, file) = line.trim().split_once(char::is_whitespace)?;
            let ok = file.trim_start().trim_start_matches('*') == name && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit());
            ok.then(|| hash.to_ascii_lowercase())
        })
    }
}

/// The Store version's updates, through the Store.
#[cfg(windows)]
mod store {
    use anyhow::{bail, Result};
    use windows::Services::Store::{StoreContext, StorePackageUpdateState};

    fn context() -> Result<StoreContext> {
        // WinRT wants the thread in an apartment; one that already is stays as it is.
        let _ = unsafe { windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED) };
        Ok(StoreContext::GetDefault()?)
    }

    pub fn newer() -> Result<Option<String>> {
        let updates = context()?.GetAppAndOptionalStorePackageUpdatesAsync()?.join()?;
        let mut newest = None;
        for u in &updates {
            let v = u.Package()?.Id()?.Version()?;
            newest = Some(format!("{}.{}.{}", v.Major, v.Minor, v.Build));
        }
        Ok(newest)
    }

    pub fn install() -> Result<()> {
        let ctx = context()?;
        let updates = ctx.GetAppAndOptionalStorePackageUpdatesAsync()?.join()?;
        if updates.Size()? == 0 {
            bail!("the Store offers no update");
        }
        if !ctx.CanSilentlyDownloadStorePackageUpdates()? {
            bail!("the Store cannot update quietly now (for example on a metered connection)");
        }
        // Installing closes flockswitch; Windows starts it again.
        let result = ctx.TrySilentDownloadAndInstallStorePackageUpdatesAsync(&updates)?.join()?;
        match result.OverallState()? {
            StorePackageUpdateState::Completed => Ok(()),
            state => bail!("the Store update ended as {state:?}"),
        }
    }
}

#[cfg(not(windows))]
mod store {
    use anyhow::{bail, Result};

    pub fn newer() -> Result<Option<String>> {
        bail!("no Microsoft Store here")
    }

    pub fn install() -> Result<()> {
        bail!("no Microsoft Store here")
    }
}

#[cfg(test)]
mod tests {
    use super::github::{certutil_hash, expected_sha256, pick_asset};
    use super::*;

    #[test]
    fn versions() {
        assert!(is_newer("1.6.0", "1.5.0"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(!is_newer("1.6.0", "1.6.0"));
        assert!(!is_newer("1.5.9", "1.6.0"));
        assert!(!is_newer("1.6", "1.6.0"));
    }

    #[test]
    fn assets() {
        let names = ["flockswitch-setup-1.6.0.exe", "flockswitch-windows-portable-1.6.0.zip", "SHA256SUMS.txt"];
        assert_eq!(pick_asset(&names, true), Some("flockswitch-setup-1.6.0.exe"));
        assert_eq!(pick_asset(&names, false), Some("flockswitch-windows-portable-1.6.0.zip"));
        assert_eq!(pick_asset(&["SHA256SUMS.txt"], false), None);
    }

    #[test]
    fn checksums() {
        let h = "a".repeat(64);
        let sums = format!("{}  flockswitch-setup-1.6.0.exe\n{h} *other.zip\n", "B".repeat(64));
        assert_eq!(expected_sha256(&sums, "flockswitch-setup-1.6.0.exe"), Some("b".repeat(64)));
        assert_eq!(expected_sha256(&sums, "other.zip"), Some(h));
        assert_eq!(expected_sha256(&sums, "missing.zip"), None);
    }

    #[test]
    fn certutil_output() {
        let hex = "0123456789abcdef".repeat(4);
        let spaced: String = hex.chars().collect::<Vec<_>>().chunks(2).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(" ");
        for text in [format!("SHA256 hash of x:\r\n{hex}\r\nCertUtil: done.\r\n"), format!("SHA256 해시 x:\n{}\nCertUtil: 완료\n", spaced.to_uppercase())] {
            assert_eq!(certutil_hash(&text), Some(hex.clone()));
        }
        assert_eq!(certutil_hash("CertUtil: error"), None);
    }
}
