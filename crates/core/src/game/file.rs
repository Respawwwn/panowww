//! Reading and writing a Game file: envelope check, migration from older
//! schema versions, then validation. The store hands us the text; `core`
//! never touches disk.

use serde::de::Error as _;
use serde_json::Value;
use thiserror::Error;

use super::{CURRENT_VERSION, FORMAT, FieldError, Game, validate};

/// Why a file did not yield a Game. The messages are the plain English the
/// UI shows.
#[derive(Debug, Error)]
pub enum GameError {
    /// The `format` field is missing or not [`FORMAT`].
    #[error("This file is not a Panowww Game.")]
    NotAGame,
    /// Written by a newer Panowww than this one.
    #[error(
        "This Game was made with a newer Panowww (format version {version}). Update Panowww to open it."
    )]
    NewerVersion { version: u64 },
    /// Not valid JSON, missing or duplicate fields, unknown fields, wrong
    /// types, a bad id…
    #[error("This Game file is damaged: {0}.")]
    Damaged(#[from] serde_json::Error),
    /// Well-formed, but one or more fields break the rules of spec §5.
    #[error("This Game has {} invalid field(s).", .0.len())]
    Invalid(Vec<FieldError>),
}

/// A Game read from a file, and whether it was migrated from an older
/// version (then the store rewrites the file).
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub game: Game,
    pub migrated: bool,
}

impl Game {
    /// Parses, migrates and validates the text of a Game file.
    pub fn from_json(json: &str) -> Result<Loaded, GameError> {
        // Peek at the envelope leniently to decide how to parse the rest.
        let value: Value = serde_json::from_str(json)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde_json::Error::custom("expected an object"))?;
        if object.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(GameError::NotAGame);
        }
        let version = object
            .get("version")
            .and_then(Value::as_u64)
            .ok_or_else(|| serde_json::Error::custom("`version` must be a positive integer"))?;
        let loaded = migrate(version, json)?;
        let errors = validate(&loaded.game);
        if errors.is_empty() {
            Ok(loaded)
        } else {
            Err(GameError::Invalid(errors))
        }
    }

    /// The text of this Game's file: pretty JSON with a trailing newline.
    ///
    /// # Panics
    ///
    /// Never: a `Game` always serialises.
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("a Game always serialises");
        json.push('\n');
        json
    }
}

/// Parses `json` as schema `version`, bringing it up to [`CURRENT_VERSION`].
/// Each past version gets an arm here; a version this build does not know
/// is refused.
fn migrate(version: u64, json: &str) -> Result<Loaded, GameError> {
    match version {
        1 => Ok(Loaded {
            game: serde_json::from_str(json)?,
            migrated: false,
        }),
        v if v > u64::from(CURRENT_VERSION) => Err(GameError::NewerVersion { version: v }),
        v => Err(GameError::Damaged(serde_json::Error::custom(format!(
            "unknown format version {v}"
        )))),
    }
}
