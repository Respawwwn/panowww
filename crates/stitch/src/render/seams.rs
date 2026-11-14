//! The seam finder (spec §16.6 step 6): where two Shots meet, the label
//! boundary moves from the Voronoi line to the path along which the two
//! Shots agree best, a minimum-error boundary found by dynamic programming
//! on the quarter-resolution difference of the two warped Shots, then
//! lifted back to full resolution. Parallax, wind-blown foliage and
//! swallowed Steps show as a clean cut through agreeing pixels instead of
//! a cut through an object.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::type_complexity,
    clippy::needless_range_loop,
    reason = "pixel coordinates and sizes move between integers and floats throughout; one stage, its inputs named; the two dynamic programmes read as one piece with the textbook names"
)]

use std::collections::HashMap;
use std::sync::Arc;

use rayon::prelude::*;

use panowww_core::ports::Frame;

use super::view::{INSIDE_MARGIN, Sampler, ShotView, direction};
use crate::geom::Cam;

/// Reduction of the seam search.
const SCALE: usize = 4;
/// How far the search region extends around a boundary, in reduced pixels.
const REACH: usize = 12;
/// Boundaries shorter than this stay where the Voronoi put them.
const MIN_BOUNDARY: usize = 8;
/// Cost of leaving the overlap: the seam never does.
const OUTSIDE: f32 = 1.0e6;

const NO_LABEL: u16 = u16::MAX;

/// One Shot warped into the band at reduced resolution: grey and whether
/// the Shot sees each pixel.
struct Warp {
    grey: Vec<f32>,
    valid: Vec<bool>,
}

/// Moves the seams of `labels` (a band of `eh` rows starting at Panorama
/// row `e0`) to where the Shots agree.
pub fn refine(
    mut labels: Vec<u16>,
    views: &[ShotView],
    touching: &[usize],
    cache: &HashMap<usize, Arc<Frame>>,
    cam: &Cam,
    width: usize,
    height: usize,
    e0: usize,
    eh: usize,
) -> Vec<u16> {
    let (w4, h4) = (width / SCALE, eh / SCALE);
    if w4 < 2 * REACH || h4 < 2 * REACH {
        return labels;
    }
    // Labels at the reduced scale: the centre pixel of each block.
    let mut small: Vec<u16> = (0..w4 * h4)
        .map(|k| {
            let (x, y) = (k % w4, k / w4);
            labels[(y * SCALE + SCALE / 2) * width + x * SCALE + SCALE / 2]
        })
        .collect();
    let before = small.clone();
    // Every touching Shot warped at the reduced scale.
    let (f, cx, cy) = (cam.f as f32, cam.cx as f32, cam.cy as f32);
    let (sw, sh) = (cam.w as f32, cam.h as f32);
    let warps: HashMap<usize, Warp> = touching
        .par_iter()
        .map(|&i| {
            let v = &views[i];
            let sampler = Sampler::new(&cache[&i]);
            let mut grey = vec![0f32; w4 * h4];
            let mut valid = vec![false; w4 * h4];
            for y in 0..h4 {
                for x in 0..w4 {
                    let d = direction(
                        x * SCALE + SCALE / 2,
                        e0 + y * SCALE + SCALE / 2,
                        width,
                        height,
                    );
                    let l = v.local(d);
                    if l[2] <= 1e-6 {
                        continue;
                    }
                    let u = cx + f * l[0] / l[2];
                    let vv = cy - f * l[1] / l[2];
                    if u < INSIDE_MARGIN
                        || vv < INSIDE_MARGIN
                        || u > sw - 1.0 - INSIDE_MARGIN
                        || vv > sh - 1.0 - INSIDE_MARGIN
                    {
                        continue;
                    }
                    let px = sampler.rgb(u, vv);
                    grey[y * w4 + x] = 0.299 * px[0] + 0.587 * px[1] + 0.114 * px[2];
                    valid[y * w4 + x] = true;
                }
            }
            (i, Warp { grey, valid })
        })
        .collect();
    // Boundaries between pairs of labels.
    let mut boundaries: HashMap<(u16, u16), Vec<(usize, usize)>> = HashMap::new();
    for y in 0..h4 {
        for x in 0..w4 {
            let a = small[y * w4 + x];
            if a == NO_LABEL {
                continue;
            }
            for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                if nx >= w4 || ny >= h4 {
                    continue;
                }
                let b = small[ny * w4 + nx];
                if b == NO_LABEL || b == a {
                    continue;
                }
                let key = if a < b { (a, b) } else { (b, a) };
                boundaries.entry(key).or_default().push((x, y));
            }
        }
    }
    let mut pairs: Vec<((u16, u16), Vec<(usize, usize)>)> = boundaries
        .into_iter()
        .filter(|(_, pts)| pts.len() >= MIN_BOUNDARY)
        .collect();
    pairs.sort_by_key(|(k, _)| *k);
    for ((a, b), pts) in pairs {
        let (Some(wa), Some(wb)) = (warps.get(&(a as usize)), warps.get(&(b as usize))) else {
            continue;
        };
        let (minx, maxx) = pts
            .iter()
            .fold((usize::MAX, 0), |m, p| (m.0.min(p.0), m.1.max(p.0)));
        let (miny, maxy) = pts
            .iter()
            .fold((usize::MAX, 0), |m, p| (m.0.min(p.1), m.1.max(p.1)));
        let x0 = minx.saturating_sub(REACH);
        let x1 = (maxx + REACH + 1).min(w4);
        let y0 = miny.saturating_sub(REACH);
        let y1 = (maxy + REACH + 1).min(h4);
        let (bw, bh) = (x1 - x0, y1 - y0);
        if bw < 3 || bh < 3 {
            continue;
        }
        // Difference of the two Shots where both see the pixel.
        let cost = |x: usize, y: usize| -> f32 {
            let k = y * w4 + x;
            if wa.valid[k] && wb.valid[k] {
                (wa.grey[k] - wb.grey[k]).abs()
            } else {
                OUTSIDE
            }
        };
        // Which side is A: by the mean position of its pixels in the box.
        let (mut sa, mut na, mut sb, mut nb) = ((0.0f64, 0.0f64), 0usize, (0.0f64, 0.0f64), 0usize);
        for y in y0..y1 {
            for x in x0..x1 {
                match small[y * w4 + x] {
                    l if l == a => {
                        sa = (sa.0 + x as f64, sa.1 + y as f64);
                        na += 1;
                    }
                    l if l == b => {
                        sb = (sb.0 + x as f64, sb.1 + y as f64);
                        nb += 1;
                    }
                    _ => {}
                }
            }
        }
        if na == 0 || nb == 0 {
            continue;
        }
        let (ma, mb) = (
            (sa.0 / na as f64, sa.1 / na as f64),
            (sb.0 / nb as f64, sb.1 / nb as f64),
        );
        let vertical = (maxy - miny) >= (maxx - minx);
        if vertical {
            // One x per row, moving at most one column per row.
            let mut acc = vec![0f32; bw * bh];
            let mut from = vec![0u8; bw * bh];
            for x in 0..bw {
                acc[x] = cost(x0 + x, y0);
            }
            for y in 1..bh {
                for x in 0..bw {
                    let mut best = (acc[(y - 1) * bw + x], 1u8);
                    if x > 0 && acc[(y - 1) * bw + x - 1] < best.0 {
                        best = (acc[(y - 1) * bw + x - 1], 0);
                    }
                    if x + 1 < bw && acc[(y - 1) * bw + x + 1] < best.0 {
                        best = (acc[(y - 1) * bw + x + 1], 2);
                    }
                    acc[y * bw + x] = best.0 + cost(x0 + x, y0 + y);
                    from[y * bw + x] = best.1;
                }
            }
            let mut x = (0..bw)
                .min_by(|&p, &q| acc[(bh - 1) * bw + p].total_cmp(&acc[(bh - 1) * bw + q]))
                .unwrap_or(0);
            if acc[(bh - 1) * bw + x] >= OUTSIDE {
                continue;
            }
            let mut path = vec![0usize; bh];
            for y in (0..bh).rev() {
                path[y] = x;
                if y > 0 {
                    x = match from[y * bw + x] {
                        0 => x - 1,
                        2 => x + 1,
                        _ => x,
                    };
                }
            }
            let a_left = ma.0 < mb.0;
            for y in 0..bh {
                for x in 0..bw {
                    let k = (y0 + y) * w4 + x0 + x;
                    let l = small[k];
                    if (l != a && l != b) || !(wa.valid[k] && wb.valid[k]) {
                        continue;
                    }
                    let left = x < path[y];
                    small[k] = if left == a_left { a } else { b };
                }
            }
        } else {
            // One y per column, moving at most one row per column.
            let mut acc = vec![0f32; bw * bh];
            let mut from = vec![0u8; bw * bh];
            for y in 0..bh {
                acc[y * bw] = cost(x0, y0 + y);
            }
            for x in 1..bw {
                for y in 0..bh {
                    let mut best = (acc[y * bw + x - 1], 1u8);
                    if y > 0 && acc[(y - 1) * bw + x - 1] < best.0 {
                        best = (acc[(y - 1) * bw + x - 1], 0);
                    }
                    if y + 1 < bh && acc[(y + 1) * bw + x - 1] < best.0 {
                        best = (acc[(y + 1) * bw + x - 1], 2);
                    }
                    acc[y * bw + x] = best.0 + cost(x0 + x, y0 + y);
                    from[y * bw + x] = best.1;
                }
            }
            let mut y = (0..bh)
                .min_by(|&p, &q| acc[p * bw + bw - 1].total_cmp(&acc[q * bw + bw - 1]))
                .unwrap_or(0);
            if acc[y * bw + bw - 1] >= OUTSIDE {
                continue;
            }
            let mut path = vec![0usize; bw];
            for x in (0..bw).rev() {
                path[x] = y;
                if x > 0 {
                    y = match from[y * bw + x] {
                        0 => y - 1,
                        2 => y + 1,
                        _ => y,
                    };
                }
            }
            let a_above = ma.1 < mb.1;
            for y in 0..bh {
                for x in 0..bw {
                    let k = (y0 + y) * w4 + x0 + x;
                    let l = small[k];
                    if (l != a && l != b) || !(wa.valid[k] && wb.valid[k]) {
                        continue;
                    }
                    let above = y < path[x];
                    small[k] = if above == a_above { a } else { b };
                }
            }
        }
    }
    // Lift the moved labels back: a full-resolution pixel whose block changed
    // takes the block's new Shot when that Shot sees it.
    labels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(j, row)| {
            let y4 = j / SCALE;
            for x in 0..width {
                let k4 = y4 * w4 + x / SCALE;
                let (old, new) = (before[k4], small[k4]);
                if old == new || new == NO_LABEL {
                    continue;
                }
                let v = &views[new as usize];
                let l = v.local(direction(x, e0 + j, width, height));
                if l[2] <= 1e-6 {
                    continue;
                }
                let u = cx + f * l[0] / l[2];
                let vv = cy - f * l[1] / l[2];
                if u < INSIDE_MARGIN
                    || vv < INSIDE_MARGIN
                    || u > sw - 1.0 - INSIDE_MARGIN
                    || vv > sh - 1.0 - INSIDE_MARGIN
                {
                    continue;
                }
                row[x] = new;
            }
        });
    labels
}
