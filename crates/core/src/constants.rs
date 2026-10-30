//! App constants owned by `core` (spec Appendix A). Not Game fields: a Game
//! stays shareable across machines while these may move per OS later.

use std::time::Duration;

/// Longest single relative mouse motion, in raw units.
pub const MOUSE_SUB_MOVE_MAX: u32 = 20;
/// Pause between two mouse sub-moves.
pub const MOUSE_SUB_MOVE_INTERVAL: Duration = Duration::from_millis(8);
/// Pause after pressing the drag button and before releasing it.
pub const DRAG_SETTLE: Duration = Duration::from_millis(20);
/// Cost of one Shot or Sample in the duration estimate.
pub const CAPTURE_COST: Duration = Duration::from_millis(50);
/// Longer side of the live thumbnail sent with every Shot, in pixels.
pub const THUMBNAIL_LONG_SIDE: u32 = 192;
/// JPEG quality of the live thumbnail.
pub const THUMBNAIL_JPEG_QUALITY: u8 = 70;
/// Longer side the screen-unchanged detector reduces both images to.
pub const DETECTOR_LONG_SIDE: u32 = 160;
/// Mean absolute greyscale difference, as a share of full scale, under
/// which the detector calls two images unchanged (1 %).
pub const DETECTOR_MAX_DIFFERENCE: f64 = 0.01;
/// Overlap the stitcher wants between two neighbouring Shots, in percent.
pub const OVERLAP_TARGET: std::ops::RangeInclusive<f64> = 25.0..=35.0;
/// Relative change of a fine calibration adjustment, in percent.
pub const ADJUST_FINE_PERCENT: u32 = 2;
/// Relative change of a coarse calibration adjustment, in percent.
pub const ADJUST_COARSE_PERCENT: u32 = 10;
