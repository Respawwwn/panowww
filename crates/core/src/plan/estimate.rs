//! The duration estimate of a run (spec §7.6) and its display form.

use std::time::Duration;

use super::CapturePlan;

/// The countdown plus the sum of the plan. For a Session this is exactly
/// spec §7.6:
///
/// ```text
/// countdown + N × R × (waitAfterShot + stepTime(turn) + waitAfterMove + 50 ms)
///           + (R − 1) × (stepTime(pitch) + waitAfterMove)
/// ```
pub fn estimate(plan: &CapturePlan, countdown: Duration) -> Duration {
    countdown + plan.duration()
}

/// `≈ 4 min 20 s`, `≈ 45 s`, `≈ 1 h 12 min`: rounded to the second, zero
/// parts dropped, seconds dropped above an hour.
pub fn format_duration(duration: Duration) -> String {
    let secs = (duration.as_millis() + 500) / 1000;
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    match (h, m, s) {
        (0, 0, s) => format!("≈ {s} s"),
        (0, m, 0) => format!("≈ {m} min"),
        (0, m, s) => format!("≈ {m} min {s} s"),
        (h, 0, _) => format!("≈ {h} h"),
        (h, m, _) => format!("≈ {h} h {m} min"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_rounds_and_drops_zero_parts() {
        let cases = [
            (0, "≈ 0 s"),
            (499, "≈ 0 s"),
            (500, "≈ 1 s"),
            (45_000, "≈ 45 s"),
            (60_000, "≈ 1 min"),
            (260_000, "≈ 4 min 20 s"),
            (259_600, "≈ 4 min 20 s"),
            (3_600_000, "≈ 1 h"),
            (4_320_000, "≈ 1 h 12 min"),
            (4_323_540_000, "≈ 1200 h 59 min"),
        ];
        for (ms, text) in cases {
            assert_eq!(format_duration(Duration::from_millis(ms)), text, "{ms} ms");
        }
    }
}
