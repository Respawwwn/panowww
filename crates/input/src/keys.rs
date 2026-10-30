//! The one table from a W3C `KeyboardEvent.code` (physical position) to the
//! macOS virtual key (`kVK_*`, Carbon `Events.h`) and the Windows scan code
//! (set 1, with the `0xE0` extended prefix as a flag). A Game made on a
//! QWERTY keyboard presses the same physical key on AZERTY.
//!
//! The codes here are exactly the ones `panowww_core::game::key::BINDABLE`
//! accepts: the schema validation rejects everything else.

/// One physical key on both operating systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapping {
    /// The `KeyboardEvent.code`.
    pub code: &'static str,
    /// `kVK_*` virtual key code.
    pub mac_vk: u16,
    /// Scan code, set 1.
    pub win_scan: u16,
    /// Needs `KEYEVENTF_EXTENDEDKEY` (the `0xE0` prefix).
    pub win_extended: bool,
}

const fn key(code: &'static str, mac_vk: u16, win_scan: u16) -> KeyMapping {
    KeyMapping {
        code,
        mac_vk,
        win_scan,
        win_extended: false,
    }
}

const fn ext(code: &'static str, mac_vk: u16, win_scan: u16) -> KeyMapping {
    KeyMapping {
        code,
        mac_vk,
        win_scan,
        win_extended: true,
    }
}

/// Every key a Game may bind.
pub const TABLE: &[KeyMapping] = &[
    // Writing system keys.
    key("Backquote", 0x32, 0x29),
    key("Backslash", 0x2A, 0x2B),
    key("BracketLeft", 0x21, 0x1A),
    key("BracketRight", 0x1E, 0x1B),
    key("Comma", 0x2B, 0x33),
    key("Digit0", 0x1D, 0x0B),
    key("Digit1", 0x12, 0x02),
    key("Digit2", 0x13, 0x03),
    key("Digit3", 0x14, 0x04),
    key("Digit4", 0x15, 0x05),
    key("Digit5", 0x17, 0x06),
    key("Digit6", 0x16, 0x07),
    key("Digit7", 0x1A, 0x08),
    key("Digit8", 0x1C, 0x09),
    key("Digit9", 0x19, 0x0A),
    key("Equal", 0x18, 0x0D),
    key("IntlBackslash", 0x0A, 0x56),
    key("IntlRo", 0x5E, 0x73),
    key("IntlYen", 0x5D, 0x7D),
    key("KeyA", 0x00, 0x1E),
    key("KeyB", 0x0B, 0x30),
    key("KeyC", 0x08, 0x2E),
    key("KeyD", 0x02, 0x20),
    key("KeyE", 0x0E, 0x12),
    key("KeyF", 0x03, 0x21),
    key("KeyG", 0x05, 0x22),
    key("KeyH", 0x04, 0x23),
    key("KeyI", 0x22, 0x17),
    key("KeyJ", 0x26, 0x24),
    key("KeyK", 0x28, 0x25),
    key("KeyL", 0x25, 0x26),
    key("KeyM", 0x2E, 0x32),
    key("KeyN", 0x2D, 0x31),
    key("KeyO", 0x1F, 0x18),
    key("KeyP", 0x23, 0x19),
    key("KeyQ", 0x0C, 0x10),
    key("KeyR", 0x0F, 0x13),
    key("KeyS", 0x01, 0x1F),
    key("KeyT", 0x11, 0x14),
    key("KeyU", 0x20, 0x16),
    key("KeyV", 0x09, 0x2F),
    key("KeyW", 0x0D, 0x11),
    key("KeyX", 0x07, 0x2D),
    key("KeyY", 0x10, 0x15),
    key("KeyZ", 0x06, 0x2C),
    key("Minus", 0x1B, 0x0C),
    key("Period", 0x2F, 0x34),
    key("Quote", 0x27, 0x28),
    key("Semicolon", 0x29, 0x27),
    key("Slash", 0x2C, 0x35),
    // Functional keys.
    key("Backspace", 0x33, 0x0E),
    key("Enter", 0x24, 0x1C),
    key("Space", 0x31, 0x39),
    key("Tab", 0x30, 0x0F),
    // Control pad. On Apple keyboards Insert sits where Help is (kVK_Help).
    ext("Delete", 0x75, 0x53),
    ext("End", 0x77, 0x4F),
    ext("Home", 0x73, 0x47),
    ext("Insert", 0x72, 0x52),
    ext("PageDown", 0x79, 0x51),
    ext("PageUp", 0x74, 0x49),
    // Arrow pad.
    ext("ArrowDown", 0x7D, 0x50),
    ext("ArrowLeft", 0x7B, 0x4B),
    ext("ArrowRight", 0x7C, 0x4D),
    ext("ArrowUp", 0x7E, 0x48),
    // Numpad.
    key("Numpad0", 0x52, 0x52),
    key("Numpad1", 0x53, 0x4F),
    key("Numpad2", 0x54, 0x50),
    key("Numpad3", 0x55, 0x51),
    key("Numpad4", 0x56, 0x4B),
    key("Numpad5", 0x57, 0x4C),
    key("Numpad6", 0x58, 0x4D),
    key("Numpad7", 0x59, 0x47),
    key("Numpad8", 0x5B, 0x48),
    key("Numpad9", 0x5C, 0x49),
    key("NumpadAdd", 0x45, 0x4E),
    key("NumpadComma", 0x5F, 0x7E),
    key("NumpadDecimal", 0x41, 0x53),
    ext("NumpadDivide", 0x4B, 0x35),
    ext("NumpadEnter", 0x4C, 0x1C),
    key("NumpadEqual", 0x51, 0x59),
    key("NumpadMultiply", 0x43, 0x37),
    key("NumpadSubtract", 0x4E, 0x4A),
    // Function keys.
    key("Escape", 0x35, 0x01),
    key("F1", 0x7A, 0x3B),
    key("F2", 0x78, 0x3C),
    key("F3", 0x63, 0x3D),
    key("F4", 0x76, 0x3E),
    key("F5", 0x60, 0x3F),
    key("F6", 0x61, 0x40),
    key("F7", 0x62, 0x41),
    key("F8", 0x64, 0x42),
    key("F9", 0x65, 0x43),
    key("F10", 0x6D, 0x44),
    key("F11", 0x67, 0x57),
    key("F12", 0x6F, 0x58),
    key("F13", 0x69, 0x64),
    key("F14", 0x6B, 0x65),
    key("F15", 0x71, 0x66),
    key("F16", 0x6A, 0x67),
    key("F17", 0x40, 0x68),
    key("F18", 0x4F, 0x69),
    key("F19", 0x50, 0x6A),
    key("F20", 0x5A, 0x6B),
];

/// The mapping of a `KeyboardEvent.code`, if a Game may bind it.
pub fn lookup(code: &str) -> Option<&'static KeyMapping> {
    TABLE.iter().find(|k| k.code == code)
}

/// The code of a macOS virtual key.
pub fn code_for_mac_vk(vk: u16) -> Option<&'static str> {
    TABLE.iter().find(|k| k.mac_vk == vk).map(|k| k.code)
}

/// The code of a Windows scan code and its extended flag.
pub fn code_for_win_scan(scan: u16, extended: bool) -> Option<&'static str> {
    TABLE
        .iter()
        .find(|k| k.win_scan == scan && k.win_extended == extended)
        .map(|k| k.code)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use panowww_core::game::key::BINDABLE;

    use super::*;

    #[test]
    fn the_table_is_exactly_the_bindable_set() {
        let table: HashSet<&str> = TABLE.iter().map(|k| k.code).collect();
        let bindable: HashSet<&str> = BINDABLE.iter().copied().collect();
        let missing: Vec<&&str> = bindable.difference(&table).collect();
        let extra: Vec<&&str> = table.difference(&bindable).collect();
        assert!(missing.is_empty(), "bindable but unmapped: {missing:?}");
        assert!(extra.is_empty(), "mapped but not bindable: {extra:?}");
    }

    #[test]
    fn every_code_maps_both_ways_on_both_systems() {
        for k in TABLE {
            assert_eq!(lookup(k.code), Some(k), "{}", k.code);
            assert_eq!(
                code_for_mac_vk(k.mac_vk),
                Some(k.code),
                "{} shares its kVK",
                k.code
            );
            assert_eq!(
                code_for_win_scan(k.win_scan, k.win_extended),
                Some(k.code),
                "{} shares its scan code",
                k.code
            );
        }
        assert_eq!(lookup("ShiftLeft"), None);
        assert_eq!(code_for_mac_vk(0x38), None, "shift is not bindable");
    }

    #[test]
    fn spot_checks_against_the_apple_and_microsoft_tables() {
        let w = lookup("KeyW").unwrap();
        assert_eq!((w.mac_vk, w.win_scan, w.win_extended), (0x0D, 0x11, false));
        let right = lookup("ArrowRight").unwrap();
        assert_eq!(
            (right.mac_vk, right.win_scan, right.win_extended),
            (0x7C, 0x4D, true)
        );
        let enter = lookup("NumpadEnter").unwrap();
        assert_eq!(
            (enter.mac_vk, enter.win_scan, enter.win_extended),
            (0x4C, 0x1C, true)
        );
        assert_eq!(
            lookup("Enter").unwrap().win_scan,
            0x1C,
            "same scan, no prefix"
        );
        assert_eq!(lookup("Space").unwrap().mac_vk, 0x31);
        assert_eq!(lookup("F12").unwrap().mac_vk, 0x6F);
    }
}
