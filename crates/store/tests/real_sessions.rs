//! The real Sessions of the sibling repository `panowww-stitch-fixtures`,
//! stitched again and held to the Panorama that was approved by eye: the
//! report must match its snapshot exactly, the pixels must stay within a
//! small tolerance of the reference JPEG. The tolerance is there for other
//! CPUs, whose floating point may differ in the last bit; on one machine the
//! output is byte for byte the same.
//!
//! Ignored by default: the fixtures weigh half a gigabyte and live in
//! <https://github.com/Respawwwn/panowww-stitch-fixtures>, expected next to
//! this repository or where `PANOWWW_STITCH_FIXTURES` points. `just
//! stitch-regress` runs them; `just stitch-regress-accept` rewrites the
//! references and the snapshots from the current code.

use std::path::{Path, PathBuf};
use std::time::Instant;

use image::RgbImage;
use panowww_core::ports::CancelFlag;
use panowww_stitch::{Grid, Limits, ShotRef, StitchOptions, stitch};
use panowww_store::panorama::{self, FolderShotSource, write_panorama};
use panowww_store::session::{SessionRecord, shot_file_name};
use panowww_store::sink::WallClock;

/// Set to rewrite the reference Panorama instead of comparing with it.
const WRITE: &str = "PANOWWW_STITCH_FIXTURES_WRITE";
/// Mean absolute difference per channel the Panorama may drift from its
/// reference. The same Panorama encoded at JPEG quality 90 instead of 100
/// moves by 0.17 levels on average and 0.024 % of its pixels by more than
/// 8: both limits sit well under that, so even a change of that size fails.
const MEAN_TOLERANCE: f64 = 0.05;
/// A channel that moved by more than this many levels counts as a far pixel…
const FAR_LEVELS: u8 = 8;
/// …and at most this share of the pixels may be far.
const FAR_TOLERANCE: f64 = 0.000_05;

struct FrozenClock(jiff::Zoned);

impl WallClock for FrozenClock {
    fn now(&self) -> jiff::Zoned {
        self.0.clone()
    }
}

/// `PANOWWW_STITCH_FIXTURES`, else the sibling folder of this repository.
fn fixtures() -> PathBuf {
    std::env::var_os("PANOWWW_STITCH_FIXTURES").map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../panowww-stitch-fixtures")
                .canonicalize()
                .expect(
                    "the fixtures: clone Respawwwn/panowww-stitch-fixtures next to this repository \
                     or set PANOWWW_STITCH_FIXTURES",
                )
        },
        PathBuf::from,
    )
}

/// The report, rounded, as the snapshot holds it.
#[derive(Debug)]
#[allow(dead_code, reason = "read by the Debug snapshot")]
struct Summary {
    size: String,
    field_of_view_deg: String,
    turn_step_deg: String,
    pitch_step_deg: String,
    start_pole_offset_deg: String,
    closing_turn_deg: String,
    drift_deg: String,
    unaligned: Vec<String>,
    dropped_steps: Vec<String>,
}

fn name(shot: ShotRef) -> String {
    shot_file_name(shot.row, shot.col)
        .trim_end_matches(".png")
        .to_owned()
}

fn decode(path: &Path) -> RgbImage {
    image::ImageReader::open(path)
        .and_then(|r| r.decode().map_err(std::io::Error::other))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .into_rgb8()
}

/// Mean absolute difference per channel, and the share of far pixels.
#[allow(
    clippy::cast_precision_loss,
    reason = "sums over a few tens of millions of samples, well within f64"
)]
fn distance(a: &RgbImage, b: &RgbImage) -> (f64, f64) {
    let mut sum = 0u64;
    let mut far = 0u64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let mut worst = 0u8;
        for (ca, cb) in pa.0.iter().zip(pb.0.iter()) {
            let d = ca.abs_diff(*cb);
            sum += u64::from(d);
            worst = worst.max(d);
        }
        far += u64::from(worst > FAR_LEVELS);
    }
    let pixels = u64::from(a.width()) * u64::from(a.height());
    (sum as f64 / (pixels * 3) as f64, far as f64 / pixels as f64)
}

fn check(slug: &str) {
    let fixtures = fixtures();
    let folder = fixtures.join("sessions").join(slug);
    let record =
        SessionRecord::read(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()));
    let source = FolderShotSource::from_record(&folder, &record);
    let grid = Grid::from_game(&record.game);
    let started = Instant::now();
    let stitched = stitch(
        &source,
        &grid,
        &Limits::default(),
        &StitchOptions::default(),
        &|_| {},
        &CancelFlag::new(),
    )
    .unwrap_or_else(|e| panic!("{slug}: {e}"));
    let r = &stitched.report;
    eprintln!(
        "{slug}: {} × {} in {:.1} s · hfov {:.2}° · {} unaligned · {} dropped",
        r.width,
        r.height,
        started.elapsed().as_secs_f64(),
        r.field_of_view_deg,
        r.unaligned.len(),
        r.dropped_steps.len()
    );

    let summary = Summary {
        size: format!("{} × {}", r.width, r.height),
        field_of_view_deg: format!("{:.2}", r.field_of_view_deg),
        turn_step_deg: format!("{:.2}", r.turn_step_deg),
        pitch_step_deg: format!("{:.2}", r.pitch_step_deg),
        start_pole_offset_deg: format!("{:.2}", r.start_pole_offset_deg),
        closing_turn_deg: format!("{:.2}", r.closing_turn_deg),
        drift_deg: format!("{:.2}", r.drift_deg),
        unaligned: r.unaligned.iter().copied().map(name).collect(),
        dropped_steps: r
            .dropped_steps
            .iter()
            .map(|[twin, duplicate]| format!("{} = {}", name(*twin), name(*duplicate)))
            .collect(),
    };
    insta::assert_debug_snapshot!(slug, summary);

    // The JPEG as the store writes it, so the reference is a real panorama.jpg.
    // The writer adds the panorama block to session.json, so it gets a copy.
    let out = tempfile::tempdir().unwrap();
    std::fs::copy(folder.join("session.json"), out.path().join("session.json")).unwrap();
    let clock = FrozenClock("2026-11-15T12:00:00+01:00[+01:00]".parse().unwrap());
    write_panorama(out.path(), &stitched, 100, "regression", started, &clock).unwrap();
    let produced = out.path().join(panorama::FILE);
    let reference = fixtures.join("expected").join(format!("{slug}.jpg"));
    if std::env::var_os(WRITE).is_some() {
        std::fs::create_dir_all(reference.parent().unwrap()).unwrap();
        std::fs::copy(&produced, &reference).unwrap();
        eprintln!("{slug}: reference written to {}", reference.display());
        return;
    }
    assert!(
        reference.is_file(),
        "{slug}: no reference at {}; `just stitch-regress-accept {slug}` writes one",
        reference.display()
    );
    let (a, b) = (decode(&produced), decode(&reference));
    assert_eq!(
        a.dimensions(),
        b.dimensions(),
        "{slug}: the Panorama changed size"
    );
    let (mean, far) = distance(&a, &b);
    eprintln!(
        "{slug}: mean difference {mean:.4} levels · {:.4} % of the pixels moved by more than {FAR_LEVELS}",
        far * 100.0
    );
    assert!(
        mean <= MEAN_TOLERANCE,
        "{slug}: the Panorama drifted from its reference by {mean:.3} levels on average (tolerance {MEAN_TOLERANCE})"
    );
    assert!(
        far <= FAR_TOLERANCE,
        "{slug}: {:.3} % of the pixels moved by more than {FAR_LEVELS} levels (tolerance {:.3} %)",
        far * 100.0,
        FAR_TOLERANCE * 100.0
    );
}

macro_rules! real_session {
    ($name:ident, $slug:literal) => {
        #[test]
        #[ignore = "a real Session of the sibling fixtures repository: `just stitch-regress`"]
        fn $name() {
            check($slug);
        }
    };
}

// NoClip in a browser, 5 × 6, around 89° of field of view. Two of them had
// one sky Shot unaligned until the matcher chose its centres by texture.
real_session!(noclip_2026_11_14_12_58_50, "noclip-2026-11-14_12-58-50");
real_session!(noclip_2026_11_14_13_50_29, "noclip-2026-11-14_13-50-29");
real_session!(noclip_2026_11_14_13_52_35, "noclip-2026-11-14_13-52-35");
// NoClip over a World of Warcraft map, 5 × 7: matches PTGui's result.
real_session!(
    noclip_wow_2026_11_14_13_11_28,
    "noclip-wow-2026-11-14_13-11-28"
);
// Mario Kart in NoClip on a MacBook screen, 7 × 14: sky Shots in Rows 6 and
// 7 that only the grid can place, fourteen at first, four since the matcher
// chose its centres by texture.
real_session!(
    noclip_generic_macbook_2026_11_01_13_42_30,
    "noclip-generic-macbook-2026-11-01_13-42-30"
);
// World of Warcraft, 7 × 14: two Shots dropped where the camera did not move;
// matches PTGui's result.
real_session!(
    world_of_warcraft_2026_11_07_23_14_46,
    "world-of-warcraft-2026-11-07_23-14-46"
);
// Rainbow Six Siege on Windows, 5 × 7 at 91°: a parking garage of white
// walls and grey floor, where the Rows' closing pairs had one match each
// and the front seam broke by 20 px before the matcher's texture cells.
real_session!(r6_2026_11_15_14_31_29, "r6-2026-11-15_14-31-29");
