//! File system side of Panowww: everything that touches disk, paths only
//! (spec §6).
//!
//! Owns the app data folder ([`AppPaths`]), the Games folder
//! ([`games::GamesFolder`]), `settings.json` ([`settings::Settings`]),
//! `local-games.json` ([`local::LocalGames`]), the file watcher that
//! re-reads a changed Game ([`watcher`]), Session folders and their
//! `session.json` ([`sink::SessionSink`], the core `ShotSink`), the PNG
//! worker ([`png`]), the Sessions listing, zip, reveal and delete
//! ([`sessions`]), history thumbnails ([`thumbnails`]) and opening a
//! Session in a Stitcher ([`stitchers`]). Every JSON file
//! is written atomically and carries a `format` / `version` envelope
//! migrated like the Game file.
#![forbid(unsafe_code)]

pub mod error;
pub mod games;
pub mod json;
pub mod local;
pub mod paths;
pub mod png;
pub mod session;
pub mod sessions;
pub mod settings;
pub mod sink;
pub mod stitchers;
pub mod thumbnails;
pub mod watcher;

pub use error::StoreError;
pub use paths::AppPaths;
