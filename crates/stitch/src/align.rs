//! The alignment, end to end (spec §16.6 steps 2 to 5): field of view, Row
//! placement, matching passes from coarse to fine with the gimbal
//! adjustment between them, the free per-Shot refinement, and what the
//! poses say about the capture.

#![allow(
    clippy::cast_precision_loss,
    reason = "counts become means and fractions"
)]

use std::time::Instant;

use panowww_core::game::StartPole;
use panowww_core::ports::CancelFlag;

use crate::adjust::{bundle_adjust, refine_shots};
use crate::dropped::Dropped;
use crate::error::StitchError;
use crate::focal;
use crate::geom::{Cam, Rot, wrap_deg, yaw_pitch};
use crate::grid::{Grid, GridParams};
use crate::levels::Pyramids;
use crate::matching::{Corr, Pass, correspondences, residual_px};
use crate::place_rows::place_rows;
use crate::report::{ShotRef, Stage, StitchMeasured, StitchProgress};

/// Matching passes with the gimbal adjustment after each (spec §16.8).
const GIMBAL_PASSES: [Pass; 4] = [
    Pass {
        level: 8,
        radius: 12,
        search: 30,
    },
    Pass {
        level: 4,
        radius: 16,
        search: 16,
    },
    Pass {
        level: 2,
        radius: 32,
        search: 10,
    },
    Pass {
        level: 2,
        radius: 32,
        search: 4,
    },
];
/// Matching passes with the free per-Shot refinement after each.
const FREE_PASSES: [Pass; 2] = [
    Pass {
        level: 2,
        radius: 32,
        search: 4,
    },
    Pass {
        level: 2,
        radius: 32,
        search: 3,
    },
];

/// The aligned Session.
#[derive(Debug, Clone)]
pub struct Alignment {
    pub f_px: f64,
    pub poses: Vec<Rot>,
    pub corrs: Vec<Corr>,
    /// Shots without a single accepted match, kept at their grid pose.
    pub unaligned: Vec<usize>,
    pub residual_px: f64,
    pub measured: Measured,
    pub overlaps_ms: u32,
    pub align_ms: u32,
}

/// What the poses say about the capture, degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured {
    pub field_of_view: f64,
    pub turn_step: f64,
    pub pitch_step: f64,
    pub start_pole_offset: f64,
    pub closing_turn: f64,
    pub drift: f64,
}

impl From<Measured> for StitchMeasured {
    fn from(m: Measured) -> Self {
        Self {
            field_of_view_deg: m.field_of_view,
            turn_step_deg: Some(m.turn_step),
            pitch_step_deg: Some(m.pitch_step),
            start_pole_offset_deg: Some(m.start_pole_offset),
            closing_turn_deg: Some(m.closing_turn),
            drift_deg: Some(m.drift),
        }
    }
}

/// Aligns the Shots of `pyramids` laid out as `grid`, starting from the
/// grid prior corrected for `dropped` Steps.
pub fn align(
    pyramids: &Pyramids,
    grid: &Grid,
    dropped: &Dropped,
    progress: &(dyn Fn(StitchProgress) + Sync),
    cancel: &CancelFlag,
) -> Result<Alignment, StitchError> {
    let started = Instant::now();
    let total = pyramids.count();
    let anchor = pyramids.index_of(grid.horizon_row(), 1).unwrap_or(0);
    let mut params = place(pyramids, grid, dropped, progress);
    if cancel.is_raised() {
        return Err(StitchError::Cancelled);
    }
    let pairs = pyramids.neighbour_pairs(grid);
    let passes = GIMBAL_PASSES.len() as f32;
    let mut checked = false;
    for (k, pass) in GIMBAL_PASSES.iter().enumerate() {
        progress(StitchProgress::new(
            Stage::Overlaps,
            0.5 + 0.5 * k as f32 / passes,
        ));
        let l = pyramids.shot(0).level(pass.level);
        if pass.fitted(l.w, l.h).is_none() {
            continue;
        }
        let corrs = correspondences(pyramids, &params.poses(), params.f_px, *pass, &pairs);
        if cancel.is_raised() {
            return Err(StitchError::Cancelled);
        }
        // Too few links is judged on the first pass at a quarter resolution
        // or finer: the eighth may be too coarse for small Shots.
        if !checked && pass.level <= 4 {
            check_linked(pyramids, &corrs, total)?;
            checked = true;
        }
        let fit = bundle_adjust(&mut params, &corrs, anchor, pyramids.width, pyramids.height);
        tracing::debug!(
            pass = k,
            matches = corrs.len(),
            rms_px = fit.rms_px,
            iterations = fit.iterations,
            "gimbal pass"
        );
    }
    if !checked {
        return Err(StitchError::TooFewLinked {
            linked: 0,
            total,
            unlinked: shot_refs(pyramids, &(0..total).collect::<Vec<_>>()),
        });
    }
    let overlaps_ms = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
    let align_started = Instant::now();
    let cam = Cam::new(params.f_px, pyramids.width, pyramids.height);
    let mut poses = params.poses();
    {
        let snapshot = measure(pyramids, grid, dropped, &poses, &cam);
        progress(
            StitchProgress::new(
                Stage::Align {
                    row: 1,
                    rows: grid.rows,
                },
                0.0,
            )
            .with_measured(snapshot.into()),
        );
    }
    let corrs = free_passes(
        pyramids, grid, &pairs, &mut poses, &cam, anchor, progress, cancel,
    )?;
    let unaligned = unlinked(&corrs, total);
    let measured = measure(pyramids, grid, dropped, &poses, &cam);
    Ok(Alignment {
        f_px: params.f_px,
        residual_px: residual_px(&corrs, &poses, &cam),
        poses,
        corrs,
        unaligned,
        measured,
        overlaps_ms,
        align_ms: u32::try_from(align_started.elapsed().as_millis()).unwrap_or(u32::MAX),
    })
}

/// Steps 2 and 3 (spec §16.6): the field of view, the Rows re-chained
/// from their measured Steps, then every Row placed outward from the
/// horizon, with the measurements reported as they become known.
fn place(
    pyramids: &Pyramids,
    grid: &Grid,
    dropped: &Dropped,
    progress: &(dyn Fn(StitchProgress) + Sync),
) -> GridParams {
    let mut params = GridParams::nominal(pyramids, grid, &dropped.drops, 1.0);
    let found = focal::estimate(pyramids, grid, &params.poses());
    params.f_px = found.f_px;
    let rechained = focal::chain_rows(pyramids, grid, &mut params);
    tracing::debug!(rechained, "Rows re-chained from their measured Steps");
    tracing::debug!(
        f_px = found.f_px,
        ncc = found.ncc,
        measured = found.measured,
        "field of view"
    );
    // The field of view is known: the card shows it while the Rows are
    // placed; the Steps and poles follow once they are.
    progress(
        StitchProgress::new(Stage::Overlaps, 0.05).with_measured(StitchMeasured {
            field_of_view_deg: Cam::new(params.f_px, pyramids.width, pyramids.height).hfov_deg(),
            turn_step_deg: None,
            pitch_step_deg: None,
            start_pole_offset_deg: None,
            closing_turn_deg: None,
            drift_deg: None,
        }),
    );
    let placed = place_rows(pyramids, grid, &mut params, &|f| {
        progress(StitchProgress::new(Stage::Overlaps, 0.05 + 0.45 * f));
    });
    for p in &placed {
        tracing::debug!(
            row = p.row,
            pitch = p.pitch_offset_deg,
            heading = p.heading_offset_deg,
            ncc = p.ncc,
            "Row placed"
        );
    }
    let cam = Cam::new(params.f_px, pyramids.width, pyramids.height);
    let snapshot = measure(pyramids, grid, dropped, &params.poses(), &cam);
    progress(StitchProgress::new(Stage::Overlaps, 0.5).with_measured(snapshot.into()));
    params
}

/// The Align stage proper: matching rounds with a free small rotation per
/// Shot after each, progress reported Row by Row.
#[allow(clippy::too_many_arguments, reason = "one stage, its inputs named")]
fn free_passes(
    pyramids: &Pyramids,
    grid: &Grid,
    pairs: &[(usize, usize)],
    poses: &mut [Rot],
    cam: &Cam,
    anchor: usize,
    progress: &(dyn Fn(StitchProgress) + Sync),
    cancel: &CancelFlag,
) -> Result<Vec<Corr>, StitchError> {
    let rows = grid.rows;
    let mut corrs = Vec::new();
    for (k, pass) in FREE_PASSES.iter().enumerate() {
        for row in 1..=rows {
            progress(StitchProgress::new(
                Stage::Align { row, rows },
                (k as f32 * rows as f32 + (row - 1) as f32)
                    / (FREE_PASSES.len() as f32 * rows as f32),
            ));
        }
        corrs = correspondences(pyramids, poses, cam.f, *pass, pairs);
        if cancel.is_raised() {
            return Err(StitchError::Cancelled);
        }
        let fit = refine_shots(poses, &corrs, anchor, cam);
        tracing::debug!(
            pass = k,
            matches = corrs.len(),
            rms_px = fit.rms_px,
            iterations = fit.iterations,
            "free pass"
        );
    }
    progress(StitchProgress::new(Stage::Align { row: rows, rows }, 1.0));
    Ok(corrs)
}

/// Shots that no accepted match touches.
fn unlinked(corrs: &[Corr], total: usize) -> Vec<usize> {
    let mut linked = vec![false; total];
    for c in corrs {
        linked[c.ia] = true;
        linked[c.ib] = true;
    }
    (0..total).filter(|&i| !linked[i]).collect()
}

/// Fails when fewer than half the Shots (and fewer than three) are linked:
/// the field of view cannot be held and the sphere cannot be placed.
fn check_linked(pyramids: &Pyramids, corrs: &[Corr], total: usize) -> Result<(), StitchError> {
    let unlinked = unlinked(corrs, total);
    let linked = total - unlinked.len();
    if linked < 3 || linked * 2 < total {
        return Err(StitchError::TooFewLinked {
            linked,
            total,
            unlinked: shot_refs(pyramids, &unlinked),
        });
    }
    Ok(())
}

pub fn shot_refs(pyramids: &Pyramids, indices: &[usize]) -> Vec<ShotRef> {
    indices
        .iter()
        .map(|&i| {
            let (row, col) = pyramids.position(i);
            ShotRef { row, col }
        })
        .collect()
}

/// Steps, poles and closing turns read off the poses. A dropped Step is
/// left out of the Steps, and its Row out of the closing turns.
fn measure(
    pyramids: &Pyramids,
    grid: &Grid,
    dropped: &Dropped,
    poses: &[Rot],
    cam: &Cam,
) -> Measured {
    let yp: Vec<(f64, f64)> = poses
        .iter()
        .map(|r| {
            let (y, p) = yaw_pitch(r);
            (y.to_degrees(), p.to_degrees())
        })
        .collect();
    let n = grid.shots_per_row;
    // Turn Steps on the Rows away from the poles; the closing Step apart.
    let mut steps = Vec::new();
    let mut closings = Vec::new();
    for row in 1..=grid.rows {
        if yp
            .iter()
            .zip(pyramids.shots())
            .filter(|(_, s)| s.row == row)
            .map(|(v, _)| v.1.abs())
            .fold(0.0, f64::max)
            > 75.0
        {
            continue;
        }
        let mut row_steps = Vec::new();
        let mut row_dropped = false;
        for col in 1..n {
            if let (Some(i), Some(j)) =
                (pyramids.index_of(row, col), pyramids.index_of(row, col + 1))
            {
                if dropped.pairs.contains(&(i, j)) {
                    row_dropped = true;
                    continue;
                }
                row_steps.push(wrap_deg(yp[j].0 - yp[i].0));
            }
        }
        if let (Some(i), Some(j)) = (pyramids.index_of(row, n), pyramids.index_of(row, 1))
            && !row_steps.is_empty()
            && !row_dropped
        {
            let closing = wrap_deg(yp[j].0 - yp[i].0);
            let mean = row_steps.iter().sum::<f64>() / row_steps.len() as f64;
            closings.push(closing - mean);
        }
        steps.extend(row_steps);
    }
    let turn_step = steps.iter().sum::<f64>() / steps.len().max(1) as f64;
    let drift = (steps.iter().map(|d| (d - turn_step).powi(2)).sum::<f64>()
        / steps.len().max(1) as f64)
        .sqrt();
    let mut pitches = Vec::new();
    for row in 1..grid.rows {
        for col in 1..=n {
            if let (Some(i), Some(j)) =
                (pyramids.index_of(row, col), pyramids.index_of(row + 1, col))
            {
                pitches.push((yp[j].1 - yp[i].1).abs());
            }
        }
    }
    let pitch_step = pitches.iter().sum::<f64>() / pitches.len().max(1) as f64;
    let row1: Vec<f64> = pyramids
        .shots()
        .iter()
        .zip(&yp)
        .filter(|(s, _)| s.row == 1)
        .map(|(_, v)| v.1)
        .collect();
    let mean_row1 = row1.iter().sum::<f64>() / row1.len().max(1) as f64;
    let start_pole_offset = match grid.start_pole {
        StartPole::Nadir => mean_row1 + 90.0,
        StartPole::Zenith => 90.0 - mean_row1,
    };
    Measured {
        field_of_view: cam.hfov_deg(),
        turn_step,
        pitch_step,
        start_pole_offset,
        closing_turn: closings.iter().sum::<f64>() / closings.len().max(1) as f64,
        drift,
    }
}
