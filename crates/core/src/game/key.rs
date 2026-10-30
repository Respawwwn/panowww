//! Physical keys named by their W3C `KeyboardEvent.code`, so a Game binds the
//! same key cap on every keyboard layout. The `input` crate maps a code to
//! `kVK_*` or a scan code.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A physical key by its `KeyboardEvent.code`, such as `KeyW` or
/// `ArrowRight`. Any string deserialises so a damaged Game still loads for
/// the validator; [`PhysicalKey::check`] says whether a Game may bind it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalKey(String);

/// Why a code cannot be bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProblem {
    /// Not a `KeyboardEvent.code` this app knows.
    Unknown,
    /// A modifier or lock key: Shift, Control, Alt, Meta, Caps Lock…
    Modifier,
    /// A key the OS owns: Power, Print Screen, media and browser keys…
    System,
}

impl PhysicalKey {
    /// Wraps a code without checking it.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// The `KeyboardEvent.code`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `Ok` when a Game may bind this key.
    pub fn check(&self) -> Result<(), KeyProblem> {
        let code = self.0.as_str();
        if BINDABLE.contains(&code) {
            Ok(())
        } else if MODIFIERS.contains(&code) {
            Err(KeyProblem::Modifier)
        } else if SYSTEM.contains(&code) {
            Err(KeyProblem::System)
        } else {
            Err(KeyProblem::Unknown)
        }
    }

    /// Shorthand for `check().is_ok()`.
    pub fn is_bindable(&self) -> bool {
        self.check().is_ok()
    }
}

/// Every code a Game may bind: the writing system, functional, control pad,
/// arrow, numpad and function keys of the W3C code tables.
pub const BINDABLE: &[&str] = &[
    // Writing system keys.
    "Backquote",
    "Backslash",
    "BracketLeft",
    "BracketRight",
    "Comma",
    "Digit0",
    "Digit1",
    "Digit2",
    "Digit3",
    "Digit4",
    "Digit5",
    "Digit6",
    "Digit7",
    "Digit8",
    "Digit9",
    "Equal",
    "IntlBackslash",
    "IntlRo",
    "IntlYen",
    "KeyA",
    "KeyB",
    "KeyC",
    "KeyD",
    "KeyE",
    "KeyF",
    "KeyG",
    "KeyH",
    "KeyI",
    "KeyJ",
    "KeyK",
    "KeyL",
    "KeyM",
    "KeyN",
    "KeyO",
    "KeyP",
    "KeyQ",
    "KeyR",
    "KeyS",
    "KeyT",
    "KeyU",
    "KeyV",
    "KeyW",
    "KeyX",
    "KeyY",
    "KeyZ",
    "Minus",
    "Period",
    "Quote",
    "Semicolon",
    "Slash",
    // Functional keys.
    "Backspace",
    "Convert",
    "Enter",
    "KanaMode",
    "NonConvert",
    "Space",
    "Tab",
    // Control pad.
    "Delete",
    "End",
    "Help",
    "Home",
    "Insert",
    "PageDown",
    "PageUp",
    // Arrow pad.
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    // Numpad.
    "Numpad0",
    "Numpad1",
    "Numpad2",
    "Numpad3",
    "Numpad4",
    "Numpad5",
    "Numpad6",
    "Numpad7",
    "Numpad8",
    "Numpad9",
    "NumpadAdd",
    "NumpadBackspace",
    "NumpadClear",
    "NumpadClearEntry",
    "NumpadComma",
    "NumpadDecimal",
    "NumpadDivide",
    "NumpadEnter",
    "NumpadEqual",
    "NumpadMultiply",
    "NumpadParenLeft",
    "NumpadParenRight",
    "NumpadSubtract",
    // Function keys.
    "Escape",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "F13",
    "F14",
    "F15",
    "F16",
    "F17",
    "F18",
    "F19",
    "F20",
    "F21",
    "F22",
    "F23",
    "F24",
];

/// Modifier and lock keys: known codes a Game may not bind.
const MODIFIERS: &[&str] = &[
    "AltLeft",
    "AltRight",
    "CapsLock",
    "ControlLeft",
    "ControlRight",
    "Fn",
    "FnLock",
    "Hyper",
    "MetaLeft",
    "MetaRight",
    "NumLock",
    "ScrollLock",
    "ShiftLeft",
    "ShiftRight",
    "Super",
    "Turbo",
];

/// Keys the OS or the hardware owns: known codes a Game may not bind.
const SYSTEM: &[&str] = &[
    "Abort",
    "Again",
    "AudioVolumeDown",
    "AudioVolumeMute",
    "AudioVolumeUp",
    "BrowserBack",
    "BrowserFavorites",
    "BrowserForward",
    "BrowserHome",
    "BrowserRefresh",
    "BrowserSearch",
    "BrowserStop",
    "ContextMenu",
    "Copy",
    "Cut",
    "Eject",
    "Find",
    "Lang1",
    "Lang2",
    "Lang3",
    "Lang4",
    "Lang5",
    "LaunchApp1",
    "LaunchApp2",
    "LaunchMail",
    "MediaPlayPause",
    "MediaSelect",
    "MediaStop",
    "MediaTrackNext",
    "MediaTrackPrevious",
    "Open",
    "Paste",
    "Pause",
    "Power",
    "PrintScreen",
    "Props",
    "Select",
    "Sleep",
    "Undo",
    "WakeUp",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_arrows_and_numpad_are_bindable() {
        for code in ["KeyW", "ArrowRight", "Numpad8", "Space", "F12", "Escape"] {
            assert_eq!(PhysicalKey::new(code).check(), Ok(()), "{code}");
        }
    }

    #[test]
    fn modifiers_are_refused_as_such() {
        for code in [
            "ShiftLeft",
            "ControlRight",
            "AltLeft",
            "MetaLeft",
            "CapsLock",
            "NumLock",
        ] {
            assert_eq!(
                PhysicalKey::new(code).check(),
                Err(KeyProblem::Modifier),
                "{code}"
            );
        }
    }

    #[test]
    fn system_keys_are_refused_as_such() {
        for code in [
            "Power",
            "PrintScreen",
            "MediaPlayPause",
            "BrowserBack",
            "ContextMenu",
        ] {
            assert_eq!(
                PhysicalKey::new(code).check(),
                Err(KeyProblem::System),
                "{code}"
            );
        }
    }

    #[test]
    fn anything_else_is_unknown() {
        for code in ["", "w", "KeyAA", "Shift", "ArrowRight ", "Right"] {
            assert_eq!(
                PhysicalKey::new(code).check(),
                Err(KeyProblem::Unknown),
                "{code:?}"
            );
        }
    }

    #[test]
    fn the_three_tables_do_not_overlap() {
        for code in BINDABLE {
            assert!(
                !MODIFIERS.contains(code) && !SYSTEM.contains(code),
                "{code}"
            );
        }
        for code in MODIFIERS {
            assert!(!SYSTEM.contains(code), "{code}");
        }
    }

    #[test]
    fn serialises_as_the_bare_code() {
        let key = PhysicalKey::new("KeyW");
        assert_eq!(serde_json::to_string(&key).unwrap(), "\"KeyW\"");
        assert_eq!(
            serde_json::from_str::<PhysicalKey>("\"KeyW\"").unwrap(),
            key
        );
    }
}
