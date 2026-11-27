//! Wake from sleep and unlock of the session: the network thread looks for the other
//! computers again at once instead of waiting for its own timers.
//!
//! A hidden window on the network input thread receives
//! WM_POWERBROADCAST (resume) and WM_WTSSESSION_CHANGE (unlock) and sets a flag. A
//! Modern Standby PC may wake without a resume broadcast, so the window also asks for
//! the display-on power setting and for suspend/resume notices.

use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND};

static PENDING: AtomicBool = AtomicBool::new(false);

/// Records that the PC woke up or was unlocked.
pub fn notify() {
    PENDING.store(true, Ordering::Release);
}

/// True once per wake-up or unlock.
pub fn take() -> bool {
    PENDING.swap(false, Ordering::AcqRel)
}

/// Whether a WM_POWERBROADCAST `event` (wParam) means the PC is running again.
fn is_resume(event: u32) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND};
    event == PBT_APMRESUMEAUTOMATIC || event == PBT_APMRESUMESUSPEND
}

/// Whether a PBT_POWERSETTINGCHANGE `setting` (the GUID in POWERBROADCAST_SETTING) with
/// its first data byte `data` is the display turning on (GUID_CONSOLE_DISPLAY_STATE:
/// 0 off, 1 on, 2 dimmed).
fn is_display_on(setting: &windows_sys::core::GUID, data: u8) -> bool {
    use windows_sys::Win32::System::SystemServices::GUID_CONSOLE_DISPLAY_STATE;
    let g = GUID_CONSOLE_DISPLAY_STATE;
    setting.data1 == g.data1 && setting.data2 == g.data2 && setting.data3 == g.data3 && setting.data4 == g.data4 && data == 1
}

/// Whether a WM_WTSSESSION_CHANGE `event` (wParam) is the unlock of the session.
fn is_unlock(event: u32) -> bool {
    event == windows_sys::Win32::UI::WindowsAndMessaging::WTS_SESSION_UNLOCK
}

unsafe extern "system" fn window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    message: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    match message {
        WM_POWERBROADCAST if is_resume(wparam as u32) => notify(),
        WM_POWERBROADCAST if wparam as u32 == PBT_POWERSETTINGCHANGE && lparam != 0 => {
            // Safety: for this event lParam points at a POWERBROADCAST_SETTING.
            let setting = unsafe { &*(lparam as *const windows_sys::Win32::System::Power::POWERBROADCAST_SETTING) };
            if setting.DataLength >= 1 && is_display_on(&setting.PowerSetting, setting.Data[0]) {
                notify();
            }
        }
        WM_WTSSESSION_CHANGE if is_unlock(wparam as u32) => notify(),
        _ => {}
    }
    DefWindowProcW(hwnd, message, wparam, lparam)
}

/// Owned and dropped by the input thread, whose message loop dispatches the notices.
pub struct Listener(HWND, [isize; 2]);

impl Listener {
    pub fn new() -> Result<Self, String> {
        use std::ptr::{null, null_mut};
        use windows_sys::Win32::{
            System::{
                Power::{RegisterPowerSettingNotification, RegisterSuspendResumeNotification},
                LibraryLoader::GetModuleHandleW,
                RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION},
                SystemServices::GUID_CONSOLE_DISPLAY_STATE,
            },
            UI::WindowsAndMessaging::*,
        };
        let class: Vec<u16> = "FlockswitchWake\0".encode_utf16().collect();
        // Safety: plain Win32 calls; the window is only used on this thread.
        unsafe {
            let instance = GetModuleHandleW(null());
            let class_info = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: null_mut(),
                lpszMenuName: null(),
                lpszClassName: class.as_ptr(),
            };
            if RegisterClassW(&class_info) == 0
                && std::io::Error::last_os_error().raw_os_error() != Some(ERROR_CLASS_ALREADY_EXISTS as i32)
            {
                return Err("cannot register the window class".into());
            }
            // A top-level window that is never shown: message-only windows get no power broadcasts.
            let hwnd = CreateWindowExW(0, class.as_ptr(), class.as_ptr(), WS_OVERLAPPED, 0, 0, 0, 0, null_mut(), null_mut(), instance, null());
            if hwnd.is_null() {
                return Err("cannot create the window".into());
            }
            if WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0 {
                log::warn!("unlock notices unavailable (WTSRegisterSessionNotification failed); wake notices still work");
            }
            // Modern Standby (S0 low power idle): without these a resume may not be broadcast
            // to a plain window. A failed registration only loses that extra notice.
            let display = RegisterPowerSettingNotification(hwnd, &GUID_CONSOLE_DISPLAY_STATE, DEVICE_NOTIFY_WINDOW_HANDLE);
            let resume = RegisterSuspendResumeNotification(hwnd, DEVICE_NOTIFY_WINDOW_HANDLE);
            if display == 0 || resume == 0 {
                log::warn!("modern standby wake notices unavailable (display {}, resume {})", display != 0, resume != 0);
            }
            Ok(Self(hwnd, [display, resume]))
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // Safety: the window was created on this thread and no longer receives notices.
        unsafe {
            use windows_sys::Win32::System::Power::{UnregisterPowerSettingNotification, UnregisterSuspendResumeNotification};
            if self.1[0] != 0 { UnregisterPowerSettingNotification(self.1[0]); }
            if self.1[1] != 0 { UnregisterSuspendResumeNotification(self.1[1]); }
            windows_sys::Win32::System::RemoteDesktop::WTSUnRegisterSessionNotification(self.0);
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    #[test]
    fn an_event_is_taken_once() {
        super::take();
        assert!(!super::take());
        super::notify();
        super::notify();
        assert!(super::take());
        assert!(!super::take());
    }

    #[test]
    fn only_display_on_counts() {
        use windows_sys::Win32::System::SystemServices::{GUID_CONSOLE_DISPLAY_STATE, GUID_MONITOR_POWER_ON};
        assert!(super::is_display_on(&GUID_CONSOLE_DISPLAY_STATE, 1));
        assert!(!super::is_display_on(&GUID_CONSOLE_DISPLAY_STATE, 0));
        assert!(!super::is_display_on(&GUID_CONSOLE_DISPLAY_STATE, 2));
        assert!(!super::is_display_on(&GUID_MONITOR_POWER_ON, 1));
    }

    #[test]
    fn only_a_resume_and_an_unlock_count() {
        assert!(super::is_resume(PBT_APMRESUMEAUTOMATIC));
        assert!(super::is_resume(PBT_APMRESUMESUSPEND));
        assert!(!super::is_resume(PBT_APMSUSPEND));
        assert!(super::is_unlock(WTS_SESSION_UNLOCK));
        assert!(!super::is_unlock(WTS_SESSION_LOCK));
    }
}
