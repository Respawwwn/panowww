//! Where Panowww keeps its files on this machine (spec §6). `app` resolves
//! the data folder from Tauri's `app_data_dir`; tests hand in a temporary
//! folder.

use std::path::{Path, PathBuf};

use crate::StoreError;

/// Name of the Games folder inside the data folder.
pub const GAMES_DIR: &str = "games";
/// Name of the settings file inside the data folder.
pub const SETTINGS_FILE: &str = "settings.json";
/// Name of the machine-local Game state file inside the data folder.
pub const LOCAL_GAMES_FILE: &str = "local-games.json";
/// Where history thumbnails are generated on demand.
pub const THUMBNAILS_DIR: &str = "cache/thumbnails";

/// The app data folder and the files inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    data: PathBuf,
}

impl AppPaths {
    pub fn new(data: impl Into<PathBuf>) -> Self {
        Self { data: data.into() }
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    /// `games/<name>.panowww.json` live here.
    pub fn games_dir(&self) -> PathBuf {
        self.data.join(GAMES_DIR)
    }

    pub fn settings_file(&self) -> PathBuf {
        self.data.join(SETTINGS_FILE)
    }

    pub fn local_games_file(&self) -> PathBuf {
        self.data.join(LOCAL_GAMES_FILE)
    }

    pub fn thumbnails_dir(&self) -> PathBuf {
        self.data.join(THUMBNAILS_DIR)
    }

    /// Creates the data, Games and thumbnail folders if missing.
    pub fn ensure(&self) -> Result<(), StoreError> {
        for dir in [self.games_dir(), self.thumbnails_dir()] {
            std::fs::create_dir_all(&dir).map_err(|e| StoreError::io("create", &dir, e))?;
        }
        Ok(())
    }
}

/// `~/Pictures/Panowww`, the default Sessions folder.
pub fn default_sessions_folder() -> PathBuf {
    std::env::home_dir()
        .unwrap_or_default()
        .join("Pictures")
        .join("Panowww")
}

/// `~` or `~/…` with the home folder in place of the tilde; `None` for
/// any other path. The UI's placeholder default is written that way, and
/// a literal `~` is a folder named `~` under the working directory (seen
/// on Windows, 2026-11-03).
pub fn expand_home(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    match components.next() {
        Some(std::path::Component::Normal(first)) if first == "~" => {
            Some(std::env::home_dir()?.join(components.as_path()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_live_under_the_data_folder() {
        let paths = AppPaths::new("/tmp/panowww-data");
        assert_eq!(paths.games_dir(), Path::new("/tmp/panowww-data/games"));
        assert_eq!(
            paths.settings_file(),
            Path::new("/tmp/panowww-data/settings.json")
        );
        assert_eq!(
            paths.local_games_file(),
            Path::new("/tmp/panowww-data/local-games.json")
        );
        assert_eq!(
            paths.thumbnails_dir(),
            Path::new("/tmp/panowww-data/cache/thumbnails")
        );
    }

    #[test]
    fn a_tilde_expands_to_the_home_folder_and_nothing_else_does() {
        let home = std::env::home_dir().unwrap();
        assert_eq!(
            expand_home(Path::new("~/Pictures/Panowww")),
            Some(home.join("Pictures").join("Panowww"))
        );
        assert_eq!(expand_home(Path::new("~")), Some(home.clone()));
        assert_eq!(expand_home(&home.join("Pictures")), None);
        assert_eq!(expand_home(Path::new("Pictures/~/x")), None);
    }

    #[test]
    fn ensure_creates_the_folders() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().join("data"));
        paths.ensure().unwrap();
        assert!(paths.games_dir().is_dir());
        assert!(paths.thumbnails_dir().is_dir());
        paths.ensure().unwrap();
    }

    #[test]
    fn default_sessions_folder_is_pictures_panowww() {
        let folder = default_sessions_folder();
        assert!(folder.ends_with("Pictures/Panowww"), "{}", folder.display());
    }
}
