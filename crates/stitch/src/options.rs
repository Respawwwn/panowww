//! The choices a Stitch is made with (spec §16.3, Options card): the Game
//! remembers them, the Panorama records them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What becomes of the Shots without control points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UnalignedShots {
    /// Placed where the grid says, so the sky stays whole.
    #[default]
    PlaceFromGrid,
    /// Left out, as PTGui does: what only they saw stays black.
    LeaveOut,
}

/// The options of one Stitch. `jpeg_quality` is not read by the Stitch
/// itself but by whoever writes the Panorama; it travels with the others
/// so a Game remembers one set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct StitchOptions {
    /// The seam finder: Shots meet along the line where they agree best,
    /// instead of halfway between their centres.
    pub anti_ghost: bool,
    pub unaligned_shots: UnalignedShots,
    /// 50 to 100.
    pub jpeg_quality: u8,
}

impl StitchOptions {
    pub const MIN_QUALITY: u8 = 50;
    pub const MAX_QUALITY: u8 = 100;

    /// The quality brought into 50–100.
    #[must_use]
    pub fn quality(&self) -> u8 {
        self.jpeg_quality
            .clamp(Self::MIN_QUALITY, Self::MAX_QUALITY)
    }
}

impl Default for StitchOptions {
    /// Anti-ghost on, unaligned Shots placed from the grid, quality 100.
    fn default() -> Self {
        Self {
            anti_ghost: true,
            unaligned_shots: UnalignedShots::PlaceFromGrid,
            jpeg_quality: Self::MAX_QUALITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_the_defaults_and_the_quality_stays_in_range() {
        let parsed: StitchOptions = serde_json::from_str(r#"{"antiGhost":false}"#).unwrap();
        assert_eq!(
            parsed,
            StitchOptions {
                anti_ghost: false,
                ..StitchOptions::default()
            }
        );
        let low = StitchOptions {
            jpeg_quality: 10,
            ..StitchOptions::default()
        };
        assert_eq!(low.quality(), 50);
        assert_eq!(
            serde_json::to_string(&StitchOptions::default()).unwrap(),
            r#"{"antiGhost":true,"unalignedShots":"placeFromGrid","jpegQuality":100}"#
        );
    }
}
