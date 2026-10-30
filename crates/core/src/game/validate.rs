//! The field rules of a Game (spec §5), checked the same way on load and in
//! the editor: one plain-English message per failing field, never two.

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::key::KeyProblem;
use super::{Game, Input, MouseMode, PhysicalKey};

/// Longest Game name, in characters.
pub const NAME_MAX_CHARS: usize = 60;
/// Longest notes, in characters.
pub const NOTES_MAX_CHARS: usize = 2000;
/// Shots in one Row.
pub const SHOTS_PER_ROW: RangeInclusive<u32> = 3..=72;
/// Rows from pole to pole.
pub const ROWS: RangeInclusive<u32> = 2..=36;
/// `waitAfterMoveMs` and `waitAfterShotMs`.
pub const WAIT_MS: RangeInclusive<u32> = 0..=10_000;
/// Key and stick holds per Step.
pub const HOLD_MS: RangeInclusive<u32> = 1..=5_000;
/// Mouse units per Step.
pub const MOUSE_DELTA: RangeInclusive<u32> = 1..=100_000;
/// Right-stick deflection.
pub const STICK_DEFLECTION: RangeInclusive<f64> = 0.1..=1.0;

/// One failing field, named by its JSON path (`shotsPerRow`,
/// `input.turnRightKey`), with the message the UI shows next to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    fn new(field: &str, message: impl Into<String>) -> Self {
        Self {
            field: field.to_owned(),
            message: message.into(),
        }
    }
}

/// Every rule of spec §5 that a Game can break on its own. Name uniqueness
/// needs the other Games on the machine: see [`validate_name_unique`].
pub fn validate(game: &Game) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut fail = |field: &str, message: String| errors.push(FieldError::new(field, message));

    let name_chars = game.name.trim().chars().count();
    if name_chars == 0 {
        fail("name", "Give the Game a name.".to_owned());
    } else if name_chars > NAME_MAX_CHARS {
        fail(
            "name",
            format!("Keep the name to {NAME_MAX_CHARS} characters or fewer."),
        );
    }
    if game
        .notes
        .as_ref()
        .is_some_and(|n| n.chars().count() > NOTES_MAX_CHARS)
    {
        fail(
            "notes",
            format!("Keep the notes to {NOTES_MAX_CHARS} characters or fewer."),
        );
    }
    if !SHOTS_PER_ROW.contains(&game.shots_per_row) {
        fail("shotsPerRow", between("Shots per Row", &SHOTS_PER_ROW, ""));
    }
    if !ROWS.contains(&game.rows) {
        fail("rows", between("Rows", &ROWS, ""));
    }
    if !WAIT_MS.contains(&game.wait_after_move_ms) {
        fail(
            "waitAfterMoveMs",
            between("Wait after move", &WAIT_MS, " ms"),
        );
    }
    if !WAIT_MS.contains(&game.wait_after_shot_ms) {
        fail(
            "waitAfterShotMs",
            between("Wait after Shot", &WAIT_MS, " ms"),
        );
    }
    errors.extend(validate_input(&game.input));
    errors
}

/// The rules of the `input` block, with fields named `input.<field>`.
fn validate_input(input: &Input) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut fail = |field: &str, message: String| {
        errors.push(FieldError::new(&format!("input.{field}"), message));
    };

    match input {
        Input::Keyboard {
            turn_right_key,
            look_up_key,
            look_down_key,
            turn_hold_ms,
            pitch_hold_ms,
        } => {
            let keys = [
                ("turnRightKey", turn_right_key),
                ("lookUpKey", look_up_key),
                ("lookDownKey", look_down_key),
            ];
            for (field, key) in keys {
                if let Some(message) = key_message(key, &keys) {
                    fail(field, message);
                }
            }
            if !HOLD_MS.contains(turn_hold_ms) {
                fail("turnHoldMs", between("Turn hold", &HOLD_MS, " ms"));
            }
            if !HOLD_MS.contains(pitch_hold_ms) {
                fail("pitchHoldMs", between("Pitch hold", &HOLD_MS, " ms"));
            }
        }
        Input::Mouse {
            mode,
            drag_button,
            turn_delta,
            pitch_delta,
            ..
        } => {
            match (mode, drag_button) {
                (MouseMode::Drag, None) => fail(
                    "dragButton",
                    "Choose the mouse button to drag with.".to_owned(),
                ),
                (MouseMode::Move, Some(_)) => fail(
                    "dragButton",
                    "A drag button only applies in drag mode.".to_owned(),
                ),
                _ => {}
            }
            if !MOUSE_DELTA.contains(turn_delta) {
                fail("turnDelta", between("Turn distance", &MOUSE_DELTA, ""));
            }
            if !MOUSE_DELTA.contains(pitch_delta) {
                fail("pitchDelta", between("Pitch distance", &MOUSE_DELTA, ""));
            }
        }
        Input::Gamepad {
            stick_deflection,
            turn_hold_ms,
            pitch_hold_ms,
            ..
        } => {
            // `contains` is false for NaN too.
            if !STICK_DEFLECTION.contains(stick_deflection) {
                fail(
                    "stickDeflection",
                    "Stick deflection must be between 0.1 and 1.0.".to_owned(),
                );
            }
            if !HOLD_MS.contains(turn_hold_ms) {
                fail("turnHoldMs", between("Turn hold", &HOLD_MS, " ms"));
            }
            if !HOLD_MS.contains(pitch_hold_ms) {
                fail("pitchHoldMs", between("Pitch hold", &HOLD_MS, " ms"));
            }
        }
    }
    errors
}

/// The one message for a key field: a bad code first, then a clash with
/// another key of the same Game.
fn key_message(key: &PhysicalKey, all: &[(&str, &PhysicalKey); 3]) -> Option<String> {
    match key.check() {
        Err(KeyProblem::Unknown) => Some("Pick a key on the keyboard.".to_owned()),
        Err(KeyProblem::Modifier) => {
            Some("Modifier keys such as Shift or Ctrl cannot be used.".to_owned())
        }
        Err(KeyProblem::System) => Some("System and media keys cannot be used.".to_owned()),
        Ok(()) if all.iter().filter(|(_, other)| *other == key).count() > 1 => {
            Some("Each key must be different.".to_owned())
        }
        Ok(()) => None,
    }
}

fn between(what: &str, range: &RangeInclusive<u32>, unit: &str) -> String {
    format!(
        "{what} must be between {} and {}{unit}.",
        range.start(),
        range.end()
    )
}

/// The `name` error when another Game on this machine already has this name,
/// compared case-insensitively after trimming.
pub fn validate_name_unique<S: AsRef<str>>(
    name: &str,
    other_names: impl IntoIterator<Item = S>,
) -> Option<FieldError> {
    super::name::is_taken(name, other_names)
        .then(|| FieldError::new("name", "Another Game already has this name."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Device, DragButton, NewGameDefaults};

    fn game() -> Game {
        Game::new("Test", NewGameDefaults::default())
    }

    fn fields(game: &Game) -> Vec<String> {
        validate(game).into_iter().map(|e| e.field).collect()
    }

    #[test]
    fn a_fresh_game_is_valid() {
        assert_eq!(validate(&game()), []);
    }

    #[test]
    fn name_is_trimmed_and_bounded() {
        let mut g = game();
        g.name = "   ".to_owned();
        assert_eq!(fields(&g), ["name"]);
        g.name = "é".repeat(60);
        assert!(validate(&g).is_empty(), "60 characters, not bytes");
        g.name = "é".repeat(61);
        assert_eq!(fields(&g), ["name"]);
        g.name = format!("  {}  ", "a".repeat(60));
        assert!(
            validate(&g).is_empty(),
            "padding is trimmed before counting"
        );
    }

    #[test]
    fn notes_are_optional_and_bounded() {
        let mut g = game();
        g.notes = Some("é".repeat(2000));
        assert_eq!(validate(&g), []);
        g.notes = Some("é".repeat(2001));
        assert_eq!(fields(&g), ["notes"]);
    }

    #[test]
    fn grid_and_waits_have_their_bounds() {
        for (set, field, low, high) in [
            (
                (|g: &mut Game, v| g.shots_per_row = v) as fn(&mut Game, u32),
                "shotsPerRow",
                3,
                72,
            ),
            (|g, v| g.rows = v, "rows", 2, 36),
            (
                |g, v| g.wait_after_move_ms = v,
                "waitAfterMoveMs",
                0,
                10_000,
            ),
            (
                |g, v| g.wait_after_shot_ms = v,
                "waitAfterShotMs",
                0,
                10_000,
            ),
        ] {
            let mut g = game();
            set(&mut g, low);
            assert!(validate(&g).is_empty(), "{field} = {low}");
            set(&mut g, high);
            assert!(validate(&g).is_empty(), "{field} = {high}");
            set(&mut g, high + 1);
            assert_eq!(fields(&g), [field]);
            if let Some(below) = low.checked_sub(1) {
                set(&mut g, below);
                assert_eq!(fields(&g), [field]);
            }
        }
    }

    #[test]
    fn keyboard_keys_must_be_bindable_and_distinct() {
        let mut g = game();
        g.input = Input::Keyboard {
            turn_right_key: PhysicalKey::new("ShiftLeft"),
            look_up_key: PhysicalKey::new("Power"),
            look_down_key: PhysicalKey::new("nope"),
            turn_hold_ms: 100,
            pitch_hold_ms: 100,
        };
        let errors = validate(&g);
        assert_eq!(
            errors.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["input.turnRightKey", "input.lookUpKey", "input.lookDownKey"]
        );
        assert!(errors[0].message.contains("Modifier"));
        assert!(errors[1].message.contains("System"));
        assert!(errors[2].message.contains("Pick a key"));

        g.input = Input::Keyboard {
            turn_right_key: PhysicalKey::new("KeyD"),
            look_up_key: PhysicalKey::new("KeyD"),
            look_down_key: PhysicalKey::new("KeyS"),
            turn_hold_ms: 100,
            pitch_hold_ms: 100,
        };
        let errors = validate(&g);
        assert_eq!(
            errors.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["input.turnRightKey", "input.lookUpKey"]
        );
        assert!(
            errors
                .iter()
                .all(|e| e.message == "Each key must be different.")
        );
    }

    #[test]
    fn a_bad_key_gets_one_message_even_when_it_also_clashes() {
        let mut g = game();
        g.input = Input::Keyboard {
            turn_right_key: PhysicalKey::new("ShiftLeft"),
            look_up_key: PhysicalKey::new("ShiftLeft"),
            look_down_key: PhysicalKey::new("KeyS"),
            turn_hold_ms: 100,
            pitch_hold_ms: 100,
        };
        let errors = validate(&g);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.message.contains("Modifier")));
    }

    #[test]
    fn holds_are_1_to_5000() {
        for device in [Device::Keyboard, Device::Gamepad] {
            let mut g = game();
            g.input = Input::defaults_for(device);
            for (turn, pitch) in [(0, 1), (5000, 5001)] {
                match &mut g.input {
                    Input::Keyboard {
                        turn_hold_ms,
                        pitch_hold_ms,
                        ..
                    }
                    | Input::Gamepad {
                        turn_hold_ms,
                        pitch_hold_ms,
                        ..
                    } => {
                        *turn_hold_ms = turn;
                        *pitch_hold_ms = pitch;
                    }
                    Input::Mouse { .. } => unreachable!(),
                }
                let fields = fields(&g);
                assert_eq!(fields.len(), 1, "{device:?} {turn}/{pitch}: {fields:?}");
            }
        }
    }

    #[test]
    fn mouse_drag_button_follows_the_mode() {
        let mut g = game();
        g.input = Input::Mouse {
            mode: MouseMode::Drag,
            drag_button: None,
            turn_delta: 840,
            pitch_delta: 610,
            invert_vertical: false,
        };
        assert_eq!(fields(&g), ["input.dragButton"]);
        g.input = Input::Mouse {
            mode: MouseMode::Drag,
            drag_button: Some(DragButton::Right),
            turn_delta: 840,
            pitch_delta: 610,
            invert_vertical: true,
        };
        assert_eq!(validate(&g), []);
        g.input = Input::Mouse {
            mode: MouseMode::Move,
            drag_button: Some(DragButton::Left),
            turn_delta: 1,
            pitch_delta: 100_000,
            invert_vertical: false,
        };
        assert_eq!(fields(&g), ["input.dragButton"]);
    }

    #[test]
    fn mouse_deltas_are_1_to_100000() {
        let mut g = game();
        g.input = Input::Mouse {
            mode: MouseMode::Move,
            drag_button: None,
            turn_delta: 0,
            pitch_delta: 100_001,
            invert_vertical: false,
        };
        assert_eq!(fields(&g), ["input.turnDelta", "input.pitchDelta"]);
    }

    #[test]
    fn stick_deflection_is_a_tenth_to_one() {
        let mut g = game();
        for (value, ok) in [
            (0.1, true),
            (1.0, true),
            (0.099, false),
            (1.01, false),
            (f64::NAN, false),
        ] {
            g.input = Input::Gamepad {
                stick_deflection: value,
                turn_hold_ms: 100,
                pitch_hold_ms: 100,
                invert_vertical: false,
            };
            assert_eq!(validate(&g).is_empty(), ok, "{value}");
            if !ok {
                assert_eq!(fields(&g), ["input.stickDeflection"]);
            }
        }
    }

    #[test]
    fn name_uniqueness_ignores_case_and_padding() {
        assert!(validate_name_unique("Elden Ring", ["other"]).is_none());
        let error = validate_name_unique(" elden ring ", ["Elden Ring"]).unwrap();
        assert_eq!(error.field, "name");
    }
}
