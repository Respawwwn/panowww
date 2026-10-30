//! The Games folder (spec §5 *Storage* and *Identity on import*): one
//! `<name>.panowww.json` per Game, the file name following the name and
//! never being the identity. Damaged files stay listed with their errors.

use std::path::{Path, PathBuf};

use panowww_core::game::{FieldError, Game, GameError, NewGameDefaults, name};
use uuid::Uuid;

use crate::StoreError;
use crate::json;
use crate::paths::AppPaths;

/// One file of the Games folder.
#[derive(Debug, Clone, PartialEq)]
pub enum GameEntry {
    Game {
        game: Game,
        path: PathBuf,
    },
    /// A file `core` refused. Listed as *Damaged* with Reveal and Delete,
    /// re-read when it changes on disk.
    Damaged {
        path: PathBuf,
        /// The error in plain English.
        message: String,
        /// One per failing field when the file parsed but broke a rule.
        errors: Vec<FieldError>,
    },
}

impl GameEntry {
    pub fn path(&self) -> &Path {
        match self {
            Self::Game { path, .. } | Self::Damaged { path, .. } => path,
        }
    }

    pub fn game(&self) -> Option<&Game> {
        match self {
            Self::Game { game, .. } => Some(game),
            Self::Damaged { .. } => None,
        }
    }
}

/// What importing a file led to.
#[derive(Debug, Clone, PartialEq)]
pub enum Import {
    /// Unknown id: the Game was added, its name made unique.
    Added(Game),
    /// A Game with this id exists; nothing was written. The caller asks
    /// the user and calls [`GamesFolder::import_replace`] or
    /// [`GamesFolder::import_keep_both`].
    KnownId { incoming: Game, existing: Game },
}

/// The Games folder and every operation on it. Stateless: each operation
/// reads the folder, so changes made by hand are seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamesFolder {
    dir: PathBuf,
}

impl GamesFolder {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            dir: paths.games_dir(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Every `.panowww.json` file, valid Games first sorted by name, then
    /// damaged files sorted by file name. A Game from an older format is
    /// migrated and rewritten on the way.
    pub fn list(&self) -> Result<Vec<GameEntry>, StoreError> {
        let mut entries = Vec::new();
        for path in self.files()? {
            entries.push(Self::read_entry(&path)?);
        }
        entries.sort_by_cached_key(|entry| match entry {
            GameEntry::Game { game, .. } => (0, game.name.to_lowercase()),
            GameEntry::Damaged { path, .. } => (
                1,
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase(),
            ),
        });
        Ok(entries)
    }

    /// The valid Games only.
    pub fn games(&self) -> Result<Vec<Game>, StoreError> {
        Ok(self
            .list()?
            .into_iter()
            .filter_map(|e| e.game().cloned())
            .collect())
    }

    /// The Game with `id` and its file.
    pub fn find(&self, id: Uuid) -> Result<Option<(Game, PathBuf)>, StoreError> {
        Ok(self.list()?.into_iter().find_map(|entry| match entry {
            GameEntry::Game { game, path } if game.id == id => Some((game, path)),
            _ => None,
        }))
    }

    /// A new keyboard Game with the defaults, named `name` made unique,
    /// written to its file.
    pub fn create(&self, name: &str, defaults: NewGameDefaults) -> Result<Game, StoreError> {
        let names = self.names()?;
        let game = Game::new(&name::unique(name, &names), defaults);
        self.write_new(&game)?;
        Ok(game)
    }

    /// Writes `game` over its file, renaming the file when the name
    /// changed. A Game that is not in the folder yet gets a new file.
    /// Refuses an invalid Game so no broken file is ever written.
    pub fn save(&self, game: &Game) -> Result<PathBuf, StoreError> {
        let Some((_, current)) = self.find(game.id)? else {
            return self.write_new(game);
        };
        Self::check_valid(game, &current)?;
        let wanted = name::file_name(&game.name, &self.other_files(&current)?);
        let target = self.dir.join(&wanted);
        if target != current {
            std::fs::rename(&current, &target)
                .map_err(|e| StoreError::io("rename", &current, e))?;
        }
        json::write_atomic(&target, &game.to_json())?;
        Ok(target)
    }

    /// A copy of `id` with a new id and the name `<name> copy`, made
    /// unique. Its machine-local state (the zone) is not copied.
    pub fn duplicate(&self, id: Uuid) -> Result<Game, StoreError> {
        let (mut game, _) = self.find(id)?.ok_or(StoreError::UnknownGame(id))?;
        game.id = Uuid::new_v4();
        game.name = name::copy_name(&game.name, &self.names()?);
        self.write_new(&game)?;
        Ok(game)
    }

    /// Removes the file of `id`. The caller also forgets its local state.
    pub fn delete(&self, id: Uuid) -> Result<(), StoreError> {
        let (_, path) = self.find(id)?.ok_or(StoreError::UnknownGame(id))?;
        self.delete_file(&path)
    }

    /// Removes one file of the folder, damaged or not.
    pub fn delete_file(&self, path: &Path) -> Result<(), StoreError> {
        std::fs::remove_file(path).map_err(|e| StoreError::io("delete", path, e))
    }

    /// Copies the file of `id` to `destination`, for *Share*.
    pub fn share_to(&self, id: Uuid, destination: &Path) -> Result<(), StoreError> {
        let (_, path) = self.find(id)?.ok_or(StoreError::UnknownGame(id))?;
        std::fs::copy(&path, destination)
            .map(|_| ())
            .map_err(|e| StoreError::io("copy to", destination, e))
    }

    /// Reads a Game file from anywhere. An invalid file is refused with its
    /// errors and nothing is created; an unknown id is added with a unique
    /// name; a known id is reported for the user to decide.
    pub fn import(&self, source: &Path) -> Result<Import, StoreError> {
        let text =
            std::fs::read_to_string(source).map_err(|e| StoreError::io("read", source, e))?;
        let incoming = Game::from_json(&text)
            .map_err(|source_error| StoreError::Game {
                path: source.to_owned(),
                source: source_error,
            })?
            .game;
        if let Some((existing, _)) = self.find(incoming.id)? {
            return Ok(Import::KnownId { incoming, existing });
        }
        Ok(Import::Added(self.add_unique(incoming)?))
    }

    /// *Replace*: the incoming Game takes over the existing file of its id,
    /// keeping the machine-local zone since the id is unchanged.
    pub fn import_replace(&self, incoming: Game) -> Result<Game, StoreError> {
        let (_, current) = self
            .find(incoming.id)?
            .ok_or(StoreError::UnknownGame(incoming.id))?;
        let mut game = incoming;
        let others: Vec<String> = self
            .games()?
            .into_iter()
            .filter(|g| g.id != game.id)
            .map(|g| g.name)
            .collect();
        game.name = name::unique(&game.name, &others);
        let _ = current;
        self.save(&game)?;
        Ok(game)
    }

    /// *Keep both*: the incoming Game gets a new id and a unique name, and
    /// no zone.
    pub fn import_keep_both(&self, incoming: Game) -> Result<Game, StoreError> {
        let mut game = incoming;
        game.id = Uuid::new_v4();
        self.add_unique(game)
    }

    /// Reads one file as the UI lists it.
    fn read_entry(path: &Path) -> Result<GameEntry, StoreError> {
        let text = std::fs::read_to_string(path).map_err(|e| StoreError::io("read", path, e))?;
        match Game::from_json(&text) {
            Ok(loaded) => {
                if loaded.migrated {
                    json::write_atomic(path, &loaded.game.to_json())?;
                }
                Ok(GameEntry::Game {
                    game: loaded.game,
                    path: path.to_owned(),
                })
            }
            Err(error) => {
                let errors = match &error {
                    GameError::Invalid(errors) => errors.clone(),
                    _ => Vec::new(),
                };
                Ok(GameEntry::Damaged {
                    path: path.to_owned(),
                    message: error.to_string(),
                    errors,
                })
            }
        }
    }

    /// Adds `game` with its name made unique among the folder's Games.
    fn add_unique(&self, mut game: Game) -> Result<Game, StoreError> {
        game.name = name::unique(&game.name, &self.names()?);
        self.write_new(&game)?;
        Ok(game)
    }

    /// Writes `game` to a fresh, collision-free file.
    fn write_new(&self, game: &Game) -> Result<PathBuf, StoreError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| StoreError::io("create", &self.dir, e))?;
        let path = self
            .dir
            .join(name::file_name(&game.name, &self.file_names()?));
        Self::check_valid(game, &path)?;
        json::write_atomic(&path, &game.to_json())?;
        Ok(path)
    }

    fn check_valid(game: &Game, path: &Path) -> Result<(), StoreError> {
        let errors = panowww_core::game::validate(game);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(StoreError::Game {
                path: path.to_owned(),
                source: GameError::Invalid(errors),
            })
        }
    }

    /// Every `.panowww.json` path in the folder; none when the folder does
    /// not exist yet.
    fn files(&self) -> Result<Vec<PathBuf>, StoreError> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(StoreError::io("read", &self.dir, e)),
        };
        let mut files = Vec::new();
        for entry in entries {
            let path = entry
                .map_err(|e| StoreError::io("read", &self.dir, e))?
                .path();
            let is_game_file = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(name::FILE_SUFFIX) && !n.starts_with('.'));
            if is_game_file && path.is_file() {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    fn file_names(&self) -> Result<Vec<String>, StoreError> {
        Ok(self
            .files()?
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect())
    }

    /// The file names other than `except`, for a rename.
    fn other_files(&self, except: &Path) -> Result<Vec<String>, StoreError> {
        Ok(self
            .files()?
            .iter()
            .filter(|p| p.as_path() != except)
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect())
    }

    /// The names of the valid Games.
    fn names(&self) -> Result<Vec<String>, StoreError> {
        Ok(self.games()?.into_iter().map(|g| g.name).collect())
    }
}
