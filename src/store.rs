//! The Microsoft Store version: the same program, run from its MSIX package. Windows
//! then keeps the app's files apart and removes them on uninstall, and start-at-login
//! is the package's startup task instead of the Run registry key.

/// True when this process runs from an installed MSIX package (the Store version).
pub fn packaged() -> bool {
    use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    let mut len = 0u32;
    // With no buffer this only reports whether there is a package name to return.
    let result = unsafe { GetCurrentPackageFullName(&mut len, std::ptr::null_mut()) };
    result != APPMODEL_ERROR_NO_PACKAGE
}

/// Opens Settings → Apps → Startup, where the Store version's start-at-login is switched
/// (a packaged app may turn its startup task off, but only the user can turn it back on).
pub fn open_startup_settings() {
    crate::update::open_url("ms-settings:startupapps");
}

/// Asks Windows to start flockswitch again after closing it to install a Store update (a
/// package update closes the running app). Started the same way as at login, so it comes
/// back quietly. Not after a crash or a hang. (The setup program quits and restarts it.)
pub fn restart_after_update() {
    use windows_sys::Win32::System::Recovery::{RegisterApplicationRestart, RESTART_NO_CRASH, RESTART_NO_HANG};
    // Only the arguments: Windows adds the program itself.
    let args = crate::wide(&format!("tray {}", crate::tray::LOGIN_ARG));
    let result = unsafe { RegisterApplicationRestart(args.as_ptr(), RESTART_NO_CRASH | RESTART_NO_HANG) };
    if result != 0 {
        log::warn!("could not register the restart after an update ({result:#x})");
    }
}
