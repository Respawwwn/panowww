//! The Games folder on a temporary data folder: every operation of spec §5
//! *Storage* and *Identity on import*, collisions included.

use std::path::Path;

use panowww_core::game::{Game, GameError, NewGameDefaults};
use panowww_store::games::{GameEntry, GamesFolder, Import};
use panowww_store::{AppPaths, StoreError};

fn folder() -> (tempfile::TempDir, GamesFolder) {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(dir.path());
    (dir, GamesFolder::new(&paths))
}

fn file_names(folder: &GamesFolder) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder.dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn names(folder: &GamesFolder) -> Vec<String> {
    folder
        .games()
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect()
}

#[test]
fn an_empty_or_missing_folder_lists_nothing() {
    let (_dir, folder) = folder();
    assert!(!folder.dir().exists());
    assert_eq!(folder.list().unwrap(), []);
}

#[test]
fn create_writes_a_valid_file_and_makes_names_unique() {
    let (_dir, folder) = folder();
    let first = folder
        .create("Elden Ring", NewGameDefaults::default())
        .unwrap();
    let second = folder
        .create("elden ring", NewGameDefaults::default())
        .unwrap();
    assert_eq!(first.name, "Elden Ring");
    assert_eq!(second.name, "elden ring (2)");
    assert_ne!(first.id, second.id);
    assert_eq!(
        file_names(&folder),
        ["Elden Ring.panowww.json", "elden ring (2).panowww.json"]
    );
    let text = std::fs::read_to_string(folder.dir().join("Elden Ring.panowww.json")).unwrap();
    assert_eq!(Game::from_json(&text).unwrap().game, first);
}

#[test]
fn list_sorts_valid_games_by_name_ignoring_case() {
    let (_dir, folder) = folder();
    for n in ["zelda", "Anno", "Mario"] {
        folder.create(n, NewGameDefaults::default()).unwrap();
    }
    assert_eq!(names(&folder), ["Anno", "Mario", "zelda"]);
}

#[test]
fn save_rewrites_in_place_and_renames_the_file_when_the_name_changes() {
    let (_dir, folder) = folder();
    let mut game = folder.create("Draft", NewGameDefaults::default()).unwrap();
    game.rows = 8;
    let path = folder.save(&game).unwrap();
    assert!(path.ends_with("Draft.panowww.json"));
    assert_eq!(folder.find(game.id).unwrap().unwrap().0.rows, 8);

    game.name = "Final: cut?".to_owned();
    let path = folder.save(&game).unwrap();
    assert!(
        path.ends_with("Final- cut-.panowww.json"),
        "{}",
        path.display()
    );
    assert_eq!(file_names(&folder), ["Final- cut-.panowww.json"]);
    assert_eq!(folder.find(game.id).unwrap().unwrap().0.name, "Final: cut?");
}

#[test]
fn save_avoids_a_file_name_collision_on_rename() {
    let (_dir, folder) = folder();
    let other = folder.create("Taken", NewGameDefaults::default()).unwrap();
    let mut game = folder.create("Mine", NewGameDefaults::default()).unwrap();
    // A different name (the trailing dot counts) whose sanitised file name
    // collides with Taken's.
    game.name = "Taken.".to_owned();
    let path = folder.save(&game).unwrap();
    assert!(
        path.ends_with("Taken (2).panowww.json"),
        "{}",
        path.display()
    );
    assert_eq!(folder.find(other.id).unwrap().unwrap().0.name, "Taken");
}

#[test]
fn save_of_an_unknown_game_adds_it_and_an_invalid_game_is_refused() {
    let (_dir, folder) = folder();
    let game = Game::new("Fresh", NewGameDefaults::default());
    folder.save(&game).unwrap();
    assert_eq!(names(&folder), ["Fresh"]);

    let mut broken = game.clone();
    broken.rows = 0;
    let error = folder.save(&broken).unwrap_err();
    assert!(
        matches!(
            error,
            StoreError::Game {
                source: GameError::Invalid(_),
                ..
            }
        ),
        "{error}"
    );
    assert_eq!(
        folder.find(game.id).unwrap().unwrap().0.rows,
        game.rows,
        "the file is untouched"
    );
}

#[test]
fn duplicate_gets_a_new_id_and_a_copy_name() {
    let (_dir, folder) = folder();
    let game = folder
        .create("Elden Ring", NewGameDefaults::default())
        .unwrap();
    let copy = folder.duplicate(game.id).unwrap();
    assert_ne!(copy.id, game.id);
    assert_eq!(copy.name, "Elden Ring copy");
    let again = folder.duplicate(game.id).unwrap();
    assert_eq!(again.name, "Elden Ring copy (2)");
    assert_eq!(folder.games().unwrap().len(), 3);
}

#[test]
fn delete_removes_the_file_and_unknown_ids_are_errors() {
    let (_dir, folder) = folder();
    let game = folder.create("Gone", NewGameDefaults::default()).unwrap();
    folder.delete(game.id).unwrap();
    assert_eq!(folder.list().unwrap(), []);
    assert!(matches!(folder.delete(game.id), Err(StoreError::UnknownGame(id)) if id == game.id));
    assert!(matches!(
        folder.duplicate(game.id),
        Err(StoreError::UnknownGame(_))
    ));
}

#[test]
fn share_copies_the_file_as_is() {
    let (dir, folder) = folder();
    let game = folder
        .create("Share me", NewGameDefaults::default())
        .unwrap();
    let destination = dir.path().join("exported.panowww.json");
    folder.share_to(game.id, &destination).unwrap();
    assert_eq!(
        std::fs::read_to_string(&destination).unwrap(),
        std::fs::read_to_string(folder.dir().join("Share me.panowww.json")).unwrap()
    );
}

#[test]
fn damaged_files_stay_listed_with_their_errors() {
    let (_dir, folder) = folder();
    folder.create("Fine", NewGameDefaults::default()).unwrap();
    std::fs::write(folder.dir().join("broken.panowww.json"), "{ not json").unwrap();
    let mut invalid = Game::new("Invalid", NewGameDefaults::default());
    invalid.shots_per_row = 999;
    std::fs::write(folder.dir().join("invalid.panowww.json"), invalid.to_json()).unwrap();
    std::fs::write(folder.dir().join("notes.txt"), "ignored").unwrap();

    let entries = folder.list().unwrap();
    assert_eq!(entries.len(), 3);
    assert!(matches!(&entries[0], GameEntry::Game { game, .. } if game.name == "Fine"));
    let GameEntry::Damaged {
        path,
        message,
        errors,
    } = &entries[1]
    else {
        panic!("{:?}", entries[1]);
    };
    assert!(path.ends_with("broken.panowww.json"));
    assert!(message.contains("damaged"), "{message}");
    assert_eq!(errors.as_slice(), []);
    let GameEntry::Damaged { errors, .. } = &entries[2] else {
        panic!("{:?}", entries[2]);
    };
    assert_eq!(
        errors.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
        ["shotsPerRow"]
    );

    assert_eq!(names(&folder), ["Fine"], "games() skips damaged files");
    folder.delete_file(entries[1].path()).unwrap();
    assert_eq!(folder.list().unwrap().len(), 2);
}

#[test]
fn import_adds_an_unknown_id_with_a_unique_name() {
    let (dir, folder) = folder();
    folder
        .create("Elden Ring", NewGameDefaults::default())
        .unwrap();
    let incoming = Game::new("Elden Ring", NewGameDefaults::default());
    let source = dir.path().join("shared.panowww.json");
    std::fs::write(&source, incoming.to_json()).unwrap();

    let Import::Added(added) = folder.import(&source).unwrap() else {
        panic!("expected Added");
    };
    assert_eq!(added.id, incoming.id, "the id travels with the file");
    assert_eq!(added.name, "Elden Ring (2)");
    assert_eq!(
        file_names(&folder),
        ["Elden Ring (2).panowww.json", "Elden Ring.panowww.json"]
    );
}

#[test]
fn import_of_an_invalid_file_creates_nothing() {
    let (dir, folder) = folder();
    let source = dir.path().join("bad.panowww.json");
    let mut bad = Game::new("Bad", NewGameDefaults::default());
    bad.rows = 1;
    std::fs::write(&source, bad.to_json()).unwrap();
    let error = folder.import(&source).unwrap_err();
    let StoreError::Game {
        source: GameError::Invalid(errors),
        ..
    } = &error
    else {
        panic!("{error}");
    };
    assert_eq!(errors[0].field, "rows");
    assert_eq!(folder.list().unwrap(), []);

    std::fs::write(&source, "nope").unwrap();
    assert!(matches!(
        folder.import(&source),
        Err(StoreError::Game {
            source: GameError::Damaged(_),
            ..
        })
    ));
    assert!(matches!(
        folder.import(Path::new("/nowhere.panowww.json")),
        Err(StoreError::Io { .. })
    ));
}

#[test]
fn import_of_a_known_id_asks_then_replaces_or_keeps_both() {
    let (dir, folder) = folder();
    let existing = folder
        .create("Elden Ring", NewGameDefaults::default())
        .unwrap();
    let mut incoming = existing.clone();
    incoming.name = "Elden Ring – tuned".to_owned();
    incoming.rows = 9;
    let source = dir.path().join("shared.panowww.json");
    std::fs::write(&source, incoming.to_json()).unwrap();

    let Import::KnownId {
        incoming: seen,
        existing: current,
    } = folder.import(&source).unwrap()
    else {
        panic!("expected KnownId");
    };
    assert_eq!((&seen, &current), (&incoming, &existing));
    assert_eq!(names(&folder), ["Elden Ring"], "nothing written yet");

    // Replace: same id, file renamed after the new name, one Game.
    let replaced = folder.import_replace(seen.clone()).unwrap();
    assert_eq!(replaced.id, existing.id);
    assert_eq!(replaced.rows, 9);
    assert_eq!(file_names(&folder), ["Elden Ring – tuned.panowww.json"]);

    // Keep both: new id, name made unique, two Games.
    let kept = folder.import_keep_both(seen).unwrap();
    assert_ne!(kept.id, existing.id);
    assert_eq!(kept.name, "Elden Ring – tuned (2)");
    assert_eq!(folder.games().unwrap().len(), 2);
}
