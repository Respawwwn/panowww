//! Pure domain logic of Panowww, with no OS code.
//!
//! Owns the Game schema, its validation and migration ([`game`]), the capture
//! plan, the Runner that drives a Session or a Test, the screen-unchanged
//! comparison and the duration estimate. Defines the ports it consumes
//! (`Camera`, `Grabber`, `Stream`, `Clock`, `ShotSink`); the `input`,
//! `capture` and `store` crates implement them. Fakes for those ports live
//! behind the `test-support` feature.
#![forbid(unsafe_code)]

pub mod game;
