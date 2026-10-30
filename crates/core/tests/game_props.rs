//! Properties of the Game rules: every valid Game round-trips through its
//! file, and every out-of-bounds value yields exactly one message, for its
//! field.

mod strategies;

use panowww_core::game::validate::{HOLD_MS, MOUSE_DELTA, ROWS, SHOTS_PER_ROW, WAIT_MS};
use panowww_core::game::{Device, DragButton, Game, Input, MouseMode, PhysicalKey, validate};
use proptest::prelude::*;
use strategies::valid_game;

/// One rule broken on one field.
#[derive(Debug, Clone)]
enum Breach {
    ShotsPerRow(u32),
    Rows(u32),
    WaitAfterMove(u32),
    WaitAfterShot(u32),
    BlankName(String),
    LongName(usize),
    LongNotes(usize),
    TurnHold(Device, u32),
    PitchHold(Device, u32),
    TurnDelta(u32),
    PitchDelta(u32),
    StickDeflection(f64),
    /// Which of the three keys, and a code no Game may bind.
    Key(usize, String),
    DragWithoutButton,
    ButtonWithoutDrag,
}

fn outside(range: std::ops::RangeInclusive<u32>) -> impl Strategy<Value = u32> {
    any::<u32>().prop_filter("outside the range", move |v| !range.contains(v))
}

fn hold_device() -> impl Strategy<Value = Device> {
    prop_oneof![Just(Device::Keyboard), Just(Device::Gamepad)]
}

fn unbindable_code() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(&[
            "ShiftLeft",
            "MetaRight",
            "CapsLock",
            "Power",
            "PrintScreen",
            "MediaPlayPause"
        ])
        .prop_map(str::to_owned),
        "[A-Za-z0-9]{0,12}".prop_filter("a known key", |c| !PhysicalKey::new(c.as_str())
            .is_bindable()),
    ]
}

fn breach() -> impl Strategy<Value = Breach> {
    prop_oneof![
        outside(SHOTS_PER_ROW).prop_map(Breach::ShotsPerRow),
        outside(ROWS).prop_map(Breach::Rows),
        outside(WAIT_MS).prop_map(Breach::WaitAfterMove),
        outside(WAIT_MS).prop_map(Breach::WaitAfterShot),
        "[ \t\n]{0,5}".prop_map(Breach::BlankName),
        (61usize..200).prop_map(Breach::LongName),
        (2001usize..2100).prop_map(Breach::LongNotes),
        (hold_device(), outside(HOLD_MS)).prop_map(|(d, v)| Breach::TurnHold(d, v)),
        (hold_device(), outside(HOLD_MS)).prop_map(|(d, v)| Breach::PitchHold(d, v)),
        outside(MOUSE_DELTA).prop_map(Breach::TurnDelta),
        outside(MOUSE_DELTA).prop_map(Breach::PitchDelta),
        prop_oneof![-1000.0f64..0.1, 1.0f64..1000.0]
            .prop_filter("inside", |v| !(0.1..=1.0).contains(v))
            .prop_map(Breach::StickDeflection),
        (0usize..3, unbindable_code()).prop_map(|(i, c)| Breach::Key(i, c)),
        Just(Breach::DragWithoutButton),
        Just(Breach::ButtonWithoutDrag),
    ]
}

/// Applies `breach` to a valid Game and names the field it must be
/// reported on.
fn apply(game: &mut Game, breach: Breach) -> &'static str {
    match breach {
        Breach::ShotsPerRow(v) => {
            game.shots_per_row = v;
            "shotsPerRow"
        }
        Breach::Rows(v) => {
            game.rows = v;
            "rows"
        }
        Breach::WaitAfterMove(v) => {
            game.wait_after_move_ms = v;
            "waitAfterMoveMs"
        }
        Breach::WaitAfterShot(v) => {
            game.wait_after_shot_ms = v;
            "waitAfterShotMs"
        }
        Breach::BlankName(s) => {
            game.name = s;
            "name"
        }
        Breach::LongName(n) => {
            game.name = "n".repeat(n);
            "name"
        }
        Breach::LongNotes(n) => {
            game.notes = Some("n".repeat(n));
            "notes"
        }
        input_breach => apply_input(game, input_breach),
    }
}

/// The `input` half of [`apply`]: installs the device the breach needs,
/// then breaks one of its fields.
fn apply_input(game: &mut Game, breach: Breach) -> &'static str {
    match breach {
        Breach::TurnHold(device, v) => {
            game.input = Input::defaults_for(device);
            match &mut game.input {
                Input::Keyboard { turn_hold_ms, .. } | Input::Gamepad { turn_hold_ms, .. } => {
                    *turn_hold_ms = v;
                }
                Input::Mouse { .. } => unreachable!(),
            }
            "input.turnHoldMs"
        }
        Breach::PitchHold(device, v) => {
            game.input = Input::defaults_for(device);
            match &mut game.input {
                Input::Keyboard { pitch_hold_ms, .. } | Input::Gamepad { pitch_hold_ms, .. } => {
                    *pitch_hold_ms = v;
                }
                Input::Mouse { .. } => unreachable!(),
            }
            "input.pitchHoldMs"
        }
        Breach::TurnDelta(v) => {
            game.input = Input::defaults_for(Device::Mouse);
            let Input::Mouse { turn_delta, .. } = &mut game.input else {
                unreachable!()
            };
            *turn_delta = v;
            "input.turnDelta"
        }
        Breach::PitchDelta(v) => {
            game.input = Input::defaults_for(Device::Mouse);
            let Input::Mouse { pitch_delta, .. } = &mut game.input else {
                unreachable!()
            };
            *pitch_delta = v;
            "input.pitchDelta"
        }
        Breach::StickDeflection(v) => {
            game.input = Input::defaults_for(Device::Gamepad);
            let Input::Gamepad {
                stick_deflection, ..
            } = &mut game.input
            else {
                unreachable!()
            };
            *stick_deflection = v;
            "input.stickDeflection"
        }
        Breach::Key(which, code) => {
            game.input = Input::defaults_for(Device::Keyboard);
            let Input::Keyboard {
                turn_right_key,
                look_up_key,
                look_down_key,
                ..
            } = &mut game.input
            else {
                unreachable!()
            };
            let (slot, field) = match which {
                0 => (turn_right_key, "input.turnRightKey"),
                1 => (look_up_key, "input.lookUpKey"),
                _ => (look_down_key, "input.lookDownKey"),
            };
            *slot = PhysicalKey::new(code);
            field
        }
        Breach::DragWithoutButton => {
            game.input = Input::defaults_for(Device::Mouse);
            let Input::Mouse { mode, .. } = &mut game.input else {
                unreachable!()
            };
            *mode = MouseMode::Drag;
            "input.dragButton"
        }
        Breach::ButtonWithoutDrag => {
            game.input = Input::defaults_for(Device::Mouse);
            let Input::Mouse { drag_button, .. } = &mut game.input else {
                unreachable!()
            };
            *drag_button = Some(DragButton::Left);
            "input.dragButton"
        }
        game_breach => unreachable!("{game_breach:?} is handled by `apply`"),
    }
}

proptest! {
    #[test]
    fn any_valid_game_is_valid_and_round_trips(game in valid_game()) {
        prop_assert_eq!(validate(&game), vec![]);
        let loaded = Game::from_json(&game.to_json()).unwrap();
        prop_assert_eq!(&loaded.game, &game);
        prop_assert!(!loaded.migrated);
    }

    #[test]
    fn any_out_of_bounds_value_yields_one_message_for_its_field(game in valid_game(), breach in breach()) {
        let mut game = game;
        let field = apply(&mut game, breach);
        let errors = validate(&game);
        prop_assert_eq!(errors.len(), 1, "{:?}", errors);
        prop_assert_eq!(errors[0].field.as_str(), field);
        prop_assert!(matches!(Game::from_json(&game.to_json()), Err(panowww_core::game::GameError::Invalid(_))));
    }
}
