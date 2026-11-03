//! The Microsoft Store version: the same program, run from its MSIX package. Windows
//! then keeps the app's files apart and removes them on uninstall, and start-at-login
//! is the package's startup task instead of the Run registry key.

/// True when this process runs from an installed MSIX package (the Store version).
pub fn packaged() -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
        use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
        let mut len = 0u32;
        // With no buffer this only reports whether there is a package name to return.
        let result = unsafe { GetCurrentPackageFullName(&mut len, std::ptr::null_mut()) };
        result != APPMODEL_ERROR_NO_PACKAGE
    }
    #[cfg(not(windows))]
    false
}

/// Opens Settings → Apps → Startup, where the Store version's start-at-login is switched
/// (a packaged app may turn its startup task off, but only the user can turn it back on).
pub fn open_startup_settings() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        let (verb, uri) = (wide("open"), wide("ms-settings:startupapps"));
        unsafe {
            ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), uri.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
        }
    }
}

/// Asks Windows to start flockswitch again after closing it to install a Store update (a
/// package update closes the running app). Started the same way as at login, so it comes
/// back quietly. Not after a crash or a hang. (The setup program quits and restarts it.)
pub fn restart_after_update() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Recovery::{RegisterApplicationRestart, RESTART_NO_CRASH, RESTART_NO_HANG};
        // Only the arguments: Windows adds the program itself.
        let args = format!("tray {}", crate::tray::LOGIN_ARG);
        let wide: Vec<u16> = args.encode_utf16().chain(Some(0)).collect();
        let result = unsafe { RegisterApplicationRestart(wide.as_ptr(), RESTART_NO_CRASH | RESTART_NO_HANG) };
        if result != 0 {
            log::warn!("could not register the restart after an update ({result:#x})");
        }
    }
}
