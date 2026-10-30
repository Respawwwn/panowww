//! Properties of the calibration adjustment (spec §7.5): never outside the
//! schema bounds, monotonic in the observation, and coarse moves at least
//! as far as fine.

mod strategies;

use panowww_core::calibrate::{Magnitude, Observation, adjust, adjust_step};
use panowww_core::game::validate::{HOLD_MS, MOUSE_DELTA};
use panowww_core::game::{Input, validate};
use panowww_core::plan::Axis;
use proptest::prelude::*;
use strategies::valid_game;

fn observation() -> impl Strategy<Value = Observation> {
    prop_oneof![Just(Observation::Overshot), Just(Observation::Undershot)]
}

fn magnitude() -> impl Strategy<Value = Magnitude> {
    prop_oneof![Just(Magnitude::Fine), Just(Magnitude::Coarse)]
}

fn axis() -> impl Strategy<Value = Axis> {
    prop_oneof![Just(Axis::Turn), Just(Axis::Pitch)]
}

proptest! {
    #[test]
    fn adjust_stays_in_bounds_and_moves_the_right_way(
        value in any::<u32>(),
        observation in observation(),
        magnitude in magnitude(),
    ) {
        for bounds in [HOLD_MS, MOUSE_DELTA] {
            let next = adjust(value, observation, magnitude, &bounds);
            prop_assert!(bounds.contains(&next), "{next} outside {bounds:?}");
            if bounds.contains(&value) {
                match observation {
                    Observation::Undershot => prop_assert!(next >= value),
                    Observation::Overshot => prop_assert!(next <= value),
                }
                if value != *bounds.start() && value != *bounds.end() {
                    prop_assert_ne!(next, value, "a press always does something");
                }
            }
        }
    }

    #[test]
    fn coarse_moves_at_least_as_far_as_fine(value in HOLD_MS, observation in observation()) {
        let fine = adjust(value, observation, Magnitude::Fine, &HOLD_MS);
        let coarse = adjust(value, observation, Magnitude::Coarse, &HOLD_MS);
        prop_assert!(fine.abs_diff(value) <= coarse.abs_diff(value));
    }

    #[test]
    fn adjusting_a_valid_game_keeps_it_valid_and_touches_one_field(
        game in valid_game(),
        axis in axis(),
        observation in observation(),
        magnitude in magnitude(),
    ) {
        let mut adjusted = game.clone();
        let field = adjust_step(&mut adjusted, axis, observation, magnitude);
        prop_assert_eq!(validate(&adjusted), vec![]);
        prop_assert!(field.starts_with("input."));
        // Everything outside the input block is untouched, and so is the other axis.
        prop_assert_eq!((&adjusted.name, adjusted.shots_per_row, adjusted.rows), (&game.name, game.shots_per_row, game.rows));
        let other_axis_field = match (&game.input, axis) {
            (Input::Mouse { pitch_delta, .. }, Axis::Turn) => Some(*pitch_delta),
            (Input::Mouse { turn_delta, .. }, Axis::Pitch) => Some(*turn_delta),
            (Input::Keyboard { pitch_hold_ms, .. } | Input::Gamepad { pitch_hold_ms, .. }, Axis::Turn) => Some(*pitch_hold_ms),
            (Input::Keyboard { turn_hold_ms, .. } | Input::Gamepad { turn_hold_ms, .. }, Axis::Pitch) => Some(*turn_hold_ms),
        };
        let other_axis_after = match (&adjusted.input, axis) {
            (Input::Mouse { pitch_delta, .. }, Axis::Turn) => Some(*pitch_delta),
            (Input::Mouse { turn_delta, .. }, Axis::Pitch) => Some(*turn_delta),
            (Input::Keyboard { pitch_hold_ms, .. } | Input::Gamepad { pitch_hold_ms, .. }, Axis::Turn) => Some(*pitch_hold_ms),
            (Input::Keyboard { turn_hold_ms, .. } | Input::Gamepad { turn_hold_ms, .. }, Axis::Pitch) => Some(*turn_hold_ms),
        };
        prop_assert_eq!(other_axis_field, other_axis_after);
    }
}
