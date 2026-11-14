//! What a Stitch tells while it runs and once it is done (spec §16.5).

use serde::Serialize;
use ts_rs::TS;

/// The stages of a Stitch, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "stage", rename_all = "camelCase")]
#[ts(export)]
pub enum Stage {
    Read,
    Overlaps,
    #[serde(rename_all = "camelCase")]
    Align {
        row: u32,
        rows: u32,
    },
    Render,
    Blend,
    Save,
}

impl Stage {
    /// Share of the whole, measured on the spike (spec §16.8): the UI's
    /// remaining-time estimate rests on it.
    pub fn share(self) -> f32 {
        match self {
            Stage::Read => 0.07,
            Stage::Overlaps => 0.20,
            Stage::Align { .. } => 0.30,
            Stage::Render => 0.25,
            Stage::Blend => 0.15,
            Stage::Save => 0.03,
        }
    }

    /// Share of the whole done before this stage starts.
    fn before(self) -> f32 {
        match self {
            Stage::Read => 0.0,
            Stage::Overlaps => 0.07,
            Stage::Align { .. } => 0.27,
            Stage::Render => 0.57,
            Stage::Blend => 0.82,
            Stage::Save => 0.97,
        }
    }
}

/// One progress report: the stage, how far into it, how far overall.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StitchProgress {
    pub stage: Stage,
    /// 0–1 within the stage.
    pub stage_fraction: f32,
    /// 0–1 of the whole Stitch.
    pub overall: f32,
}

impl StitchProgress {
    pub fn new(stage: Stage, stage_fraction: f32) -> Self {
        let f = stage_fraction.clamp(0.0, 1.0);
        Self {
            stage,
            stage_fraction: f,
            overall: (stage.before() + stage.share() * f).clamp(0.0, 1.0),
        }
    }
}

/// A Shot named by its place in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct ShotRef {
    pub row: u32,
    pub col: u32,
}

/// Milliseconds spent in each stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StageDurations {
    pub read_ms: u32,
    pub overlaps_ms: u32,
    pub align_ms: u32,
    pub render_ms: u32,
    pub blend_ms: u32,
}

/// What was measured from the Shots (spec §16.3, §16.4): nothing in here
/// comes from the Game's numbers.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StitchReport {
    pub width: u32,
    pub height: u32,
    pub field_of_view_deg: f64,
    pub turn_step_deg: f64,
    pub pitch_step_deg: f64,
    /// Positive when Row 1 stopped short of its pole.
    pub start_pole_offset_deg: f64,
    /// Positive when a Row turned past its start heading.
    pub closing_turn_deg: f64,
    /// RMS deviation of the turn Steps from their mean.
    pub drift_deg: f64,
    /// RMS residual of the matches after the alignment, full-resolution pixels.
    pub residual_px: f64,
    /// Shots without any control points, placed from the grid.
    pub unaligned: Vec<ShotRef>,
    /// `[twin, duplicate]` pairs of identical consecutive Shots.
    pub dropped_steps: Vec<[ShotRef; 2]>,
    pub anti_ghost: bool,
    pub durations: StageDurations,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_shares_sum_to_one_and_progress_is_monotonic() {
        let all = [
            Stage::Read,
            Stage::Overlaps,
            Stage::Align { row: 1, rows: 7 },
            Stage::Render,
            Stage::Blend,
            Stage::Save,
        ];
        let sum: f32 = all.iter().map(|s| s.share()).sum();
        assert!((sum - 1.0).abs() < 1e-6);
        let mut last = -1.0;
        for s in all {
            for f in [0.0, 0.5, 1.0] {
                let p = StitchProgress::new(s, f).overall;
                assert!(p >= last - 1e-6, "{s:?} {f} went backwards");
                last = p;
            }
        }
        assert!((last - 1.0).abs() < 1e-6);
    }
}
