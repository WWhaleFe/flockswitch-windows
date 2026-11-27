//! How a received key (a HID usage) becomes a `SendInput` keyboard event. Pure, so it is
//! also tested on non-Windows build hosts.
use flock_net::keymap;

const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_SCANCODE: u32 = 0x0008;

/// The fields of a `KEYBDINPUT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyFields {
    pub vk: u16,
    pub scan: u16,
    pub flags: u32,
}

/// The event for `usage` going down or up, or None for a key Windows has no code for.
/// The Korean/English key (HID Lang1 0x90) and the Hanja key (Lang2 0x91) go as virtual keys
/// VK_HANGUL / VK_HANJA together with their scan codes (0x72 / 0x71), which is what a Korean
/// keyboard sends and what the Microsoft Korean IME toggles on. Everything else goes by
/// scan code.
pub fn key_fields(usage: u16, down: bool) -> Option<KeyFields> {
    let scan = keymap::hid_to_windows(usage)?;
    let up = if down { 0 } else { KEYEVENTF_KEYUP };
    Some(match keymap::hid_to_windows_vk(usage) {
        Some(vk) => KeyFields { vk, scan: scan.code, flags: up },
        None => KeyFields {
            vk: 0,
            scan: scan.code,
            flags: KEYEVENTF_SCANCODE | if scan.extended { KEYEVENTF_EXTENDEDKEY } else { 0 } | up,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_korean_english_key_is_vk_hangul_with_scan_code_0x72() {
        assert_eq!(keymap::LANG1, 0x90);
        assert_eq!(key_fields(0x90, true), Some(KeyFields { vk: 0x15, scan: 0x72, flags: 0 }));
        assert_eq!(key_fields(0x90, false), Some(KeyFields { vk: 0x15, scan: 0x72, flags: KEYEVENTF_KEYUP }));
    }

    #[test]
    fn the_hanja_key_is_vk_hanja_with_scan_code_0x71() {
        assert_eq!(key_fields(0x91, true), Some(KeyFields { vk: 0x19, scan: 0x71, flags: 0 }));
    }

    #[test]
    fn other_keys_go_by_scan_code() {
        // A: plain scan code. Right arrow: extended.
        assert_eq!(key_fields(0x04, true), Some(KeyFields { vk: 0, scan: 0x1e, flags: KEYEVENTF_SCANCODE }));
        assert_eq!(
            key_fields(0x4f, false),
            Some(KeyFields { vk: 0, scan: 0x4d, flags: KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP })
        );
        assert_eq!(key_fields(0xffff, true), None);
    }
}
