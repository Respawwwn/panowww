//! The virtual Xbox 360 pad (spec §8.3): `ViGEmBus` 1.22.0 through
//! `vigem-client`, behind our own type so the crate can be swapped. The
//! driver is archived and never redistributed; [`check`] tells onboarding
//! and the pre-flight whether it is installed.
//!
//! Verified 2026-11-01: the pad shows up as an Xbox 360 controller and its
//! sticks move. Plugging while a previous target is still attached fails
//! now and then with Win32 error 483 (`ERROR_DEVICE_HARDWARE_ERROR`) from
//! the wait-ready call, so a pad unplugs itself when dropped and
//! [`Pad::plug`] retries once.

use panowww_core::ports::InputError;
use vigem_client::{Client, TargetId, XGamepad, Xbox360Wired};

/// One plugged virtual pad. Dropping it unplugs the controller.
pub struct Pad {
    target: Xbox360Wired<Client>,
}

impl std::fmt::Debug for Pad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pad")
            .field("plugged", &self.is_plugged())
            .finish_non_exhaustive()
    }
}

impl Pad {
    /// Connects to the bus, plugs a wired Xbox 360 target and waits until
    /// the system sees it. [`InputError::DeviceMissing`] when the driver is
    /// not installed.
    pub fn plug() -> Result<Self, InputError> {
        let client = Client::connect().map_err(map_error)?;
        let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        target.plugin().map_err(map_error)?;
        if let Err(first) = target.wait_ready() {
            tracing::warn!(error = %first, "virtual pad not ready, plugging again");
            let _ = target.unplug();
            target.plugin().map_err(map_error)?;
            target.wait_ready().map_err(map_error)?;
        }
        Ok(Self { target })
    }

    /// Whether the target is still attached to the bus.
    pub fn is_plugged(&self) -> bool {
        self.target.is_attached()
    }

    /// The pad's `XInput` slot (0–3) once a game can see it.
    pub fn user_index(&mut self) -> Option<u32> {
        self.target.get_user_index().ok()
    }

    /// Right stick to (`x`, `y`), each axis clamped to −1..1, `XInput`
    /// convention (up is positive). (0, 0) is neutral.
    pub fn stick(&mut self, x: f64, y: f64) -> Result<(), InputError> {
        let report = XGamepad {
            thumb_rx: axis(x),
            thumb_ry: axis(y),
            ..XGamepad::default()
        };
        self.target.update(&report).map_err(map_error)
    }
}

impl Drop for Pad {
    fn drop(&mut self) {
        let _ = self.target.update(&XGamepad::default());
        let _ = self.target.unplug();
    }
}

/// Whether `ViGEmBus` is installed and reachable: the pre-flight of gamepad
/// Games and the onboarding row.
pub fn check() -> Result<(), InputError> {
    Client::connect().map(drop).map_err(map_error)
}

fn map_error(error: vigem_client::Error) -> InputError {
    match error {
        vigem_client::Error::BusNotFound => InputError::DeviceMissing,
        other => InputError::Failed(format!("ViGEmBus: {other}")),
    }
}

/// A share of full travel to the signed 16-bit stick axis.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the value is clamped to ±i16::MAX before the cast"
)]
fn axis(share: f64) -> i16 {
    (share.clamp(-1.0, 1.0) * f64::from(i16::MAX)).round() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_span_the_signed_range_and_clamp() {
        assert_eq!(axis(0.0), 0);
        assert_eq!(axis(1.0), i16::MAX);
        assert_eq!(axis(-1.0), -i16::MAX);
        assert_eq!(axis(0.5), 16384);
        assert_eq!(axis(7.0), i16::MAX);
        assert_eq!(axis(-7.0), -i16::MAX);
    }

    #[test]
    fn a_missing_driver_is_the_device_missing_error() {
        assert_eq!(
            map_error(vigem_client::Error::BusNotFound),
            InputError::DeviceMissing
        );
        assert!(matches!(
            map_error(vigem_client::Error::NoFreeSlot),
            InputError::Failed(m) if m.starts_with("ViGEmBus:")
        ));
    }

    /// Needs the driver: plugs a pad, waits for an `XInput` slot, drops it.
    #[test]
    #[ignore = "needs ViGEmBus installed; run by hand on the Windows machine"]
    fn a_pad_plugs_and_gets_a_slot() {
        check().unwrap();
        let mut pad = Pad::plug().unwrap();
        assert!(pad.is_plugged());
        assert!(pad.user_index().is_some());
        pad.stick(0.8, -0.2).unwrap();
    }
}
