//! Camera geometry. World axes: X right, Y up, Z forward at yaw 0. A
//! Shot's pixels (u right, v down) unproject through its rotation. Yaw
//! grows turning right, pitch grows looking up.

use nalgebra::{Rotation3, Vector3};

pub type Rot = Rotation3<f64>;
pub type V3 = Vector3<f64>;

/// Rotation of a Shot: yaw about the vertical, then pitch up.
pub fn pose(yaw_rad: f64, pitch_rad: f64) -> Rot {
    Rot::from_axis_angle(&V3::y_axis(), yaw_rad) * Rot::from_axis_angle(&V3::x_axis(), -pitch_rad)
}

/// A rectilinear camera: focal length in pixels and the image size, all in
/// the pixels of one resolution level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cam {
    pub f: f64,
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
}

impl Cam {
    pub fn new(f: f64, width: u32, height: u32) -> Self {
        let (w, h) = (f64::from(width), f64::from(height));
        Self {
            f,
            cx: w / 2.0,
            cy: h / 2.0,
            w,
            h,
        }
    }

    /// World ray of a pixel.
    pub fn ray(&self, r: &Rot, u: f64, v: f64) -> V3 {
        let local = V3::new((u - self.cx) / self.f, -(v - self.cy) / self.f, 1.0);
        (r * local).normalize()
    }

    /// Pixel of a world ray, if in front of the camera.
    pub fn project(&self, r: &Rot, d: &V3) -> Option<(f64, f64)> {
        let l = r.inverse() * d;
        if l.z <= 1e-6 {
            return None;
        }
        Some((self.cx + self.f * l.x / l.z, self.cy - self.f * l.y / l.z))
    }

    /// Whether a pixel lies at least `margin` inside the image.
    pub fn inside(&self, uv: (f64, f64), margin: f64) -> bool {
        uv.0 >= margin
            && uv.1 >= margin
            && uv.0 <= self.w - 1.0 - margin
            && uv.1 <= self.h - 1.0 - margin
    }

    pub fn hfov_deg(&self) -> f64 {
        2.0 * (self.w / (2.0 * self.f)).atan().to_degrees()
    }

    /// Focal length for a horizontal field of view, in this camera's pixels.
    pub fn f_for_hfov(width: f64, hfov_deg: f64) -> f64 {
        (width / 2.0) / (hfov_deg.to_radians() / 2.0).tan()
    }
}

/// The optical axis of a pose.
pub fn axis(r: &Rot) -> V3 {
    r * V3::z()
}

/// Yaw and pitch of a pose, in radians.
pub fn yaw_pitch(r: &Rot) -> (f64, f64) {
    let z = axis(r);
    (z.x.atan2(z.z), z.y.clamp(-1.0, 1.0).asin())
}

/// Wraps an angle in degrees to (−180, 180].
pub fn wrap_deg(x: f64) -> f64 {
    let mut x = x % 360.0;
    if x > 180.0 {
        x -= 360.0;
    }
    if x <= -180.0 {
        x += 360.0;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaw_turns_right_and_pitch_looks_up() {
        let z = axis(&pose(30f64.to_radians(), 20f64.to_radians()));
        assert!((z.x.atan2(z.z).to_degrees() - 30.0).abs() < 1e-9);
        assert!((z.y.asin().to_degrees() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn project_inverts_ray() {
        let cam = Cam::new(800.0, 1600, 900);
        let r = pose(1.0, -0.4);
        let d = cam.ray(&r, 123.0, 456.0);
        let (u, v) = cam.project(&r, &d).unwrap();
        assert!((u - 123.0).abs() < 1e-6 && (v - 456.0).abs() < 1e-6);
    }

    #[test]
    fn wrap_keeps_angles_in_a_turn() {
        assert!((wrap_deg(370.0) - 10.0).abs() < 1e-9);
        assert!((wrap_deg(-190.0) - 170.0).abs() < 1e-9);
        assert!((wrap_deg(180.0) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn field_of_view_round_trips() {
        let f = Cam::f_for_hfov(1600.0, 90.0);
        assert!((Cam::new(f, 1600, 900).hfov_deg() - 90.0).abs() < 1e-9);
    }
}
