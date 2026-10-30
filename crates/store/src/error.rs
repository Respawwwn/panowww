//! The store's error: every variant names the file involved, in plain
//! English, so the UI can show it as is.

use std::path::PathBuf;

use panowww_core::game::GameError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Could not {action} {}: {source}", path.display())]
    Io {
        /// `read`, `write`, `rename`, `delete`, `create`…
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{} is not valid JSON: {source}", path.display())]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// The file is not a Panowww file of the expected kind.
    #[error("{} is not a Panowww {expected} file.", path.display())]
    WrongFormat {
        path: PathBuf,
        expected: &'static str,
    },
    #[error("{} was written by a newer Panowww (format version {version}). Update Panowww to open it.", path.display())]
    NewerVersion { path: PathBuf, version: u64 },
    /// A Game file that `core` refused: damaged, newer, or invalid.
    #[error("{}: {source}", path.display())]
    Game {
        path: PathBuf,
        #[source]
        source: GameError,
    },
    #[error("No Game with id {0} on this machine.")]
    UnknownGame(Uuid),
    #[error("Could not watch the Games folder: {0}")]
    Watch(#[from] notify::Error),
    /// A Shot could not be encoded, written or read back as PNG.
    #[error("PNG {}: {message}", path.display())]
    Png { path: PathBuf, message: String },
    #[error("Could not zip {}: {message}", path.display())]
    Zip { path: PathBuf, message: String },
}

impl StoreError {
    pub(crate) fn io(
        action: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }
}

/// The port error the Runner sees: the message only.
impl From<StoreError> for panowww_core::ports::StoreError {
    fn from(error: StoreError) -> Self {
        Self::Failed(error.to_string())
    }
}
