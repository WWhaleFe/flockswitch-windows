//! Wake from sleep and unlock of the session: the network thread looks for the other
//! computers again at once instead of waiting for its own timers.
//!
//! A hidden window of its own (on its own thread, with its own message loop) receives
//! WM_POWERBROADCAST (resume) and WM_WTSSESSION_CHANGE (unlock) and sets a flag.

use std::sync::atomic::{AtomicBool, Ordering};

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
        WM_WTSSESSION_CHANGE if is_unlock(wparam as u32) => notify(),
        _ => {}
    }
    DefWindowProcW(hwnd, message, wparam, lparam)
}

/// Starts listening on a thread of its own. Call once.
pub fn start() {
    let spawned = std::thread::Builder::new().name("wake".into()).spawn(|| {
        if let Err(e) = message_loop() {
            log::warn!("wake and unlock notices unavailable: {e}");
        }
    });
    if let Err(e) = spawned {
        log::warn!("wake and unlock notices unavailable: {e}");
    }
}

fn message_loop() -> Result<(), String> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION}},
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
        if RegisterClassW(&class_info) == 0 {
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
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
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
    fn only_a_resume_and_an_unlock_count() {
        assert!(super::is_resume(PBT_APMRESUMEAUTOMATIC));
        assert!(super::is_resume(PBT_APMRESUMESUSPEND));
        assert!(!super::is_resume(PBT_APMSUSPEND));
        assert!(super::is_unlock(WTS_SESSION_UNLOCK));
        assert!(!super::is_unlock(WTS_SESSION_LOCK));
    }
}
