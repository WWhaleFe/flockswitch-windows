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
