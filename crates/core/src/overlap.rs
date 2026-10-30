//! Check overlap maths (spec §7.4, §11.8): the Overlap two neighbouring
//! Shots share, from the offset the user dialled in to align the two
//! Samples, and its verdict against the stitcher's target.

use crate::constants::OVERLAP_TARGET;

/// How the Overlap compares with [`OVERLAP_TARGET`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Inside the target.
    Ok,
    /// Too little: more Shots per Row, or more Rows.
    Low,
    /// More than needed: fewer Shots per Row, or fewer Rows.
    High,
}

/// `1 − offset / size`, in percent, clamped to 0–100. `offset` is how far
/// the second Sample had to slide along the Step axis to line up with the
/// first, in the same unit as `size`, the Sample's extent on that axis.
pub fn overlap_percent(offset: i32, size: u32) -> f64 {
    if size == 0 {
        return 0.0;
    }
    let share = 1.0 - f64::from(offset.unsigned_abs()) / f64::from(size);
    share.clamp(0.0, 1.0) * 100.0
}

/// The verdict for an Overlap in percent; the target bounds are inside it.
pub fn verdict(percent: f64) -> Verdict {
    if percent < *OVERLAP_TARGET.start() {
        Verdict::Low
    } else if percent > *OVERLAP_TARGET.end() {
        Verdict::High
    } else {
        Verdict::Ok
    }
}

/// The percent as shown in the chip and the verdict: whole number.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to 0-100 first"
)]
pub fn rounded_percent(percent: f64) -> u8 {
    // Clamped first, so the cast is exact.
    percent.round().clamp(0.0, 100.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_is_one_minus_offset_over_size() {
        assert!((overlap_percent(0, 1000) - 100.0).abs() < f64::EPSILON);
        assert!((overlap_percent(700, 1000) - 30.0).abs() < 1e-9);
        assert!(
            (overlap_percent(-700, 1000) - 30.0).abs() < 1e-9,
            "sliding either way"
        );
        assert!((overlap_percent(1000, 1000) - 0.0).abs() < f64::EPSILON);
        assert!(
            (overlap_percent(1500, 1000) - 0.0).abs() < f64::EPSILON,
            "clamped"
        );
        assert!(
            (overlap_percent(10, 0) - 0.0).abs() < f64::EPSILON,
            "no size, no Overlap"
        );
    }

    #[test]
    fn verdict_bounds_are_inclusive() {
        assert_eq!(verdict(25.0), Verdict::Ok);
        assert_eq!(verdict(30.0), Verdict::Ok);
        assert_eq!(verdict(35.0), Verdict::Ok);
        assert_eq!(verdict(24.999), Verdict::Low);
        assert_eq!(verdict(0.0), Verdict::Low);
        assert_eq!(verdict(35.001), Verdict::High);
        assert_eq!(verdict(100.0), Verdict::High);
    }

    #[test]
    fn rounded_percent_is_a_whole_number() {
        assert_eq!(rounded_percent(29.5), 30);
        assert_eq!(rounded_percent(29.49), 29);
        assert_eq!(rounded_percent(100.0), 100);
        assert_eq!(rounded_percent(0.0), 0);
    }
}
