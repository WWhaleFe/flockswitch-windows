//! Which keys to release locally when capture starts with modifiers already held.

const VK_LMENU: u16 = 0xa4;
const VK_RMENU: u16 = 0xa5;
const VK_LWIN: u16 = 0x5b;
const VK_RWIN: u16 = 0x5c;

/// Unassigned virtual key: tapped while Alt or Win is held so the release does not open a menu.
pub const VK_UNASSIGNED: u16 = 0xe8;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyStep {
    Up(u16),
    Tap(u16),
}
/// Local key-ups for modifiers held when capture starts: Ctrl and Shift first, then (only
/// when Alt or Win is among them) a tap of an unassigned key, then Alt and Win.
pub fn release_plan(held: &[u16]) -> Vec<KeyStep> {
    let is_menu = |vk: u16| matches!(vk, VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN);
    let mut plan: Vec<KeyStep> = held.iter().copied().filter(|&vk| !is_menu(vk)).map(KeyStep::Up).collect();
    let menu: Vec<u16> = held.iter().copied().filter(|&vk| is_menu(vk)).collect();
    if !menu.is_empty() {
        plan.push(KeyStep::Tap(VK_UNASSIGNED));
        plan.extend(menu.into_iter().map(KeyStep::Up));
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    const VK_LCONTROL: u16 = 0xa2;
    const VK_RSHIFT: u16 = 0xa1;
    #[test]
    fn held_modifiers_are_released_with_a_tap_before_alt_and_win() {
        assert!(release_plan(&[]).is_empty());
        assert_eq!(
            release_plan(&[VK_LCONTROL, VK_RSHIFT]),
            vec![KeyStep::Up(VK_LCONTROL), KeyStep::Up(VK_RSHIFT)]
        );
        assert_eq!(
            release_plan(&[VK_LMENU, VK_LCONTROL, VK_LWIN]),
            vec![KeyStep::Up(VK_LCONTROL), KeyStep::Tap(VK_UNASSIGNED), KeyStep::Up(VK_LMENU), KeyStep::Up(VK_LWIN)]
        );
    }
}
