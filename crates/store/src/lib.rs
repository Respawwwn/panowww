//! File system side of Panowww: everything that touches disk, paths only.
//!
//! Owns the app data folder, the Games folder, `settings.json`,
//! `local-games.json`, Session folders and their `session.json`, the PNG
//! worker, thumbnails, zip, reveal, delete and sizes. Implements the core
//! `ShotSink` port.
#![forbid(unsafe_code)]
