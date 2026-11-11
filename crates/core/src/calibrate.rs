//! The calibration assistant's maths (spec §7.5): after Check overlap the
//! user says *Overshot* (too little Overlap, the Step is too big) or
//! *Undershot*, and the Step's own field moves by a relative amount,
//! integer-rounded and clamped to the schema bounds. The Tests' count
//! buttons (Shots per Row, Rows) are a plain clamped add, done in the UI.

use std::ops::RangeInclusive;

use crate::constants::{ADJUST_COARSE_PERCENT, ADJUST_FINE_PERCENT};
use crate::game::validate::{HOLD_MS, MOUSE_DELTA};
use crate::game::{Game, Input};
use crate::plan::Axis;

/// What the Overlap told the user about the Step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation {
    /// Too little Overlap: the Step is too big.
    Overshot,
    /// Too much Overlap: the Step is too small.
    Undershot,
}

/// How much to change by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Magnitude {
    /// ±[`ADJUST_FINE_PERCENT`].
    Fine,
    /// ±[`ADJUST_COARSE_PERCENT`].
    Coarse,
}

impl Magnitude {
    pub fn percent(self) -> u32 {
        match self {
            Self::Fine => ADJUST_FINE_PERCENT,
            Self::Coarse => ADJUST_COARSE_PERCENT,
        }
    }
}

/// `value` changed by `magnitude` percent of itself, rounded to the nearest
/// integer and at least 1 so a press always does something, shrunk after an
/// overshoot and grown after an undershoot, then clamped to `bounds`.
pub fn adjust(
    value: u32,
    observation: Observation,
    magnitude: Magnitude,
    bounds: &RangeInclusive<u32>,
) -> u32 {
    let change = (u64::from(value) * u64::from(magnitude.percent()) + 50) / 100;
    let change = u32::try_from(change).unwrap_or(u32::MAX).max(1);
    let next = match observation {
        Observation::Overshot => value.saturating_sub(change),
        Observation::Undershot => value.saturating_add(change),
    };
    next.clamp(*bounds.start(), *bounds.end())
}

/// Adjusts the field that sizes the Step of `game` on `axis` and names it
/// as the validator does (`input.turnDelta`, `input.pitchHoldMs`…). Holds
/// for keyboard and gamepad Games, deltas for mouse Games; the gamepad's
/// `stickDeflection` is left alone since it sizes both axes at once.
pub fn adjust_step(
    game: &mut Game,
    axis: Axis,
    observation: Observation,
    magnitude: Magnitude,
) -> &'static str {
    let (value, bounds, field) = match (&mut game.input, axis) {
        (
            Input::Keyboard { turn_hold_ms, .. } | Input::Gamepad { turn_hold_ms, .. },
            Axis::Turn,
        ) => (turn_hold_ms, &HOLD_MS, "input.turnHoldMs"),
        (
            Input::Keyboard { pitch_hold_ms, .. } | Input::Gamepad { pitch_hold_ms, .. },
            Axis::Pitch,
        ) => (pitch_hold_ms, &HOLD_MS, "input.pitchHoldMs"),
        (Input::Mouse { turn_delta, .. }, Axis::Turn) => {
            (turn_delta, &MOUSE_DELTA, "input.turnDelta")
        }
        (Input::Mouse { pitch_delta, .. }, Axis::Pitch) => {
            (pitch_delta, &MOUSE_DELTA, "input.pitchDelta")
        }
    };
    *value = adjust(*value, observation, magnitude, bounds);
    field
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Device, NewGameDefaults};

    #[test]
    fn adjust_is_relative_rounded_and_clamped() {
        let bounds = 1..=5000;
        assert_eq!(
            adjust(100, Observation::Undershot, Magnitude::Fine, &bounds),
            102
        );
        assert_eq!(
            adjust(100, Observation::Overshot, Magnitude::Fine, &bounds),
            98
        );
        assert_eq!(
            adjust(100, Observation::Undershot, Magnitude::Coarse, &bounds),
            110
        );
        assert_eq!(
            adjust(100, Observation::Overshot, Magnitude::Coarse, &bounds),
            90
        );
        assert_eq!(
            adjust(125, Observation::Undershot, Magnitude::Fine, &bounds),
            128,
            "2.5 rounds up"
        );
        assert_eq!(
            adjust(124, Observation::Undershot, Magnitude::Fine, &bounds),
            126,
            "2.48 rounds down"
        );
        assert_eq!(
            adjust(10, Observation::Undershot, Magnitude::Fine, &bounds),
            11,
            "never a no-op"
        );
        assert_eq!(
            adjust(1, Observation::Overshot, Magnitude::Coarse, &bounds),
            1,
            "clamped low"
        );
        assert_eq!(
            adjust(4990, Observation::Undershot, Magnitude::Coarse, &bounds),
            5000,
            "clamped high"
        );
    }

    #[test]
    fn adjust_step_picks_the_field_of_the_device_and_axis() {
        for (device, turn_field, pitch_field) in [
            (Device::Keyboard, "input.turnHoldMs", "input.pitchHoldMs"),
            (Device::Mouse, "input.turnDelta", "input.pitchDelta"),
            (Device::Gamepad, "input.turnHoldMs", "input.pitchHoldMs"),
        ] {
            let mut game = Game::new("x", NewGameDefaults::default());
            game.input = Input::defaults_for(device);
            let before = game.input.clone();
            assert_eq!(
                adjust_step(
                    &mut game,
                    Axis::Turn,
                    Observation::Undershot,
                    Magnitude::Coarse
                ),
                turn_field
            );
            assert_ne!(game.input, before, "{device:?} turn");
            let after_turn = game.input.clone();
            assert_eq!(
                adjust_step(
                    &mut game,
                    Axis::Pitch,
                    Observation::Overshot,
                    Magnitude::Fine
                ),
                pitch_field
            );
            assert_ne!(game.input, after_turn, "{device:?} pitch");
            assert!(
                crate::game::validate(&game).is_empty(),
                "{device:?} stays valid"
            );
        }
    }

    #[test]
    fn mouse_turn_adjusts_only_the_turn_delta() {
        let mut game = Game::new("x", NewGameDefaults::default());
        game.input = Input::defaults_for(Device::Mouse);
        adjust_step(
            &mut game,
            Axis::Turn,
            Observation::Overshot,
            Magnitude::Coarse,
        );
        let Input::Mouse {
            turn_delta,
            pitch_delta,
            ..
        } = game.input
        else {
            unreachable!()
        };
        assert_eq!((turn_delta, pitch_delta), (450, 500));
    }
}
