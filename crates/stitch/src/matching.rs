//! The Overlaps stage (spec §16.6 step 4): matches between grid
//! neighbours by patch correlation in a common view, up to twelve centres
//! spread over the overlap, the patch adapted to thin bands, outliers
//! rejected against the pair's median shift.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names,
    reason = "pixel coordinates and sizes move between integers and floats throughout, and image maths reads better with short names"
)]

use rayon::prelude::*;

use crate::correlate::{Window, correlate, texture};
use crate::geom::{Cam, Rot};
use crate::levels::Pyramids;

/// Correlation below which a peak is not a match.
pub const MIN_NCC: f64 = 0.45;

/// A peak kept for a pair: the centre in A, the shift, the correlation.
type Found = ((f64, f64), (f64, f64), f64);

/// A match: pixel `a` of Shot `ia` sees what pixel `b` of Shot `ib` sees,
/// both in full-resolution pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corr {
    pub ia: usize,
    pub ib: usize,
    pub a: (f64, f64),
    pub b: (f64, f64),
    pub ncc: f64,
}

/// One matching pass: which copies, which patch, which shifts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pass {
    pub level: u32,
    pub radius: i32,
    pub search: i32,
}

/// Smallest level side a pass can work on: below it the overlap holds too
/// few patches to mean anything.
const MIN_LEVEL_SIDE: usize = 64;

impl Pass {
    /// The pass sized for a level: small Shots get smaller patches and a
    /// narrower search rather than none; a level under 32 px is skipped.
    pub fn fitted(self, w: usize, h: usize) -> Option<Pass> {
        let side = w.min(h) as i32;
        if side < MIN_LEVEL_SIDE as i32 {
            return None;
        }
        Some(Pass {
            level: self.level,
            radius: self.radius.min(side / 6).max(4),
            search: self.search.min(side / 4).max(2),
        })
    }
}

/// Matches for `pairs` of Shot indices under `poses` and the focal length
/// `f_px` (full-resolution pixels), on the copies `pass` names. Empty when
/// the level is too small for the pass.
pub fn correspondences(
    pyramids: &Pyramids,
    poses: &[Rot],
    f_px: f64,
    pass: Pass,
    pairs: &[(usize, usize)],
) -> Vec<Corr> {
    let l0 = pyramids.shot(0).level(pass.level);
    let Some(pass) = pass.fitted(l0.w, l0.h) else {
        return Vec::new();
    };
    let scale = l0.scale;
    let cam = Cam::new(f_px / scale, l0.w as u32, l0.h as u32);
    let margin = f64::from(pass.radius + 2);
    let margin_b = f64::from(pass.radius / 2);
    pairs
        .par_iter()
        .flat_map_iter(|&(ia, ib)| {
            let (a, b) = (
                pyramids.shot(ia).level(pass.level),
                pyramids.shot(ib).level(pass.level),
            );
            let (ra, rb) = (&poses[ia], &poses[ib]);
            // The overlap: pixels of A whose ray lands well inside B.
            let mut pts: Vec<(f64, f64)> = Vec::new();
            let step = f64::from((cam.w.min(cam.h) as i32 / 10).clamp(4, 12));
            let mut y = margin;
            while y < cam.h - margin {
                let mut x = margin;
                while x < cam.w - margin {
                    let d = cam.ray(ra, x, y);
                    if cam
                        .project(rb, &d)
                        .is_some_and(|uv| cam.inside(uv, margin_b))
                    {
                        pts.push((x, y));
                    }
                    x += step;
                }
                y += step;
            }
            if pts.len() < 6 {
                return Vec::new();
            }
            let (minx, maxx) = pts
                .iter()
                .fold((f64::MAX, f64::MIN), |m, p| (m.0.min(p.0), m.1.max(p.0)));
            let (miny, maxy) = pts
                .iter()
                .fold((f64::MAX, f64::MIN), |m, p| (m.0.min(p.1), m.1.max(p.1)));
            // A thin overlap band (sparse grids) gets smaller patches rather than none.
            let narrow = (maxx - minx).min(maxy - miny);
            let radius = ((narrow / 2.0) as i32).clamp(pass.radius.min(10), pass.radius);
            let centres = centres(a, &pts, (minx, miny, maxx, maxy), radius);
            let window = Window {
                radius,
                search: pass.search,
            };
            let mut found: Vec<Found> = centres
                .into_iter()
                .filter_map(|c| {
                    let peak = correlate(a, b, &cam, ra, rb, c, window)?;
                    (peak.ncc >= MIN_NCC).then_some((c, peak.shift, peak.ncc))
                })
                .collect();
            // Outliers: shifts far from the pair's median are false peaks on
            // repetitive texture.
            if found.len() >= 3 {
                let mut xs: Vec<f64> = found.iter().map(|f| f.1.0).collect();
                let mut ys: Vec<f64> = found.iter().map(|f| f.1.1).collect();
                xs.sort_by(f64::total_cmp);
                ys.sort_by(f64::total_cmp);
                let (mx, my) = (xs[xs.len() / 2], ys[ys.len() / 2]);
                let tol = (f64::from(pass.search) * 0.5).max(2.0);
                found.retain(|f| (f.1.0 - mx).abs() <= tol && (f.1.1 - my).abs() <= tol);
            }
            found
                .into_iter()
                .filter_map(|(c, shift, ncc)| {
                    let d = cam.ray(ra, c.0 + shift.0, c.1 + shift.1);
                    let bpx = cam.project(rb, &d)?;
                    Some(Corr {
                        ia,
                        ib,
                        a: (c.0 * scale, c.1 * scale),
                        b: (bpx.0 * scale, bpx.1 * scale),
                        ncc,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Most cells an overlap is cut into for its centres.
const MAX_CELLS: usize = 64;

/// The centres of an overlap, a patch and a half apart: its box is cut into
/// cells of that spacing, [`MAX_CELLS`] at most, and each cell gives its
/// best-textured spot. Spread keeps the pair's median shift honest and lets
/// a wide overlap (a Row's closing pair) carry as many matches as fit;
/// texture keeps a low-texture overlap (white wall, grey floor) from being
/// probed on its flat spots only, which once left the closing pair of a
/// Row with a single match and broke the front seam of a Session.
fn centres(
    a: &crate::levels::Level,
    pts: &[(f64, f64)],
    bbox: (f64, f64, f64, f64),
    radius: i32,
) -> Vec<(f64, f64)> {
    let (minx, miny, maxx, maxy) = bbox;
    let min_dist = f64::from(radius) * 1.5;
    let (bw, bh) = ((maxx - minx).max(1.0), (maxy - miny).max(1.0));
    let (mut nx, mut ny) = (
        ((bw / min_dist).ceil() as usize).max(1),
        ((bh / min_dist).ceil() as usize).max(1),
    );
    while nx * ny > MAX_CELLS {
        if nx >= ny {
            nx -= 1;
        } else {
            ny -= 1;
        }
    }
    let mut best: Vec<Option<((f64, f64), f32)>> = vec![None; nx * ny];
    for &p in pts {
        let Some(v) = texture(a, p, radius) else {
            continue;
        };
        let cx = (((p.0 - minx) / bw) * nx as f64) as usize;
        let cy = (((p.1 - miny) / bh) * ny as f64) as usize;
        let cell = &mut best[cy.min(ny - 1) * nx + cx.min(nx - 1)];
        if cell.is_none_or(|(_, b)| v > b) {
            *cell = Some((p, v));
        }
    }
    let mut centres: Vec<(f64, f64)> = Vec::new();
    for (p, _) in best.into_iter().flatten() {
        if centres
            .iter()
            .all(|c| ((c.0 - p.0).powi(2) + (c.1 - p.1).powi(2)).sqrt() >= min_dist)
        {
            centres.push(p);
        }
    }
    centres
}
/// RMS residual of matches under poses, in full-resolution pixels.
pub fn residual_px(corrs: &[Corr], poses: &[Rot], cam: &Cam) -> f64 {
    if corrs.is_empty() {
        return 0.0;
    }
    let sum: f64 = corrs
        .iter()
        .map(|c| {
            ((cam.ray(&poses[c.ia], c.a.0, c.a.1) - cam.ray(&poses[c.ib], c.b.0, c.b.1)) * cam.f)
                .norm_squared()
        })
        .sum();
    (sum / corrs.len() as f64).sqrt()
}
