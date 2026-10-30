//! Game files: snapshots of the written form, and the files that must be
//! refused.

mod samples;

use panowww_core::game::{Device, Game, GameError, Input, MouseMode};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn sample_games_serialise_as_in_the_spec() {
    for (device, game) in samples::all() {
        insta::assert_snapshot!(device, game.to_json());
    }
}

#[test]
fn sample_games_round_trip() {
    for (_, game) in samples::all() {
        let loaded = Game::from_json(&game.to_json()).unwrap();
        assert_eq!(loaded.game, game);
        assert!(!loaded.migrated);
    }
}

#[test]
fn the_spec_example_loads() {
    let json = r#"{
      "format": "panowww-game",
      "version": 1,
      "id": "3f6c1e2a-1111-4222-8333-444444444444",
      "name": "Elden Ring – photo mode",
      "notes": "FOV 90, HUD off",
      "startPole": "nadir",
      "shotsPerRow": 13,
      "rows": 6,
      "waitAfterMoveMs": 300,
      "waitAfterShotMs": 300,
      "input": {
        "device": "mouse",
        "mode": "drag",
        "dragButton": "right",
        "turnDelta": 840,
        "pitchDelta": 610,
        "invertVertical": false
      }
    }"#;
    let loaded = Game::from_json(json).unwrap();
    assert_eq!(loaded.game.device(), Device::Mouse);
    assert!(matches!(
        loaded.game.input,
        Input::Mouse {
            mode: MouseMode::Drag,
            ..
        }
    ));
}

#[test]
fn start_pole_and_notes_are_optional_on_read() {
    let loaded = Game::from_json(&fixture("minimal")).unwrap();
    assert_eq!(loaded.game.start_pole, panowww_core::game::StartPole::Nadir);
    assert_eq!(loaded.game.notes, None);
    assert!(
        !loaded.game.to_json().contains("notes"),
        "absent notes stay absent"
    );
}

#[test]
fn another_format_is_not_a_game() {
    assert!(matches!(
        Game::from_json(&fixture("not-a-game")),
        Err(GameError::NotAGame)
    ));
    assert!(matches!(
        Game::from_json(r#"{"version": 1}"#),
        Err(GameError::NotAGame)
    ));
}

#[test]
fn a_newer_version_asks_to_update() {
    let error = Game::from_json(&fixture("newer-version")).unwrap_err();
    assert!(matches!(error, GameError::NewerVersion { version: 2 }));
    assert!(error.to_string().contains("Update Panowww"));
}

#[test]
fn unknown_versions_are_damaged() {
    assert!(matches!(
        Game::from_json(&fixture("version-zero")),
        Err(GameError::Damaged(_))
    ));
    assert!(matches!(
        Game::from_json(&fixture("missing-version")),
        Err(GameError::Damaged(_))
    ));
}

#[test]
fn unknown_fields_are_refused() {
    for name in ["unknown-field", "unknown-input-field", "unknown-device"] {
        let error = Game::from_json(&fixture(name)).unwrap_err();
        assert!(matches!(error, GameError::Damaged(_)), "{name}: {error}");
    }
}

#[test]
fn duplicate_keys_and_bad_ids_are_refused() {
    for name in ["duplicate-key", "bad-id"] {
        let error = Game::from_json(&fixture(name)).unwrap_err();
        assert!(matches!(error, GameError::Damaged(_)), "{name}: {error}");
    }
}

#[test]
fn not_json_is_damaged() {
    for text in ["", "{", "[]", "null", "\"panowww-game\""] {
        assert!(
            matches!(Game::from_json(text), Err(GameError::Damaged(_))),
            "{text:?}"
        );
    }
}

#[test]
fn out_of_bounds_values_list_every_failing_field() {
    let Err(GameError::Invalid(errors)) = Game::from_json(&fixture("invalid-values")) else {
        panic!("expected Invalid");
    };
    let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
    assert_eq!(fields, ["shotsPerRow", "rows", "input.turnHoldMs"]);
}

#[test]
fn error_messages_are_plain_english() {
    assert_eq!(
        GameError::NotAGame.to_string(),
        "This file is not a Panowww Game."
    );
    assert_eq!(
        GameError::Invalid(vec![]).to_string(),
        "This Game has 0 invalid field(s)."
    );
}
