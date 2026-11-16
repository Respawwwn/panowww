//! A Shot as the render sees it: its inverse rotation and optical axis in
//! f32, the rows of the Panorama it can cover, and a sampler.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "pixel coordinates and sizes move between integers and floats throughout"
)]

use panowww_core::ports::Frame;

use crate::geom::{Cam, Rot, V3, axis};

/// Pixels a Shot must have on each side of a landing to count as seeing it.
pub const INSIDE_MARGIN: f32 = 8.0;

#[derive(Debug, Clone)]
pub struct ShotView {
    /// Rows of the inverse rotation (world → local).
    pub rinv: [[f32; 3]; 3],
    pub axis: [f32; 3],
    /// Cosine of the angular radius of the Shot's cap, with a small margin.
    pub cos_lim: f32,
    /// First and last Panorama row this Shot can touch.
    pub rows: (usize, usize),
}

impl ShotView {
    /// For a Shot of `cam` at pose `r`, on a Panorama `height` rows tall.
    pub fn new(r: &Rot, cam: &Cam, height: usize) -> Self {
        let m = r.inverse();
        let m = m.matrix();
        let z = axis(r);
        let half_diag = ((cam.w * cam.w + cam.h * cam.h).sqrt() / 2.0 / cam.f).atan();
        // Latitude range: the four corners and the two side midpoints, plus
        // a pole when the cap contains it.
        let mut lat_min = f64::MAX;
        let mut lat_max = f64::MIN;
        for (u, v) in [
            (0.0, 0.0),
            (cam.w - 1.0, 0.0),
            (0.0, cam.h - 1.0),
            (cam.w - 1.0, cam.h - 1.0),
            (cam.cx, 0.0),
            (cam.cx, cam.h - 1.0),
            (0.0, cam.cy),
            (cam.w - 1.0, cam.cy),
            (cam.cx, cam.cy),
        ] {
            let d = cam.ray(r, u, v);
            let lat = d.y.clamp(-1.0, 1.0).asin();
            lat_min = lat_min.min(lat);
            lat_max = lat_max.max(lat);
        }
        let to_pole = |pole: V3| z.dot(&pole).clamp(-1.0, 1.0).acos();
        if to_pole(V3::y()) < half_diag {
            lat_max = std::f64::consts::FRAC_PI_2;
        }
        if to_pole(-V3::y()) < half_diag {
            lat_min = -std::f64::consts::FRAC_PI_2;
        }
        let row_of = |lat: f64| ((0.5 - lat / std::f64::consts::PI) * height as f64).floor();
        let first = row_of(lat_max).clamp(0.0, (height - 1) as f64) as usize;
        let last = row_of(lat_min).clamp(0.0, (height - 1) as f64) as usize;
        Self {
            rinv: [
                [m[(0, 0)] as f32, m[(0, 1)] as f32, m[(0, 2)] as f32],
                [m[(1, 0)] as f32, m[(1, 1)] as f32, m[(1, 2)] as f32],
                [m[(2, 0)] as f32, m[(2, 1)] as f32, m[(2, 2)] as f32],
            ],
            axis: [z.x as f32, z.y as f32, z.z as f32],
            cos_lim: (half_diag + 0.02).cos() as f32,
            rows: (first.saturating_sub(1), (last + 1).min(height - 1)),
        }
    }

    /// Local direction of a world direction.
    #[inline]
    pub fn local(&self, d: [f32; 3]) -> [f32; 3] {
        let r = &self.rinv;
        [
            r[0][0] * d[0] + r[0][1] * d[1] + r[0][2] * d[2],
            r[1][0] * d[0] + r[1][1] * d[1] + r[1][2] * d[2],
            r[2][0] * d[0] + r[2][1] * d[1] + r[2][2] * d[2],
        ]
    }

    #[inline]
    pub fn dot(&self, d: [f32; 3]) -> f32 {
        d[0] * self.axis[0] + d[1] * self.axis[1] + d[2] * self.axis[2]
    }
}

/// World direction of a Panorama pixel.
#[inline]
pub fn direction(x: usize, y: usize, width: usize, height: usize) -> [f32; 3] {
    let lat = (0.5 - (y as f32 + 0.5) / height as f32) * std::f32::consts::PI;
    let lon = ((x as f32 + 0.5) / width as f32 - 0.5) * 2.0 * std::f32::consts::PI;
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();
    [cos_lat * sin_lon, sin_lat, cos_lat * cos_lon]
}

/// A Shot's pixels with bilinear sampling, coordinates clamped into the
/// image so a landing just outside takes the edge's colour.
pub struct Sampler<'a> {
    px: &'a [u8],
    w: usize,
    h: usize,
}

impl<'a> Sampler<'a> {
    pub fn new(frame: &'a Frame) -> Self {
        Self {
            px: &frame.rgb,
            w: frame.width as usize,
            h: frame.height as usize,
        }
    }

    #[inline]
    pub fn rgb(&self, u: f32, v: f32) -> [f32; 3] {
        let u = u.clamp(0.0, (self.w - 1) as f32 - 0.001);
        let v = v.clamp(0.0, (self.h - 1) as f32 - 0.001);
        let (x0, y0) = (u as usize, v as usize);
        let (fx, fy) = (u - x0 as f32, v - y0 as f32);
        let i00 = (y0 * self.w + x0) * 3;
        let i10 = i00 + 3;
        let i01 = i00 + self.w * 3;
        let i11 = i01 + 3;
        let mut out = [0f32; 3];
        for (c, o) in out.iter_mut().enumerate() {
            let top = f32::from(self.px[i00 + c]) * (1.0 - fx) + f32::from(self.px[i10 + c]) * fx;
            let bot = f32::from(self.px[i01 + c]) * (1.0 - fx) + f32::from(self.px[i11 + c]) * fx;
            *o = top * (1.0 - fy) + bot * fy;
        }
        out
    }
}
