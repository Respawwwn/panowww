//! The field of view (spec §16.6): a Row is a full turn, so the pixel
//! offsets between its consecutive Shots must add up to 360° of yaw. The
//! focal length is the one that makes them. Measured on the Row nearest
//! the horizon, on the 1/2 copies; no number of the grid is needed beyond
//! "one turn", so a Step that is not the planned one changes nothing.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "level sizes become camera parameters"
)]

use panowww_core::ports::CancelFlag;
use rayon::prelude::*;

use crate::correlate::{Window, correlate};
use crate::error::StitchError;
use crate::geom::{Cam, Rot, axis, pose, yaw_pitch};
use crate::grid::{Grid, GridParams};
use crate::levels::Pyramids;

/// Fields of view tried as the guess that places the first search, 15°
/// apart: the correlation warps the neighbour through the guess, so a
/// guess far from the truth distorts the patch and lands on a wrong
/// peak (a World of Warcraft Session at 118° measured 90° from a single
/// 90° guess). The guess whose landings correlate best wins.
const GUESS_HFOVS: [f64; 6] = [60.0, 75.0, 90.0, 105.0, 120.0, 135.0];
/// The guess of the fallback, when no Row can be measured.
const GUESS_HFOV: f64 = 90.0;
/// Share of the predicted offset searched around it.
const SEARCH_SHARE: f64 = 0.35;
const HFOV_MIN: f64 = 30.0;
const HFOV_MAX: f64 = 150.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focal {
    /// Focal length in full-resolution pixels.
    pub f_px: f64,
    /// Mean correlation of the measured neighbour centres.
    pub ncc: f64,
    /// Neighbour pairs of the horizon Row that could be measured.
    pub measured: usize,
}

/// What each pair of a Row measured: a landing with its correlation, or
/// the prior yaw of a pair that could not be measured.
type Measured = Vec<Result<(Landing, f64), f64>>;

/// One measured neighbour: where B's optical axis lands in A, in A's
/// level pixels.
struct Landing {
    pitch: f64,
    u: f64,
    v: f64,
}

/// Estimates the focal length from the Row nearest the horizon, or from
/// one of its two neighbours when that one measures better: a camera that
/// starts short of its pole can point the nominal horizon Row at the sky.
/// Falls back to 90° when no Row measures three pairs.
pub fn estimate(
    pyramids: &Pyramids,
    grid: &Grid,
    poses: &[Rot],
    cancel: &CancelFlag,
) -> Result<Focal, StitchError> {
    let horizon = grid.horizon_row();
    let fallback = Focal {
        f_px: Cam::f_for_hfov(f64::from(pyramids.width), GUESS_HFOV),
        ncc: -1.0,
        measured: 0,
    };
    let on_horizon = estimate_on_row(pyramids, grid, poses, horizon, cancel)?;
    // The horizon Row's nominal pitch is the truest, so it is kept whenever
    // it measures at least half its pairs; the neighbours step in when it
    // looks at the sky.
    let enough = (grid.shots_per_row as usize / 2).max(3);
    if let Some(found) = on_horizon
        && found.measured >= enough
    {
        return Ok(found);
    }
    let mut best = on_horizon;
    for row in [horizon.saturating_sub(1), horizon + 1] {
        if row < 1 || row > grid.rows {
            continue;
        }
        if let Some(found) = estimate_on_row(pyramids, grid, poses, row, cancel)?
            && best.is_none_or(|b| found.measured as f64 * found.ncc > b.measured as f64 * b.ncc)
        {
            tracing::debug!(
                row,
                f_px = found.f_px,
                ncc = found.ncc,
                measured = found.measured,
                "field of view on a neighbouring Row"
            );
            best = Some(found);
        }
    }
    Ok(best.unwrap_or(fallback))
}

/// The estimate on one Row; `None` when fewer than three of its pairs
/// measure under any guess. The cancel flag is read between the guesses:
/// each one correlates every pair of the Row.
fn estimate_on_row(
    pyramids: &Pyramids,
    grid: &Grid,
    poses: &[Rot],
    row: u32,
    cancel: &CancelFlag,
) -> Result<Option<Focal>, StitchError> {
    let level = 2u32;
    let l0 = pyramids.shot(0).level(level);
    let (lw, lh) = (l0.w as u32, l0.h as u32);
    let pairs: Vec<(usize, usize)> = pyramids
        .neighbour_pairs(grid)
        .into_iter()
        .filter(|&(i, j)| pyramids.position(i).0 == row && pyramids.position(j).0 == row)
        .collect();
    if pairs.len() < 3 {
        return Ok(None);
    }
    // Where B's centre lands in A, found by correlating B's central patch
    // around the position a guess predicts, under each guess in turn; the
    // guess whose landings correlate best is the one near the truth. A
    // pair that cannot be measured (a landing outside A, a flat band)
    // keeps the yaw its prior says: after a dropped Step the closing pair
    // spans two Steps.
    let measured_under = |guess_hfov: f64| -> Measured {
        let guess = Cam::new(Cam::f_for_hfov(f64::from(lw), guess_hfov), lw, lh);
        pairs
            .par_iter()
            .map(|&(i, j)| {
                let (ya, _) = yaw_pitch(&poses[i]);
                let (yb, _) = yaw_pitch(&poses[j]);
                let prior = (yb - ya).rem_euclid(std::f64::consts::TAU);
                let prior = if prior > std::f64::consts::PI {
                    std::f64::consts::TAU - prior
                } else {
                    prior
                };
                measure_landing(pyramids, poses, &guess, level, i, j).ok_or(prior)
            })
            .collect()
    };
    let mean_ncc = |results: &Measured| -> (usize, f64) {
        let found: Vec<f64> = results
            .iter()
            .filter_map(|r| r.as_ref().ok().map(|l| l.1))
            .collect();
        let n = found.len();
        (
            n,
            if n == 0 {
                -1.0
            } else {
                found.iter().sum::<f64>() / n as f64
            },
        )
    };
    let mut best: Option<(f64, Measured)> = None;
    for guess_hfov in GUESS_HFOVS {
        if cancel.is_raised() {
            return Err(StitchError::Cancelled);
        }
        let results = measured_under(guess_hfov);
        let (n, ncc) = mean_ncc(&results);
        tracing::debug!(guess_hfov, measured = n, ncc, "field of view guess");
        // Support-weighted: a guess that measures one pair fewer with a
        // slightly higher correlation is the worse one.
        if n >= 3 && best.as_ref().is_none_or(|(b, _)| n as f64 * ncc > *b) {
            best = Some((n as f64 * ncc, results));
        }
    }
    let Some((_, results)) = best else {
        return Ok(None);
    };
    let Some(first) = solve(&results, lw, lh) else {
        return Ok(None);
    };
    // Measured again under the field of view just found: the warp is now
    // nearly right, so the landings are too. The 15° grid of guesses alone
    // leaves a few percent on the focal length.
    let again = measured_under(Cam::new(first.f_px, lw, lh).hfov_deg());
    // Kept only when it rests on as many landings and stays near: a Row of
    // gradients can send the second pass anywhere.
    let found = solve(&again, lw, lh)
        .filter(|f| f.measured >= first.measured && (f.f_px / first.f_px - 1.0).abs() < 0.2)
        .unwrap_or(first);
    Ok(Some(Focal {
        f_px: found.f_px * l0.scale,
        ncc: found.ncc,
        measured: found.measured,
    }))
}

/// The focal length, in level pixels, that makes the landings of
/// `results` add up to one turn, with the priors of the pairs that could
/// not be measured. `None` with fewer than three landings.
fn solve(results: &Measured, lw: u32, lh: u32) -> Option<Focal> {
    let landings: Vec<&(Landing, f64)> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
    if landings.len() < 3 {
        return None;
    }
    let ncc = landings.iter().map(|l| l.1).sum::<f64>() / landings.len() as f64;
    let prior_sum: f64 = results
        .iter()
        .filter_map(|r| r.as_ref().err().copied())
        .sum();
    // The yaw each landing implies for a focal length, plus the prior yaw of
    // the pairs that could not be measured, summed over the Row.
    let total_yaw = |f: f64| -> f64 {
        let cam = Cam::new(f, lw, lh);
        let sum: f64 = landings
            .iter()
            .map(|(l, _)| {
                let a = pose(0.0, l.pitch);
                let d = cam.ray(&a, l.u, l.v);
                d.x.atan2(d.z).abs()
            })
            .sum();
        sum + prior_sum
    };
    // Monotonic in f: a longer focal length means smaller angles. Bisect.
    let mut lo = Cam::f_for_hfov(f64::from(lw), HFOV_MAX);
    let mut hi = Cam::f_for_hfov(f64::from(lw), HFOV_MIN);
    for _ in 0..60 {
        let mid = f64::midpoint(lo, hi);
        if total_yaw(mid) > std::f64::consts::TAU {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(Focal {
        f_px: f64::midpoint(lo, hi),
        ncc,
        measured: landings.len(),
    })
}

/// Where Shot `j`'s optical axis lands in Shot `i`, and how well it
/// correlated there.
fn measure_landing(
    pyramids: &Pyramids,
    poses: &[Rot],
    guess: &Cam,
    level: u32,
    i: usize,
    j: usize,
) -> Option<(Landing, f64)> {
    {
        {
            let predicted = guess.project(&poses[i], &axis(&poses[j]))?;
            let dx = predicted.0 - guess.cx;
            let dy = predicted.1 - guess.cy;
            let radius = ((guess.h / 6.0) as i32).clamp(8, 24);
            // The window must fit inside A with its patch: small Shots get a
            // narrower search.
            let room = ((guess.w.min(guess.h) / 2.0) as i32 - radius - 2).max(2);
            let search = ((dx.abs().max(dy.abs()) * SEARCH_SHARE).ceil() as i32)
                .clamp(2, 60)
                .min(room);
            let margin = f64::from(radius + search + 1);
            let centre = (
                predicted.0.clamp(margin, guess.w - 1.0 - margin),
                predicted.1.clamp(margin, guess.h - 1.0 - margin),
            );
            // The patch of A at the predicted landing against B warped there:
            // the found shift moves the landing to where it really is.
            let peak = correlate(
                pyramids.shot(i).level(level),
                pyramids.shot(j).level(level),
                guess,
                &poses[i],
                &poses[j],
                centre,
                Window { radius, search },
            )?;
            if peak.ncc < 0.5 {
                return None;
            }
            // A's patch at `centre` matches B's content at `centre + shift`
            // under the guess's warp; B's axis therefore lands at
            // `predicted - shift` in A.
            let (_, pitch) = yaw_pitch(&poses[i]);
            Some((
                Landing {
                    pitch,
                    u: predicted.0 - peak.shift.0,
                    v: predicted.1 - peak.shift.1,
                },
                peak.ncc,
            ))
        }
    }
}

/// Beyond this pitch a Row looks at a pole: its neighbours' landings say
/// nothing about yaw.
const POLE_DEG: f64 = 75.0;

/// Re-chains the yaws of every Row away from the poles from the measured
/// landings of its consecutive Shots, closed to one full turn: the grid's
/// Step is replaced by what the Shots show, a swallowed Step by zero, a
/// pair that cannot be measured by its share of what is left of the turn.
/// The Row's heading (its first Shot) is kept.
pub fn chain_rows(pyramids: &Pyramids, grid: &Grid, params: &mut GridParams) -> usize {
    let level = 2u32;
    let l0 = pyramids.shot(0).level(level);
    let cam = Cam::new(params.f_px / l0.scale, l0.w as u32, l0.h as u32);
    let poses = params.poses();
    let n = grid.shots_per_row;
    let mut rechained = 0;
    for row in 1..=grid.rows {
        if params.pitch[(row - 1) as usize].abs() > POLE_DEG.to_radians() {
            continue;
        }
        let pairs: Vec<(usize, usize)> = (1..=n)
            .filter_map(|c| {
                Some((
                    pyramids.index_of(row, c)?,
                    pyramids.index_of(row, if c == n { 1 } else { c + 1 })?,
                ))
            })
            .collect();
        if pairs.len() != n as usize {
            continue;
        }
        let measured: Vec<Option<f64>> = pairs
            .par_iter()
            .map(|&(i, j)| {
                let (l, _) = measure_landing(pyramids, &poses, &cam, level, i, j)?;
                let a = pose(0.0, l.pitch);
                let d = cam.ray(&a, l.u, l.v);
                Some(d.x.atan2(d.z))
            })
            .collect();
        let known: f64 = measured.iter().flatten().sum();
        let missing = measured.iter().filter(|m| m.is_none()).count();
        if measured.len() - missing < 3 {
            continue;
        }
        let fill = if missing > 0 {
            (std::f64::consts::TAU - known) / missing as f64
        } else {
            0.0
        };
        let scale = if missing > 0 {
            1.0
        } else {
            std::f64::consts::TAU / known
        };
        let steps: Vec<f64> = measured
            .iter()
            .map(|m| m.map_or(fill, |v| v * scale))
            .collect();
        let first = pyramids.index_of(row, 1).expect("checked above");
        let mut yaw = params.yaw[first];
        for (k, &(_, j)) in pairs.iter().enumerate().take(n as usize - 1) {
            yaw += steps[k];
            params.yaw[j] = yaw;
        }
        rechained += 1;
    }
    rechained
}
