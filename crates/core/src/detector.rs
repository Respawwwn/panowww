//! The screen-unchanged detector (spec §4.6): both images reduced to
//! [`DETECTOR_LONG_SIDE`] on the long side, greyscale, mean absolute
//! difference under [`DETECTOR_MAX_DIFFERENCE`]. One-sided: unchanged
//! proves nothing moved; changed proves nothing. The Runner feeds it the
//! two detector Samples of a Test and the first two Shots of a Session.

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{PixelType, Resizer};

use crate::constants::{DETECTOR_LONG_SIDE, DETECTOR_MAX_DIFFERENCE};
use crate::ports::Frame;
use crate::runner::thumbnail::fit;

/// Whether the screen looks the same in `before` and `after`. Frames of
/// different sizes or malformed frames count as changed: no evidence.
pub fn unchanged(before: &Frame, after: &Frame) -> bool {
    difference(before, after).is_some_and(|d| d < DETECTOR_MAX_DIFFERENCE)
}

/// Mean absolute difference of the two reduced greyscale images as a share
/// of full scale: 0.0 identical, 1.0 black against white. `None` when the
/// frames differ in size or one is malformed.
pub fn difference(before: &Frame, after: &Frame) -> Option<f64> {
    if (before.width, before.height) != (after.width, after.height) {
        return None;
    }
    let a = reduced_grey(before)?;
    let b = reduced_grey(after)?;
    let sum: u32 = a
        .iter()
        .zip(&b)
        .map(|(x, y)| u32::from(x.abs_diff(*y)))
        .sum();
    let pixels = u32::try_from(a.len()).ok()?;
    Some(f64::from(sum) / (f64::from(pixels) * 255.0))
}

fn reduced_grey(frame: &Frame) -> Option<Vec<u8>> {
    if !frame.is_well_formed() || frame.width == 0 || frame.height == 0 {
        return None;
    }
    let (width, height) = fit(frame.width, frame.height, DETECTOR_LONG_SIDE);
    let source = ImageRef::new(frame.width, frame.height, &frame.rgb, PixelType::U8x3).ok()?;
    let mut reduced = Image::new(width, height, PixelType::U8x3);
    Resizer::new().resize(&source, &mut reduced, None).ok()?;
    Some(
        reduced
            .buffer()
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| luma(p[0], p[1], p[2]))
            .collect(),
    )
}

/// Rec. 601 luma, integer arithmetic.
fn luma(r: u8, g: u8, b: u8) -> u8 {
    let weighted = u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114;
    u8::try_from(weighted / 1000).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: u32 = 640;
    const H: u32 = 360;

    /// Vertical black and white stripes `period / 2` wide, shifted right
    /// by `shift` pixels, with `noise` added to every channel from a tiny
    /// deterministic generator.
    fn stripes(shift: u32, period: u32, noise: u8) -> Frame {
        let mut rgb = Vec::with_capacity((W * H * 3) as usize);
        let mut seed: u32 = 12345;
        for y in 0..H {
            for x in 0..W {
                let band = ((x + W - shift % W) / (period / 2)) % 2;
                let base: u8 = if band == 0 { 40 } else { 200 };
                for _ in 0..3 {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223 + y);
                    let jitter = u8::try_from((seed >> 24) % (u32::from(noise) + 1)).unwrap_or(0);
                    rgb.push(base.saturating_add(jitter));
                }
            }
        }
        Frame {
            width: W,
            height: H,
            rgb,
        }
    }

    #[test]
    fn identical_frames_are_unchanged() {
        let a = stripes(0, 80, 0);
        assert_eq!(difference(&a, &a), Some(0.0));
        assert!(unchanged(&a, &a));
        assert!(unchanged(
            &Frame::solid(100, 60, [7, 7, 7]),
            &Frame::solid(100, 60, [7, 7, 7])
        ));
    }

    #[test]
    fn one_sub_move_of_shift_is_a_change() {
        let before = stripes(0, 80, 0);
        let after = stripes(20, 80, 0);
        let d = difference(&before, &after).unwrap();
        assert!(d > 0.2, "{d}");
        assert!(!unchanged(&before, &after));
    }

    #[test]
    fn a_whole_period_of_shift_looks_the_same() {
        assert!(unchanged(&stripes(0, 80, 0), &stripes(80, 80, 0)));
    }

    #[test]
    fn sensor_noise_does_not_count_as_movement() {
        let before = stripes(0, 80, 0);
        let after = stripes(0, 80, 3);
        let d = difference(&before, &after).unwrap();
        assert!(d < DETECTOR_MAX_DIFFERENCE, "{d}");
        assert!(unchanged(&before, &after));
    }

    #[test]
    fn a_small_uniform_brightness_step_is_still_unchanged() {
        let a = Frame::solid(W, H, [100, 100, 100]);
        let b = Frame::solid(W, H, [102, 102, 102]);
        assert!(unchanged(&a, &b));
        let c = Frame::solid(W, H, [110, 110, 110]);
        assert!(!unchanged(&a, &c), "10 levels is 3.9 %");
    }

    #[test]
    fn different_sizes_or_malformed_frames_are_no_evidence() {
        assert_eq!(
            difference(&Frame::solid(10, 10, [0; 3]), &Frame::solid(10, 11, [0; 3])),
            None
        );
        assert!(!unchanged(
            &Frame::solid(10, 10, [0; 3]),
            &Frame::solid(10, 11, [0; 3])
        ));
        let broken = Frame {
            width: 10,
            height: 10,
            rgb: vec![0; 5],
        };
        assert!(!unchanged(&broken, &broken));
        assert!(!unchanged(
            &Frame::solid(0, 0, [0; 3]),
            &Frame::solid(0, 0, [0; 3])
        ));
    }

    #[test]
    fn luma_weights_green_most() {
        assert_eq!(luma(255, 255, 255), 255);
        assert_eq!(luma(0, 0, 0), 0);
        assert!(luma(0, 255, 0) > luma(255, 0, 0));
        assert!(luma(255, 0, 0) > luma(0, 0, 255));
    }
}
