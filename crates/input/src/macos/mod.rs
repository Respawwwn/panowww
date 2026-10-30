//! macOS: `CGEvent`s posted to the HID event tap (spec §8). Relative mouse
//! motion carries `kCGMouseEventDeltaX/Y`, which is all a captured-cursor
//! game reads; motion while a button is held is a `*MouseDragged` event.
//! Validated by the spike over a real drag camera.

pub mod accessibility;

use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGError, CGEvent, CGEventField, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMouseButton, CGWarpMouseCursorPosition,
};
use panowww_core::game::PhysicalKey;
use panowww_core::ports::{Camera, InputError, MouseButton, Point};

use crate::keys;

/// The `Camera` of a Mac. Remembers what it holds so `release_all` can let
/// go of every key and button, and picks `MouseMoved` or `*MouseDragged`
/// from the buttons held.
pub struct CgCamera {
    source: Option<CFRetained<CGEventSource>>,
    held_keys: Vec<PhysicalKey>,
    held_buttons: Vec<MouseButton>,
}

impl std::fmt::Debug for CgCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CgCamera")
            .field("held_keys", &self.held_keys)
            .field("held_buttons", &self.held_buttons)
            .finish_non_exhaustive()
    }
}

impl Default for CgCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl CgCamera {
    pub fn new() -> Self {
        Self {
            source: CGEventSource::new(CGEventSourceStateID::HIDSystemState),
            held_keys: Vec::new(),
            held_buttons: Vec::new(),
        }
    }

    /// What is held right now, for logs and tests.
    pub fn held(&self) -> (&[PhysicalKey], &[MouseButton]) {
        (&self.held_keys, &self.held_buttons)
    }

    fn key(&self, key: &PhysicalKey, down: bool) -> Result<(), InputError> {
        let mapping = keys::lookup(key.as_str()).ok_or_else(|| {
            InputError::Failed(format!("{} is not a key Panowww can press", key.as_str()))
        })?;
        let event = CGEvent::new_keyboard_event(self.source.as_deref(), mapping.mac_vk, down)
            .ok_or_else(|| {
                InputError::Failed("CGEventCreateKeyboardEvent returned NULL".to_owned())
            })?;
        post(&event);
        Ok(())
    }

    fn button(&self, button: MouseButton, down: bool) -> Result<(), InputError> {
        let (cg_button, number) = cg_button(button);
        let kind = match (button, down) {
            (MouseButton::Left, true) => CGEventType::LeftMouseDown,
            (MouseButton::Left, false) => CGEventType::LeftMouseUp,
            (MouseButton::Right, true) => CGEventType::RightMouseDown,
            (MouseButton::Right, false) => CGEventType::RightMouseUp,
            (MouseButton::Middle, true) => CGEventType::OtherMouseDown,
            (MouseButton::Middle, false) => CGEventType::OtherMouseUp,
        };
        let event = CGEvent::new_mouse_event(self.source.as_deref(), kind, cursor(), cg_button)
            .ok_or_else(|| {
                InputError::Failed("CGEventCreateMouseEvent returned NULL".to_owned())
            })?;
        CGEvent::set_integer_value_field(
            Some(&event),
            CGEventField::MouseEventButtonNumber,
            number,
        );
        CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventClickState, 1);
        post(&event);
        Ok(())
    }
}

impl Camera for CgCamera {
    fn key_down(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        self.key(&key, true)?;
        if !self.held_keys.contains(&key) {
            self.held_keys.push(key);
        }
        Ok(())
    }

    fn key_up(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        self.key(&key, false)?;
        self.held_keys.retain(|k| *k != key);
        Ok(())
    }

    /// One sub-move. The location is the cursor plus the delta (it stays put
    /// under a dissociated cursor, which is fine: the game reads the delta
    /// fields). A held button turns it into a drag of that button.
    fn mouse_delta(&mut self, dx: i32, dy: i32) -> Result<(), InputError> {
        let (kind, cg_button, number) = match self.held_buttons.last() {
            None => (CGEventType::MouseMoved, CGMouseButton::Left, None),
            Some(MouseButton::Left) => {
                (CGEventType::LeftMouseDragged, CGMouseButton::Left, Some(0))
            }
            Some(MouseButton::Right) => (
                CGEventType::RightMouseDragged,
                CGMouseButton::Right,
                Some(1),
            ),
            Some(MouseButton::Middle) => (
                CGEventType::OtherMouseDragged,
                CGMouseButton::Center,
                Some(2),
            ),
        };
        let current = cursor();
        let target = CGPoint::new(current.x + f64::from(dx), current.y + f64::from(dy));
        let event = CGEvent::new_mouse_event(self.source.as_deref(), kind, target, cg_button)
            .ok_or_else(|| {
                InputError::Failed("CGEventCreateMouseEvent returned NULL".to_owned())
            })?;
        CGEvent::set_integer_value_field(
            Some(&event),
            CGEventField::MouseEventDeltaX,
            i64::from(dx),
        );
        CGEvent::set_integer_value_field(
            Some(&event),
            CGEventField::MouseEventDeltaY,
            i64::from(dy),
        );
        if let Some(number) = number {
            CGEvent::set_integer_value_field(
                Some(&event),
                CGEventField::MouseEventButtonNumber,
                number,
            );
        }
        post(&event);
        Ok(())
    }

    fn button_down(&mut self, button: MouseButton) -> Result<(), InputError> {
        self.button(button, true)?;
        if !self.held_buttons.contains(&button) {
            self.held_buttons.push(button);
        }
        Ok(())
    }

    fn button_up(&mut self, button: MouseButton) -> Result<(), InputError> {
        self.button(button, false)?;
        self.held_buttons.retain(|b| *b != button);
        Ok(())
    }

    /// Warps the cursor to `at` (global points, origin top-left of the main
    /// display) before a drag Step.
    fn park_cursor(&mut self, at: Point) -> Result<(), InputError> {
        let error = CGWarpMouseCursorPosition(CGPoint::new(at.x, at.y));
        if error == CGError::Success {
            Ok(())
        } else {
            Err(InputError::Failed(format!(
                "CGWarpMouseCursorPosition failed: {error:?}"
            )))
        }
    }

    fn stick(&mut self, _x: f64, _y: f64) -> Result<(), InputError> {
        Err(InputError::Failed(
            "The virtual gamepad is available on Windows only.".to_owned(),
        ))
    }

    /// Releases every held key and button, keys first, and keeps going past
    /// a failure so nothing stays stuck.
    fn release_all(&mut self) -> Result<(), InputError> {
        let mut first_error = None;
        for key in std::mem::take(&mut self.held_keys) {
            if let Err(e) = self.key(&key, false) {
                first_error.get_or_insert(e);
            }
        }
        for button in std::mem::take(&mut self.held_buttons) {
            if let Err(e) = self.button(button, false) {
                first_error.get_or_insert(e);
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

fn cursor() -> CGPoint {
    let probe = CGEvent::new(None);
    CGEvent::location(probe.as_deref())
}

fn post(event: &CGEvent) {
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(event));
}

fn cg_button(button: MouseButton) -> (CGMouseButton, i64) {
    match button {
        MouseButton::Left => (CGMouseButton::Left, 0),
        MouseButton::Right => (CGMouseButton::Right, 1),
        MouseButton::Middle => (CGMouseButton::Center, 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_keys_are_refused_before_posting() {
        let mut camera = CgCamera::new();
        let error = camera.key_down(PhysicalKey::new("ShiftLeft")).unwrap_err();
        assert!(matches!(error, InputError::Failed(m) if m.contains("ShiftLeft")));
        assert_eq!(camera.held().0, []);
    }

    #[test]
    fn the_stick_is_windows_only() {
        let mut camera = CgCamera::new();
        assert!(camera.stick(0.8, 0.0).is_err());
    }

    #[test]
    fn buttons_map_to_core_graphics_numbers() {
        assert_eq!(cg_button(MouseButton::Left).1, 0);
        assert_eq!(cg_button(MouseButton::Right).1, 1);
        assert_eq!(cg_button(MouseButton::Middle).1, 2);
    }
}
