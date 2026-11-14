//! Patch correlation in a common view: a patch of Shot A around a centre
//! against Shot B warped into A's view through the current poses, over a
//! window of integer shifts, with a sub-pixel peak. Normalised cross
//! correlation on grey pixels; flat patches (sky, fog, asphalt) are refused
//! rather than matched anywhere.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names,
    reason = "pixel coordinates and sizes move between integers and floats throughout, and image maths reads better with short indices"
)]

use crate::geom::{Cam, Rot};
use crate::levels::Level;

/// Variance below which a patch is flat.
const FLAT_VARIANCE: f32 = 2.0;

/// A match found by [`correlate`]: the shift of B's warp that best overlays
/// A's patch, in A's level pixels, and how well.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Peak {
    pub shift: (f64, f64),
    pub ncc: f64,
}

/// The patch and the shifts tried, in level pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub radius: i32,
    pub search: i32,
}

/// Correlates the patch of `a` around `centre` with `b` warped into `a`'s
/// view through `ra` and `rb`, over the shifts of `window`. `cam` is the
/// camera at the levels' scale. `None` when the patch leaves `a`, is flat,
/// or `b` does not cover enough of the window.
pub fn correlate(
    a: &Level,
    b: &Level,
    cam: &Cam,
    ra: &Rot,
    rb: &Rot,
    centre: (f64, f64),
    window: Window,
) -> Option<Peak> {
    let Window { radius, search } = window;
    let side = (2 * radius + 1) as usize;
    let ext = (2 * (radius + search) + 1) as usize;
    let mut pa = vec![0f32; side * side];
    for j in 0..side {
        for i in 0..side {
            let x = centre.0 + f64::from(i as i32 - radius);
            let y = centre.1 + f64::from(j as i32 - radius);
            pa[j * side + i] = a.sample(x, y)?;
        }
    }
    let mean_a = pa.iter().sum::<f32>() / pa.len() as f32;
    let var_a: f32 = pa.iter().map(|v| (v - mean_a).powi(2)).sum::<f32>() / pa.len() as f32;
    if var_a < FLAT_VARIANCE {
        return None;
    }
    let mut pb = vec![f32::NAN; ext * ext];
    let mut valid = 0usize;
    for j in 0..ext {
        for i in 0..ext {
            let x = centre.0 + f64::from(i as i32 - radius - search);
            let y = centre.1 + f64::from(j as i32 - radius - search);
            let d = cam.ray(ra, x, y);
            if let Some((bx, by)) = cam.project(rb, &d)
                && let Some(v) = b.sample(bx, by)
            {
                pb[j * ext + i] = v;
                valid += 1;
            }
        }
    }
    if valid < ext * ext / 2 {
        return None;
    }
    let span = (2 * search + 1) as usize;
    let mut scores = vec![-2.0f32; span * span];
    let mut best = (0i32, 0i32, -2.0f32);
    for sy in -search..=search {
        for sx in -search..=search {
            let (mut n, mut sum_a, mut sum_a2, mut sum_b, mut sum_b2, mut sum_ab) =
                (0usize, 0f32, 0f32, 0f32, 0f32, 0f32);
            for j in 0..side {
                let bj = (j as i32 + search + sy) as usize;
                for i in 0..side {
                    let bi = (i as i32 + search + sx) as usize;
                    let vb = pb[bj * ext + bi];
                    if vb.is_nan() {
                        continue;
                    }
                    let va = pa[j * side + i];
                    n += 1;
                    sum_a += va;
                    sum_a2 += va * va;
                    sum_b += vb;
                    sum_b2 += vb * vb;
                    sum_ab += va * vb;
                }
            }
            if n < side * side * 3 / 4 {
                continue;
            }
            let nf = n as f32;
            let cov = sum_ab - sum_a * sum_b / nf;
            let va = sum_a2 - sum_a * sum_a / nf;
            let vb = sum_b2 - sum_b * sum_b / nf;
            if va <= 1e-3 || vb <= 1e-3 {
                continue;
            }
            let ncc = (cov / (va * vb).sqrt()).clamp(-1.0, 1.0);
            scores[((sy + search) as usize) * span + (sx + search) as usize] = ncc;
            if ncc > best.2 {
                best = (sx, sy, ncc);
            }
        }
    }
    if best.2 < -1.0 {
        return None;
    }
    let at = |sx: i32, sy: i32| -> Option<f64> {
        if sx.abs() > search || sy.abs() > search {
            return None;
        }
        let v = scores[((sy + search) as usize) * span + (sx + search) as usize];
        (v > -1.5).then_some(f64::from(v))
    };
    let refine = |m: f64, l: Option<f64>, r: Option<f64>| -> f64 {
        match (l, r) {
            (Some(l), Some(r)) if (l - 2.0 * m + r).abs() > 1e-9 => {
                (0.5 * (l - r) / (l - 2.0 * m + r)).clamp(-0.5, 0.5)
            }
            _ => 0.0,
        }
    };
    let m = f64::from(best.2);
    let dx = refine(m, at(best.0 - 1, best.1), at(best.0 + 1, best.1));
    let dy = refine(m, at(best.0, best.1 - 1), at(best.0, best.1 + 1));
    Some(Peak {
        shift: (f64::from(best.0) + dx, f64::from(best.1) + dy),
        ncc: m,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::pose;

    fn textured(w: usize, h: usize, phase: f64) -> Level {
        let mut g = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let v = ((x as f64 * 0.37 + phase).sin() * 60.0
                    + (y as f64 * 0.23).cos() * 60.0
                    + 128.0)
                    .clamp(0.0, 255.0);
                g[y * w + x] = v as u8;
            }
        }
        Level {
            w,
            h,
            scale: 1.0,
            g,
        }
    }

    #[test]
    fn an_identical_image_peaks_at_zero_shift() {
        let a = textured(200, 120, 0.0);
        let cam = Cam::new(300.0, 200, 120);
        let r = pose(0.0, 0.0);
        let p = correlate(
            &a,
            &a,
            &cam,
            &r,
            &r,
            (100.0, 60.0),
            Window {
                radius: 12,
                search: 4,
            },
        )
        .unwrap();
        assert!(
            p.ncc > 0.99 && p.shift.0.abs() < 0.05 && p.shift.1.abs() < 0.05,
            "{p:?}"
        );
    }

    #[test]
    fn a_flat_patch_is_refused() {
        let flat = Level {
            w: 100,
            h: 100,
            scale: 1.0,
            g: vec![90; 10_000],
        };
        let cam = Cam::new(100.0, 100, 100);
        let r = pose(0.0, 0.0);
        assert!(
            correlate(
                &flat,
                &flat,
                &cam,
                &r,
                &r,
                (50.0, 50.0),
                Window {
                    radius: 8,
                    search: 3
                }
            )
            .is_none()
        );
    }
}
