//! What the commands share: the data folder, the Games folder, the
//! settings and local state in memory, the selected Game, the Runner.

use std::sync::{Arc, Mutex, RwLock};

use panowww_store::AppPaths;
use panowww_store::games::GamesFolder;
use panowww_store::local::LocalGames;
use panowww_store::settings::Settings;
use uuid::Uuid;

use crate::backend::SharedBackend;
use crate::overlay::SharedOverlay;
use crate::runner::Runner;

pub struct AppState {
    pub paths: AppPaths,
    pub games: GamesFolder,
    pub settings: RwLock<Settings>,
    pub local: RwLock<LocalGames>,
    /// The Game shown in the detail area: what the Start shortcut runs.
    pub selected: Mutex<Option<Uuid>>,
    pub runner: Runner,
    pub backend: SharedBackend,
    /// Shared with the Runner; the Setup's Show zone draws the frame alone.
    pub overlay: SharedOverlay,
    pub version: String,
    /// Keeps the Games folder watch alive for the app's lifetime.
    #[allow(dead_code, reason = "held for its Drop")]
    pub watcher: Mutex<Option<panowww_store::watcher::GamesWatcher>>,
}

impl AppState {
    pub fn settings(&self) -> Settings {
        self.settings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn selected(&self) -> Option<Uuid> {
        *self
            .selected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub type SharedState = Arc<AppState>;
