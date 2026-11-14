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

use rayon::prelude::*;

use crate::correlate::{Window, correlate};
use crate::geom::{Cam, Rot, axis, pose};
use crate::grid::Grid;
use crate::levels::Pyramids;

/// Horizontal field of view used to place the first search, and the share
/// of the predicted offset searched around it.
const GUESS_HFOV: f64 = 90.0;
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

/// One measured neighbour: where B's optical axis lands in A, in A's
/// level pixels.
struct Landing {
    pitch: f64,
    u: f64,
    v: f64,
}

/// Estimates the focal length from the horizon Row. Falls back to 90°
/// when fewer than three pairs can be measured (a flat horizon).
pub fn estimate(pyramids: &Pyramids, grid: &Grid, poses: &[Rot]) -> Focal {
    let row = grid.horizon_row();
    let level = 2u32;
    let l0 = pyramids.shot(0).level(level);
    let (lw, lh) = (l0.w as u32, l0.h as u32);
    let guess = Cam::new(Cam::f_for_hfov(f64::from(lw), GUESS_HFOV), lw, lh);
    let pairs: Vec<(usize, usize)> = pyramids
        .neighbour_pairs(grid)
        .into_iter()
        .filter(|&(i, j)| pyramids.position(i).0 == row && pyramids.position(j).0 == row)
        .collect();
    let fallback = Focal {
        f_px: Cam::f_for_hfov(f64::from(pyramids.width), GUESS_HFOV),
        ncc: -1.0,
        measured: 0,
    };
    if pairs.len() < 3 {
        return fallback;
    }
    // Where B's centre lands in A, found by correlating B's central patch
    // around the position the guess predicts, with a window wide enough
    // for any field of view between 60° and 120°.
    let landings: Vec<(Landing, f64)> = pairs
        .par_iter()
        .filter_map(|&(i, j)| {
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
                &guess,
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
        })
        .collect();
    if landings.len() < 3 {
        return fallback;
    }
    let ncc = landings.iter().map(|l| l.1).sum::<f64>() / landings.len() as f64;
    // The yaw each landing implies for a focal length, summed over the
    // Row; pairs that could not be measured count as the mean of the rest.
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
        sum * pairs.len() as f64 / landings.len() as f64
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
    let f_level = f64::midpoint(lo, hi);
    Focal {
        f_px: f_level * l0.scale,
        ncc,
        measured: landings.len(),
    }
}

fn yaw_pitch(r: &Rot) -> (f64, f64) {
    let z = axis(r);
    (z.x.atan2(z.z), z.y.clamp(-1.0, 1.0).asin())
}
