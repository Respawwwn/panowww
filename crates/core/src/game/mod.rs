//! The Game: one shareable description of how to capture panoramas in one
//! video game (spec §5).
//!
//! A Game is a JSON file: a `format` / `version` envelope, Game-level fields
//! and an `input` block tagged by Input device. Keys are `camelCase`, durations
//! are integer milliseconds. [`mod@file`] reads and writes the file, [`mod@validate`]
//! checks the field rules, [`name`] keeps names and file names unique on a
//! machine, [`key`] names physical keys.

pub mod file;
pub mod key;
pub mod name;
pub mod validate;

pub use file::{GameError, Loaded};
pub use key::PhysicalKey;
pub use validate::{FieldError, validate};

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Value of the `format` field of every Game file.
pub const FORMAT: &str = "panowww-game";
/// Schema version this build writes. Older files are migrated up to it on
/// load; newer files are refused.
pub const CURRENT_VERSION: u32 = 1;

/// Shots per Row of a new Game.
pub const DEFAULT_SHOTS_PER_ROW: u32 = 13;
/// Rows of a new Game.
pub const DEFAULT_ROWS: u32 = 6;
/// Key hold of a new keyboard or gamepad block, in milliseconds.
pub const DEFAULT_HOLD_MS: u32 = 100;
/// Mouse units per Step of a new mouse block.
pub const DEFAULT_MOUSE_DELTA: u32 = 500;
/// Right-stick deflection of a new gamepad block.
pub const DEFAULT_STICK_DEFLECTION: f64 = 0.8;

/// One Game as stored in its `<name>.panowww.json` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct Game {
    /// Always [`FORMAT`]; any other value is not a Game file.
    pub format: GameFormat,
    /// Schema version of the file, [`CURRENT_VERSION`] once loaded.
    pub version: u32,
    /// Made at creation, kept through share and import. Keys all
    /// machine-local state such as the Capture zone.
    pub id: Uuid,
    /// 1–60 characters, trimmed, unique on the machine. Display only.
    pub name: String,
    /// Free text, at most 2000 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub notes: Option<String>,
    /// Pole a Session starts from. The vertical direction follows from it.
    #[serde(default)]
    pub start_pole: StartPole,
    /// Shots in one full horizontal turn, 3–72. The last Step closes back to
    /// the start heading.
    pub shots_per_row: u32,
    /// Rows from pole to pole, poles included, 2–36.
    pub rows: u32,
    /// Settle time after a camera Step, 0–10000 ms.
    pub wait_after_move_ms: u32,
    /// Settle time after a Shot, 0–10000 ms.
    pub wait_after_shot_ms: u32,
    /// Input device and its parameters.
    pub input: Input,
}

/// The single accepted value of the `format` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GameFormat {
    /// `"panowww-game"`.
    #[default]
    #[serde(rename = "panowww-game")]
    PanowwwGame,
}

/// Straight down or straight up: where a Session starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum StartPole {
    /// Straight down.
    #[default]
    Nadir,
    /// Straight up.
    Zenith,
}

/// How Panowww moves the in-game camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Device {
    Keyboard,
    Mouse,
    /// Windows only: a gamepad Game imports on macOS but cannot run there.
    Gamepad,
}

/// How mouse movement reaches the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum MouseMode {
    /// Plain relative mouse motion.
    Move,
    /// Mouse motion while a button is held.
    Drag,
}

/// Button held during a drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum DragButton {
    Left,
    Right,
    Middle,
}

/// The `input` block: only the fields of its device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "device", rename_all = "lowercase", deny_unknown_fields)]
#[ts(export)]
pub enum Input {
    /// Three physical keys, one per direction, each held for a while.
    #[serde(rename_all = "camelCase")]
    Keyboard {
        turn_right_key: PhysicalKey,
        look_up_key: PhysicalKey,
        look_down_key: PhysicalKey,
        /// 1–5000 ms per horizontal Step.
        turn_hold_ms: u32,
        /// 1–5000 ms per vertical Step.
        pitch_hold_ms: u32,
    },
    /// Relative mouse motion, plain or while dragging. Sub-move size and
    /// interval are app constants, not Game fields.
    #[serde(rename_all = "camelCase")]
    Mouse {
        mode: MouseMode,
        /// Required in drag mode, absent in move mode.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        drag_button: Option<DragButton>,
        /// Raw mouse units per horizontal Step, 1–100000.
        turn_delta: u32,
        /// Raw mouse units per vertical Step, 1–100000.
        pitch_delta: u32,
        invert_vertical: bool,
    },
    /// Right stick of a virtual gamepad (Windows only).
    #[serde(rename_all = "camelCase")]
    Gamepad {
        /// 0.1–1.0.
        stick_deflection: f64,
        /// 1–5000 ms per horizontal Step.
        turn_hold_ms: u32,
        /// 1–5000 ms per vertical Step.
        pitch_hold_ms: u32,
        invert_vertical: bool,
    },
}

/// The part of a new Game the user may override in Settings → New Game
/// defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewGameDefaults {
    pub start_pole: StartPole,
    pub wait_after_move_ms: u32,
    pub wait_after_shot_ms: u32,
}

impl Default for NewGameDefaults {
    fn default() -> Self {
        Self {
            start_pole: StartPole::Nadir,
            wait_after_move_ms: 300,
            wait_after_shot_ms: 300,
        }
    }
}

impl Game {
    /// A new keyboard Game with a fresh id and the spec's defaults. The name
    /// is trimmed; callers make it unique with [`name::unique`].
    pub fn new(name: &str, defaults: NewGameDefaults) -> Self {
        Self {
            format: GameFormat::PanowwwGame,
            version: CURRENT_VERSION,
            id: Uuid::new_v4(),
            name: name.trim().to_owned(),
            notes: None,
            start_pole: defaults.start_pole,
            shots_per_row: DEFAULT_SHOTS_PER_ROW,
            rows: DEFAULT_ROWS,
            wait_after_move_ms: defaults.wait_after_move_ms,
            wait_after_shot_ms: defaults.wait_after_shot_ms,
            input: Input::defaults_for(Device::Keyboard),
        }
    }

    /// The Input device of this Game.
    pub fn device(&self) -> Device {
        self.input.device()
    }
}

impl Input {
    /// The block the editor installs when the user switches to `device`.
    pub fn defaults_for(device: Device) -> Self {
        match device {
            Device::Keyboard => Self::Keyboard {
                turn_right_key: PhysicalKey::new("ArrowRight"),
                look_up_key: PhysicalKey::new("ArrowUp"),
                look_down_key: PhysicalKey::new("ArrowDown"),
                turn_hold_ms: DEFAULT_HOLD_MS,
                pitch_hold_ms: DEFAULT_HOLD_MS,
            },
            Device::Mouse => Self::Mouse {
                mode: MouseMode::Move,
                drag_button: None,
                turn_delta: DEFAULT_MOUSE_DELTA,
                pitch_delta: DEFAULT_MOUSE_DELTA,
                invert_vertical: false,
            },
            Device::Gamepad => Self::Gamepad {
                stick_deflection: DEFAULT_STICK_DEFLECTION,
                turn_hold_ms: DEFAULT_HOLD_MS,
                pitch_hold_ms: DEFAULT_HOLD_MS,
                invert_vertical: false,
            },
        }
    }

    /// The device this block belongs to.
    pub fn device(&self) -> Device {
        match self {
            Self::Keyboard { .. } => Device::Keyboard,
            Self::Mouse { .. } => Device::Mouse,
            Self::Gamepad { .. } => Device::Gamepad,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_has_the_spec_defaults() {
        let game = Game::new("  Elden Ring  ", NewGameDefaults::default());
        assert_eq!(game.name, "Elden Ring");
        assert_eq!(game.format, GameFormat::PanowwwGame);
        assert_eq!(game.version, CURRENT_VERSION);
        assert_eq!(game.start_pole, StartPole::Nadir);
        assert_eq!((game.shots_per_row, game.rows), (13, 6));
        assert_eq!(
            (game.wait_after_move_ms, game.wait_after_shot_ms),
            (300, 300)
        );
        assert_eq!(
            game.input,
            Input::Keyboard {
                turn_right_key: PhysicalKey::new("ArrowRight"),
                look_up_key: PhysicalKey::new("ArrowUp"),
                look_down_key: PhysicalKey::new("ArrowDown"),
                turn_hold_ms: 100,
                pitch_hold_ms: 100,
            }
        );
        assert_eq!(validate(&game), []);
    }

    #[test]
    fn new_game_takes_the_settings_overrides() {
        let game = Game::new(
            "x",
            NewGameDefaults {
                start_pole: StartPole::Zenith,
                wait_after_move_ms: 50,
                wait_after_shot_ms: 1000,
            },
        );
        assert_eq!(game.start_pole, StartPole::Zenith);
        assert_eq!(
            (game.wait_after_move_ms, game.wait_after_shot_ms),
            (50, 1000)
        );
    }

    #[test]
    fn two_new_games_get_distinct_ids() {
        let a = Game::new("a", NewGameDefaults::default());
        let b = Game::new("b", NewGameDefaults::default());
        assert_ne!(a.id, b.id);
        assert_eq!(a.id.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn device_defaults_are_valid_and_match_their_device() {
        for device in [Device::Keyboard, Device::Mouse, Device::Gamepad] {
            let input = Input::defaults_for(device);
            assert_eq!(input.device(), device);
            let mut game = Game::new("x", NewGameDefaults::default());
            game.input = input;
            assert_eq!(game.device(), device);
            assert!(validate(&game).is_empty(), "{device:?}");
        }
    }

    #[test]
    fn gamepad_default_deflection_is_the_spec_value() {
        let Input::Gamepad {
            stick_deflection, ..
        } = Input::defaults_for(Device::Gamepad)
        else {
            unreachable!()
        };
        assert!((stick_deflection - 0.8).abs() < f64::EPSILON);
    }
}
