//! Windows: Windows Graphics Capture (spec §9). One capture session per run
//! on the zone's display, kept alive for the whole run; Shots and Samples
//! are the first frame whose timestamp is after the move, cropped to the
//! zone on the CPU; the Display preview is a downscaled copy of one frame in
//! N, encoded on the capture thread. No cursor, no yellow border (verified
//! 2026-11-01 without a package manifest), SDR. Our own windows are hidden
//! from every capture with `WDA_EXCLUDEFROMCAPTURE` ([`exclusion`]).
//!
//! Coordinates are physical pixels of the virtual screen, origin at the
//! top-left of the primary display; a Capture zone is display-local.

pub mod displays;
pub mod exclusion;
mod wgc;

pub use displays::{DisplayEntry, find_display, list_displays, set_process_dpi_aware};
pub use exclusion::exclude_from_capture;
pub use wgc::{WgcGrabber, WgcStream};
