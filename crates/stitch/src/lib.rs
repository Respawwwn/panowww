//! The Stitch (spec §16, ADR 0002): assembles a completed Session's Shots
//! into a Panorama, with no OS code and no file system.
//!
//! Interface: [`stitch`] takes a [`ShotSource`] (the Shots, fetched on
//! demand), the Session's [`Grid`], the machine's [`Limits`], a progress
//! callback and a [`CancelFlag`], and returns a [`Stitched`]: the Panorama
//! and preview pixels plus a [`StitchReport`] of what was measured from
//! the Shots. The grid is only the starting guess: the field of view, the
//! pitch of every Row, the heading of every Shot and the Steps the game
//! swallowed are all measured. Errors are [`StitchError`]. The synthetic
//! Session of the `synthetic` module lives behind the `test-support` feature.
#![forbid(unsafe_code)]

mod adjust;
mod align;
mod correlate;
mod dropped;
mod error;
mod focal;
mod geom;
mod grid;
mod levels;
mod matching;
mod place_rows;
mod render;
mod report;
#[cfg(any(test, feature = "test-support"))]
pub mod synthetic;
#[cfg(test)]
mod tests;

use std::time::Instant;

use panowww_core::ports::{CancelFlag, Frame, StoreError};

pub use error::StitchError;
pub use grid::Grid;
use render::Renderer as _;
pub use report::{ShotRef, Stage, StageDurations, StitchMeasured, StitchProgress, StitchReport};

/// Where the Shots come from: a Session folder in `store`, a synthetic
/// Session in tests. Indices run `0..count()`; positions are the Session's
/// `row` and `col`, 1-based, Row 1 at the start pole.
pub trait ShotSource: Sync {
    fn count(&self) -> usize;
    /// `(row, col)` of the Shot at `index`.
    fn position(&self, index: usize) -> (u32, u32);
    /// The Shot's pixels. Called more than once per Shot: once for the
    /// alignment, again by bands for the render.
    fn shot(&self, index: usize) -> Result<Frame, StoreError>;
}

/// What the machine allows (spec §16.5): the Stitch keeps its working set
/// under `memory_bytes`, the caller's share of the RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub memory_bytes: usize,
}

impl Default for Limits {
    /// Two gigabytes: a quarter of the smallest machine Panowww targets.
    fn default() -> Self {
        Self {
            memory_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

/// What a Stitch produces: the Panorama, its reduced preview, the report.
#[derive(Debug, Clone)]
pub struct Stitched {
    pub panorama: Frame,
    pub preview: Frame,
    pub report: StitchReport,
}

/// Stitches the Shots of `source` laid out as `grid` (spec §16.6).
///
/// Stages: Read (grey pyramids, dropped Steps), field of view, Row
/// placement, Overlaps, Align, Render, Blend. `progress` is called from the
/// stitching threads, hence `Sync`; `cancel` is checked between stages and inside the long
/// ones.
///
/// # Errors
///
/// [`StitchError::Grid`] when the Shots do not cover the grid or differ in
/// size, [`StitchError::Memory`] when the pyramids alone would exceed the
/// limits, [`StitchError::Shot`] when a Shot cannot be read,
/// [`StitchError::Cancelled`] when the flag is raised.
pub fn stitch(
    source: &dyn ShotSource,
    grid: &Grid,
    limits: &Limits,
    progress: &(dyn Fn(StitchProgress) + Sync),
    cancel: &CancelFlag,
) -> Result<Stitched, StitchError> {
    let started = Instant::now();
    grid.check(source)?;
    let pyramids = levels::Pyramids::build(source, grid, *limits, progress, cancel)?;
    let dropped = dropped::detect(&pyramids);
    for pair in &dropped.pairs {
        tracing::info!(
            twin = ?pyramids.position(pair.0),
            duplicate = ?pyramids.position(pair.1),
            "the camera did not move for that Step"
        );
    }
    if cancel.is_raised() {
        return Err(StitchError::Cancelled);
    }
    let read_ms = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
    let aligned = align::align(&pyramids, grid, &dropped, progress, cancel)?;
    tracing::info!(
        hfov_deg = aligned.measured.field_of_view,
        turn_step_deg = aligned.measured.turn_step,
        pitch_step_deg = aligned.measured.pitch_step,
        residual_px = aligned.residual_px,
        f_px = aligned.f_px,
        matches = aligned.corrs.len(),
        unaligned = ?align::shot_refs(&pyramids, &aligned.unaligned),
        read_ms,
        overlaps_ms = aligned.overlaps_ms,
        align_ms = aligned.align_ms,
        "aligned"
    );
    if cancel.is_raised() {
        return Err(StitchError::Cancelled);
    }
    progress(StitchProgress::new(Stage::Render, 0.0).with_measured(aligned.measured.into()));
    let rendered = render::CpuRenderer.render(
        &render::RenderInput {
            source,
            shot_width: pyramids.width,
            shot_height: pyramids.height,
            poses: &aligned.poses,
            f_px: aligned.f_px,
            limits: *limits,
        },
        progress,
        cancel,
    )?;
    tracing::info!(
        width = rendered.panorama.width,
        height = rendered.panorama.height,
        render_ms = rendered.render_ms,
        blend_ms = rendered.blend_ms,
        peak_estimate_mb = rendered.peak_estimate_bytes / (1024 * 1024),
        "rendered"
    );
    let report = StitchReport {
        width: rendered.panorama.width,
        height: rendered.panorama.height,
        field_of_view_deg: aligned.measured.field_of_view,
        turn_step_deg: aligned.measured.turn_step,
        pitch_step_deg: aligned.measured.pitch_step,
        start_pole_offset_deg: aligned.measured.start_pole_offset,
        closing_turn_deg: aligned.measured.closing_turn,
        drift_deg: aligned.measured.drift,
        residual_px: aligned.residual_px,
        unaligned: align::shot_refs(&pyramids, &aligned.unaligned),
        dropped_steps: dropped
            .pairs
            .iter()
            .map(|&(twin, duplicate)| {
                let refs = align::shot_refs(&pyramids, &[twin, duplicate]);
                [refs[0], refs[1]]
            })
            .collect(),
        anti_ghost: true,
        durations: StageDurations {
            read_ms,
            overlaps_ms: aligned.overlaps_ms,
            align_ms: aligned.align_ms,
            render_ms: rendered.render_ms,
            blend_ms: rendered.blend_ms,
        },
    };
    Ok(Stitched {
        panorama: rendered.panorama,
        preview: rendered.preview,
        report,
    })
}
