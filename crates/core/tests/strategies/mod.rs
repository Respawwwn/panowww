//! proptest strategies shared by the test binaries: a valid Game of any
//! device, built within the bounds of spec §5.
#![allow(dead_code, reason = "each test binary uses a different subset")]

use panowww_core::game::key::BINDABLE;
use panowww_core::game::validate::{HOLD_MS, MOUSE_DELTA, ROWS, SHOTS_PER_ROW, WAIT_MS};
use panowww_core::game::{
    CURRENT_VERSION, DragButton, Game, GameFormat, Input, MouseMode, PhysicalKey, StartPole,
};
use proptest::prelude::*;
use uuid::Uuid;

pub fn name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]([a-zA-Z0-9 ]{0,58}[a-zA-Z0-9])?"
}

pub fn start_pole() -> impl Strategy<Value = StartPole> {
    prop_oneof![Just(StartPole::Nadir), Just(StartPole::Zenith)]
}

pub fn three_distinct_keys() -> impl Strategy<Value = Vec<PhysicalKey>> {
    prop::sample::subsequence(BINDABLE, 3)
        .prop_shuffle()
        .prop_map(|codes| codes.into_iter().map(PhysicalKey::new).collect())
}

pub fn keyboard() -> impl Strategy<Value = Input> {
    (three_distinct_keys(), HOLD_MS, HOLD_MS).prop_map(|(keys, turn_hold_ms, pitch_hold_ms)| {
        Input::Keyboard {
            turn_right_key: keys[0].clone(),
            look_up_key: keys[1].clone(),
            look_down_key: keys[2].clone(),
            turn_hold_ms,
            pitch_hold_ms,
        }
    })
}

pub fn mouse() -> impl Strategy<Value = Input> {
    let mode_and_button = prop_oneof![
        Just((MouseMode::Move, None)),
        prop::sample::select(&[DragButton::Left, DragButton::Right, DragButton::Middle])
            .prop_map(|b| (MouseMode::Drag, Some(b))),
    ];
    (mode_and_button, MOUSE_DELTA, MOUSE_DELTA, any::<bool>()).prop_map(
        |((mode, drag_button), turn_delta, pitch_delta, invert_vertical)| Input::Mouse {
            mode,
            drag_button,
            turn_delta,
            pitch_delta,
            invert_vertical,
        },
    )
}

pub fn gamepad() -> impl Strategy<Value = Input> {
    (0.1f64..=1.0, HOLD_MS, HOLD_MS, any::<bool>()).prop_map(
        |(stick_deflection, turn_hold_ms, pitch_hold_ms, invert_vertical)| Input::Gamepad {
            stick_deflection,
            turn_hold_ms,
            pitch_hold_ms,
            invert_vertical,
        },
    )
}

pub fn valid_game() -> impl Strategy<Value = Game> {
    (
        any::<u128>(),
        name(),
        prop::option::of("[a-zA-Z0-9 .,]{0,300}"),
        start_pole(),
        SHOTS_PER_ROW,
        ROWS,
        WAIT_MS,
        WAIT_MS,
        prop_oneof![keyboard(), mouse(), gamepad()],
    )
        .prop_map(
            |(
                id,
                name,
                notes,
                start_pole,
                shots_per_row,
                rows,
                wait_after_move_ms,
                wait_after_shot_ms,
                input,
            )| Game {
                format: GameFormat::PanowwwGame,
                version: CURRENT_VERSION,
                id: Uuid::from_u128(id),
                name,
                notes,
                start_pole,
                shots_per_row,
                rows,
                wait_after_move_ms,
                wait_after_shot_ms,
                input,
            },
        )
}
