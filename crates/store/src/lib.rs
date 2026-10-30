//! File system side of Panowww: everything that touches disk, paths only
//! (spec §6).
//!
//! Owns the app data folder ([`AppPaths`]), the Games folder
//! ([`games::GamesFolder`]), `settings.json` ([`settings::Settings`]),
//! `local-games.json` ([`local::LocalGames`]), the file watcher that
//! re-reads a changed Game ([`watcher`]), and, from issue 8 on, Session
//! folders and their `session.json`, the PNG worker, thumbnails, zip,
//! reveal, delete and sizes. Implements the core `ShotSink` port. Every
//! JSON file is written atomically and carries a `format` / `version`
//! envelope migrated like the Game file.
#![forbid(unsafe_code)]

pub mod error;
pub mod games;
pub mod json;
pub mod local;
pub mod paths;
pub mod settings;
pub mod watcher;

pub use error::StoreError;
pub use paths::AppPaths;
