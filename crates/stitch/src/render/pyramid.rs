//! Burt–Adelson pyramids on one f32 plane: 5-tap binomial reduce, bilinear
//! expand, Gaussian and Laplacian stacks. Rows in parallel.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    reason = "pixel positions become sample positions, and image maths reads better with short indices over rows"
)]

use rayon::prelude::*;

/// Bands of the multi-band blend.
pub const LEVELS: usize = 6;
/// Every box the pyramids run on is a multiple of this, so levels line up.
pub const ALIGN: usize = 1 << (LEVELS - 1);

/// One f32 plane.
#[derive(Debug, Clone)]
pub struct Plane {
    pub w: usize,
    pub h: usize,
    pub d: Vec<f32>,
}

impl Plane {
    pub fn zeros(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            d: vec![0.0; w * h],
        }
    }
}

/// 5-tap binomial blur, clamped borders, separable.
fn blur5(p: &Plane) -> Plane {
    let (w, h) = (p.w, p.h);
    let k = [
        1.0f32 / 16.0,
        4.0 / 16.0,
        6.0 / 16.0,
        4.0 / 16.0,
        1.0 / 16.0,
    ];
    let mut tmp = Plane::zeros(w, h);
    tmp.d.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let src = &p.d[y * w..(y + 1) * w];
        for x in 0..w {
            let mut a = 0.0;
            for (i, kk) in k.iter().enumerate() {
                let xx = (x as isize + i as isize - 2).clamp(0, w as isize - 1) as usize;
                a += src[xx] * kk;
            }
            row[x] = a;
        }
    });
    let mut out = Plane::zeros(w, h);
    out.d.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (i, kk) in k.iter().enumerate() {
            let yy = (y as isize + i as isize - 2).clamp(0, h as isize - 1) as usize;
            let src = &tmp.d[yy * w..(yy + 1) * w];
            for x in 0..w {
                row[x] += src[x] * kk;
            }
        }
    });
    out
}

/// Blur, then keep every other pixel. Dimensions must be even.
fn reduce(p: &Plane) -> Plane {
    let b = blur5(p);
    let (w2, h2) = (p.w / 2, p.h / 2);
    let mut out = Plane::zeros(w2, h2);
    out.d.par_chunks_mut(w2).enumerate().for_each(|(y, row)| {
        for x in 0..w2 {
            row[x] = b.d[(2 * y) * p.w + 2 * x];
        }
    });
    out
}

/// Bilinear 2× upsample to `w × h`.
pub fn expand(p: &Plane, w: usize, h: usize) -> Plane {
    let mut out = Plane::zeros(w, h);
    out.d.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let fy = (f32::midpoint(y as f32, 0.5) - 0.5).clamp(0.0, (p.h - 1) as f32);
        let y0 = fy as usize;
        let y1 = (y0 + 1).min(p.h - 1);
        let ty = fy - y0 as f32;
        for x in 0..w {
            let fx = (f32::midpoint(x as f32, 0.5) - 0.5).clamp(0.0, (p.w - 1) as f32);
            let x0 = fx as usize;
            let x1 = (x0 + 1).min(p.w - 1);
            let tx = fx - x0 as f32;
            let top = p.d[y0 * p.w + x0] * (1.0 - tx) + p.d[y0 * p.w + x1] * tx;
            let bot = p.d[y1 * p.w + x0] * (1.0 - tx) + p.d[y1 * p.w + x1] * tx;
            row[x] = top * (1.0 - ty) + bot * ty;
        }
    });
    out
}

/// Gaussian pyramid of [`LEVELS`] planes.
pub fn gaussian(p: Plane) -> Vec<Plane> {
    let mut g = vec![p];
    for _ in 1..LEVELS {
        let next = reduce(g.last().expect("one level at least"));
        g.push(next);
    }
    g
}

/// Laplacian pyramid from a Gaussian one, the top level kept as is.
pub fn laplacian(g: &[Plane]) -> Vec<Plane> {
    let mut l = Vec::with_capacity(LEVELS);
    for i in 0..LEVELS - 1 {
        let up = expand(&g[i + 1], g[i].w, g[i].h);
        let mut p = Plane::zeros(g[i].w, g[i].h);
        p.d.par_iter_mut()
            .enumerate()
            .for_each(|(k, v)| *v = g[i].d[k] - up.d[k]);
        l.push(p);
    }
    l.push(g[LEVELS - 1].clone());
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_laplacian_pyramid_collapses_back_to_its_image() {
        let (w, h) = (128, 64);
        let mut p = Plane::zeros(w, h);
        for y in 0..h {
            for x in 0..w {
                p.d[y * w + x] = ((x * 7 + y * 3) % 50) as f32;
            }
        }
        let g = gaussian(p.clone());
        let l = laplacian(&g);
        let mut img = l[LEVELS - 1].clone();
        for level in (0..LEVELS - 1).rev() {
            let up = expand(&img, l[level].w, l[level].h);
            img = Plane {
                w: l[level].w,
                h: l[level].h,
                d: l[level].d.iter().zip(&up.d).map(|(a, b)| a + b).collect(),
            };
        }
        let worst = img
            .d
            .iter()
            .zip(&p.d)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(worst < 1e-3, "collapse error {worst}");
    }
}
