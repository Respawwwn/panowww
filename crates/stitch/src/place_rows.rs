//! Row placement (spec §16.6 step 3, ADR 0002): a Row may start anywhere
//! and sit anywhere. From the horizon Row outwards, each Row gets the
//! heading (over the full turn) and the pitch (over ±20°) that make it
//! agree with its already placed neighbour: candidates scored on the 1/8
//! copies over every column, the best few verified by the fine matcher's
//! accepted-match count, then refined.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    reason = "pixel coordinates and sizes move between integers and floats throughout; the placement reads as one piece"
)]

use rayon::prelude::*;

use crate::correlate::{Window, correlate};
use crate::geom::{Cam, wrap_deg};
use crate::grid::{Grid, GridParams};
use crate::levels::Pyramids;
use crate::matching::{Pass, correspondences};

/// Pitch offsets tried, degrees: ±20° in steps of 5°, then ±5° in 1°.
const PITCH_SPAN: i32 = 4;
const PITCH_STEP: f64 = 5.0;
/// Heading step of the full-turn search, degrees.
const HEADING_STEP: f64 = 3.0;
/// Beyond this pitch a Row is a pole: its heading is unobservable.
const POLE_DEG: f64 = 85.0;
/// How many distinct candidates the fine matcher verifies.
const CANDIDATES: usize = 6;

/// What one Row's placement found, degrees relative to the grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub row: u32,
    pub pitch_offset_deg: f64,
    pub heading_offset_deg: f64,
    pub ncc: f64,
}

/// Places every Row in `params`, outward from the horizon Row.
pub fn place_rows(
    pyramids: &Pyramids,
    grid: &Grid,
    params: &mut GridParams,
    progress: &(dyn Fn(f32) + Sync),
) -> Vec<Placed> {
    let l0 = pyramids.shot(0).level(8);
    let cam = Cam::new(params.f_px / l0.scale, l0.w as u32, l0.h as u32);
    // Small Shots get smaller patches, kept inside the image.
    let patch = (l0.w.min(l0.h) as i32 / 6).clamp(3, 10);
    let window = Window {
        radius: patch,
        search: 3,
    };
    let min_y = f64::from(patch + 1);
    let horizon = grid.horizon_row();
    let mut order: Vec<(u32, u32)> = Vec::new(); // (fixed, moving)
    for r in horizon..grid.rows {
        order.push((r, r + 1));
    }
    for r in (1..horizon).rev() {
        order.push((r + 1, r));
    }
    let total = order.len().max(1) as f32;
    let mut placed = Vec::new();
    for (k, (fixed, moving)) in order.into_iter().enumerate() {
        progress(k as f32 / total);
        let upward = moving > fixed;
        let (lower, upper) = if upward {
            (fixed, moving)
        } else {
            (moving, fixed)
        };
        let pairs: Vec<(usize, usize)> = (1..=grid.shots_per_row)
            .filter_map(|c| Some((pyramids.index_of(lower, c)?, pyramids.index_of(upper, c)?)))
            .collect();
        if pairs.is_empty() {
            continue;
        }
        let pole_fixed = params.pitch[(fixed - 1) as usize].abs() > POLE_DEG.to_radians();
        let base_pitch = params.pitch[(moving - 1) as usize];
        let base_yaw = params.yaw.clone();
        let apply = |g: &mut GridParams, dp: f64, dy: f64| {
            g.pitch[(moving - 1) as usize] = base_pitch + dp;
            for (i, yaw) in g.yaw.iter_mut().enumerate() {
                if pyramids.position(i).0 == moving {
                    *yaw = base_yaw[i] + dy;
                }
            }
        };
        // The median over the columns of the best patch correlation in the
        // band where the two Rows meet: one flat column cannot decide.
        let score = |dp: f64, dy: f64| -> f64 {
            let mut g = params.clone();
            apply(&mut g, dp, dy);
            let mut v: Vec<f64> = pairs
                .par_iter()
                .filter_map(|&(i, j)| {
                    let (ra, rb) = (g.pose(i), g.pose(j));
                    let (a, b) = (pyramids.shot(i).level(8), pyramids.shot(j).level(8));
                    let mut best = -1.0f64;
                    for fx in [0.3, 0.5, 0.7] {
                        for fy in [0.12, 0.22] {
                            let c = (cam.w * fx, (cam.h * fy).max(min_y));
                            if let Some(p) = correlate(a, b, &cam, &ra, &rb, c, window) {
                                best = best.max(p.ncc);
                            }
                        }
                    }
                    (best > -1.0).then_some(best)
                })
                .collect();
            if v.is_empty() {
                -1.0
            } else {
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            }
        };
        let mut all: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, score(0.0, 0.0))];
        for dp_i in -PITCH_SPAN..=PITCH_SPAN {
            let dp = f64::from(dp_i) * PITCH_STEP;
            let (mut dy, dy_end) = if pole_fixed {
                (0.0, 0.0)
            } else {
                (-180.0, 180.0 - HEADING_STEP)
            };
            while dy <= dy_end {
                all.push((dp, dy, score(dp.to_radians(), dy.to_radians())));
                dy += HEADING_STEP;
            }
        }
        all.sort_by(|a, b| b.2.total_cmp(&a.2));
        let mut candidates: Vec<(f64, f64, f64)> = Vec::new();
        for c in &all {
            if candidates.iter().all(|k| {
                wrap_deg(k.1 - c.1).abs() > 2.0 * HEADING_STEP
                    || (k.0 - c.0).abs() > PITCH_STEP - 1.0
            }) {
                candidates.push(*c);
                if candidates.len() >= CANDIDATES {
                    break;
                }
            }
        }
        // The fine matcher decides between the candidates.
        let verify = Pass {
            level: 4,
            radius: 16,
            search: 12,
        };
        let mut best = (0.0f64, 0.0f64, -1.0f64);
        let mut best_count = 0usize;
        for (dp, dy, sc) in candidates {
            let mut g = params.clone();
            apply(&mut g, dp.to_radians(), dy.to_radians());
            let count = correspondences(pyramids, &g.poses(), params.f_px, verify, &pairs).len();
            if count > best_count || (count == best_count && sc > best.2) {
                best_count = count;
                best = (dp, dy, sc);
            }
        }
        let (cp, cy) = (best.0, best.1);
        let mut dp = cp - PITCH_STEP;
        while dp <= cp + PITCH_STEP {
            let (mut dy, dy_end) = if pole_fixed {
                (cy, cy)
            } else {
                (cy - HEADING_STEP, cy + HEADING_STEP)
            };
            while dy <= dy_end {
                let sc = score(dp.to_radians(), dy.to_radians());
                if sc > best.2 {
                    best = (dp, dy, sc);
                }
                dy += 1.0;
            }
            dp += 1.0;
        }
        // The moving Row and every Row beyond it, away from the horizon.
        let beyond = |r: u32| if upward { r >= moving } else { r <= moving };
        for r in 1..=grid.rows {
            if beyond(r) {
                params.pitch[(r - 1) as usize] += best.0.to_radians();
            }
        }
        for i in 0..params.yaw.len() {
            if beyond(pyramids.position(i).0) {
                params.yaw[i] += best.1.to_radians();
            }
        }
        placed.push(Placed {
            row: moving,
            pitch_offset_deg: best.0,
            heading_offset_deg: best.1,
            ncc: best.2,
        });
    }
    progress(1.0);
    placed
}
