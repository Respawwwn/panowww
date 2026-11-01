//! Windows: `SendInput` with scan codes and relative mouse motion (spec §8,
//! §3.3), the virtual pad of [`gamepad`]. Keys go as `KEYEVENTF_SCANCODE`
//! so a Game made on QWERTY presses the same physical key on AZERTY; mouse
//! motion is `MOUSEEVENTF_MOVE`, never `ABSOLUTE`, which is the exact delta
//! a raw-input game reads. Windows has no drag event: motion while a button
//! is held is the drag. Verified 2026-11-01 over `NoClip` and Battlefront 2
//! at 20 px / 8 ms sub-moves.
//!
//! Input into a process of higher integrity than ours is dropped silently
//! (UIPI), which is why the app runs elevated (§8.4); `SendInput` refusing
//! with `ERROR_ACCESS_DENIED` surfaces as [`InputError::PermissionDenied`].

pub mod elevation;
pub mod gamepad;

use panowww_core::game::PhysicalKey;
use panowww_core::ports::{Camera, InputError, MouseButton, Point};
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, GetLastError};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSE_EVENT_FLAGS,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SendInput,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

use crate::keys;

/// The `Camera` of a Windows machine. Remembers what it holds so
/// `release_all` can let go of every key and button, and plugs the virtual
/// pad the first time a Game asks for the stick.
pub struct SendInputCamera {
    held_keys: Vec<PhysicalKey>,
    held_buttons: Vec<MouseButton>,
    pad: Option<gamepad::Pad>,
}

impl std::fmt::Debug for SendInputCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendInputCamera")
            .field("held_keys", &self.held_keys)
            .field("held_buttons", &self.held_buttons)
            .field("pad_plugged", &self.pad.is_some())
            .finish()
    }
}

impl Default for SendInputCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl SendInputCamera {
    pub fn new() -> Self {
        Self {
            held_keys: Vec::new(),
            held_buttons: Vec::new(),
            pad: None,
        }
    }

    /// What is held right now, for logs and tests.
    pub fn held(&self) -> (&[PhysicalKey], &[MouseButton]) {
        (&self.held_keys, &self.held_buttons)
    }

    /// Whether the virtual pad is plugged, for the gamepad Game's setup.
    pub fn pad_plugged(&self) -> bool {
        self.pad.as_ref().is_some_and(gamepad::Pad::is_plugged)
    }

    fn key(key: &PhysicalKey, down: bool) -> Result<(), InputError> {
        let mapping = keys::lookup(key.as_str()).ok_or_else(|| {
            InputError::Failed(format!("{} is not a key Panowww can press", key.as_str()))
        })?;
        send(keyboard_input(
            mapping.win_scan,
            key_flags(mapping.win_extended, down),
        ))
    }

    fn button(button: MouseButton, down: bool) -> Result<(), InputError> {
        send(mouse_input(0, 0, button_flags(button, down)))
    }
}

impl Camera for SendInputCamera {
    fn key_down(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        Self::key(&key, true)?;
        if !self.held_keys.contains(&key) {
            self.held_keys.push(key);
        }
        Ok(())
    }

    fn key_up(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        Self::key(&key, false)?;
        self.held_keys.retain(|k| *k != key);
        Ok(())
    }

    /// One sub-move: a relative delta, the drag being implied by the held
    /// button.
    fn mouse_delta(&mut self, dx: i32, dy: i32) -> Result<(), InputError> {
        send(mouse_input(dx, dy, MOUSEEVENTF_MOVE))
    }

    fn button_down(&mut self, button: MouseButton) -> Result<(), InputError> {
        Self::button(button, true)?;
        if !self.held_buttons.contains(&button) {
            self.held_buttons.push(button);
        }
        Ok(())
    }

    fn button_up(&mut self, button: MouseButton) -> Result<(), InputError> {
        Self::button(button, false)?;
        self.held_buttons.retain(|b| *b != button);
        Ok(())
    }

    /// Moves the cursor to `at` (physical pixels of the virtual screen,
    /// origin top-left of the primary display) before a drag Step.
    fn park_cursor(&mut self, at: Point) -> Result<(), InputError> {
        let (x, y) = pixel(at);
        // SAFETY: takes two plain integers; a point outside every display is
        // clamped by the system, not an error.
        unsafe { SetCursorPos(x, y) }
            .map_err(|e| InputError::Failed(format!("SetCursorPos failed: {e}")))
    }

    /// Right stick to (`x`, `y`), each axis in −1..1, plugging the virtual
    /// pad on the first call.
    fn stick(&mut self, x: f64, y: f64) -> Result<(), InputError> {
        if self.pad.is_none() {
            self.pad = Some(gamepad::Pad::plug()?);
        }
        self.pad.as_mut().expect("plugged just above").stick(x, y)
    }

    /// Releases every held key and button, keys first, centres the stick,
    /// and keeps going past a failure so nothing stays stuck.
    fn release_all(&mut self) -> Result<(), InputError> {
        let mut first_error = None;
        for key in std::mem::take(&mut self.held_keys) {
            if let Err(e) = Self::key(&key, false) {
                first_error.get_or_insert(e);
            }
        }
        for button in std::mem::take(&mut self.held_buttons) {
            if let Err(e) = Self::button(button, false) {
                first_error.get_or_insert(e);
            }
        }
        if let Some(pad) = self.pad.as_mut()
            && let Err(e) = pad.stick(0.0, 0.0)
        {
            first_error.get_or_insert(e);
        }
        first_error.map_or(Ok(()), Err)
    }
}

/// Posts one event. `SendInput` returns how many it injected: 0 with
/// `ERROR_ACCESS_DENIED` when UIPI blocks us.
fn send(input: INPUT) -> Result<(), InputError> {
    let size = i32::try_from(std::mem::size_of::<INPUT>()).expect("INPUT is 40 bytes");
    // SAFETY: one fully initialised INPUT and its true size.
    let sent = unsafe { SendInput(&[input], size) };
    if sent == 1 {
        return Ok(());
    }
    // SAFETY: reads the calling thread's last-error value.
    let error = unsafe { GetLastError() };
    if error == ERROR_ACCESS_DENIED {
        Err(InputError::PermissionDenied)
    } else {
        Err(InputError::Failed(format!(
            "SendInput injected {sent} of 1 events: {error:?}"
        )))
    }
}

fn keyboard_input(scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn mouse_input(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn key_flags(extended: bool, down: bool) -> KEYBD_EVENT_FLAGS {
    let mut flags = KEYEVENTF_SCANCODE;
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }
    flags
}

fn button_flags(button: MouseButton, down: bool) -> MOUSE_EVENT_FLAGS {
    match (button, down) {
        (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
        (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
        (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
        (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
        (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
        (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
    }
}

/// Rounds a point to whole pixels for `SetCursorPos`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "display coordinates are far inside i32 once rounded"
)]
fn pixel(at: Point) -> (i32, i32) {
    (at.x.round() as i32, at.y.round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_keys_are_refused_before_sending() {
        let mut camera = SendInputCamera::new();
        let error = camera.key_down(PhysicalKey::new("ShiftLeft")).unwrap_err();
        assert!(matches!(error, InputError::Failed(m) if m.contains("ShiftLeft")));
        assert_eq!(camera.held().0, []);
        assert!(!camera.pad_plugged());
    }

    #[test]
    fn key_flags_carry_scan_code_extended_and_up() {
        assert_eq!(key_flags(false, true), KEYEVENTF_SCANCODE);
        assert_eq!(
            key_flags(true, true),
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY
        );
        assert_eq!(
            key_flags(true, false),
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
        );
    }

    #[test]
    fn every_button_has_a_down_and_an_up_flag() {
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            assert_ne!(button_flags(button, true), button_flags(button, false));
        }
        assert_eq!(
            button_flags(MouseButton::Right, true),
            MOUSEEVENTF_RIGHTDOWN
        );
    }

    #[test]
    fn mouse_input_is_relative_motion_only() {
        let input = mouse_input(-20, 3, MOUSEEVENTF_MOVE);
        // SAFETY: built by mouse_input, so the mouse member is the live one.
        let mi = unsafe { input.Anonymous.mi };
        assert_eq!((mi.dx, mi.dy), (-20, 3));
        assert_eq!(mi.dwFlags, MOUSEEVENTF_MOVE);
    }

    #[test]
    fn keyboard_input_uses_the_scan_code_not_the_virtual_key() {
        let input = keyboard_input(0x11, key_flags(false, true));
        // SAFETY: built by keyboard_input, so the keyboard member is live.
        let ki = unsafe { input.Anonymous.ki };
        assert_eq!(ki.wVk, VIRTUAL_KEY(0));
        assert_eq!(ki.wScan, 0x11);
    }

    #[test]
    fn parking_rounds_to_pixels() {
        assert_eq!(pixel(Point { x: 959.6, y: 540.4 }), (960, 540));
    }
}
