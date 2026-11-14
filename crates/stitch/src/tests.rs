//! The stages of this issue against the synthetic Session.

use panowww_core::game::StartPole;
use panowww_core::ports::{CancelFlag, Frame, StoreError};

use crate::geom::Cam;
use crate::synthetic::{MemorySource, Perturbation, session};
use crate::{
    Grid, Limits, ShotSource, StitchError, StitchProgress, align, dropped, focal, grid, levels,
    stitch,
};

fn small_grid() -> Grid {
    Grid {
        rows: 7,
        shots_per_row: 14,
        start_pole: StartPole::Nadir,
    }
}

fn no_progress(_: StitchProgress) {}

#[test]
fn the_field_of_view_is_found_within_one_percent() {
    let grid = small_grid();
    let (source, truth) = session(640, 320, 90.0, &grid, &Perturbation::none(&grid));
    let pyramids = levels::Pyramids::build(
        &source,
        &grid,
        Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap();
    let drops = dropped::detect(&pyramids);
    assert_eq!(drops.pairs, Vec::<(usize, usize)>::new());
    let params = grid::GridParams::nominal(&pyramids, &grid, &drops.drops, 1.0);
    let found = focal::estimate(&pyramids, &grid, &params.poses());
    let error = (found.f_px - truth.f_px).abs() / truth.f_px;
    assert!(
        error < 0.01,
        "f {} vs {} ({:.1} %)",
        found.f_px,
        truth.f_px,
        error * 100.0
    );
    assert!(found.ncc > 0.8);
}

#[test]
fn the_field_of_view_survives_the_hard_capture() {
    let grid = small_grid();
    let (source, truth) = session(640, 320, 90.0, &grid, &Perturbation::hard(&grid));
    let pyramids = levels::Pyramids::build(
        &source,
        &grid,
        Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap();
    let drops = dropped::detect(&pyramids);
    let params = grid::GridParams::nominal(&pyramids, &grid, &drops.drops, 1.0);
    let found = focal::estimate(&pyramids, &grid, &params.poses());
    // The turn Step is 26.7° where the grid says 25.7°: the horizon Row's
    // prior is one degree off per Shot, the search still lands within 2 %.
    let error = (found.f_px - truth.f_px).abs() / truth.f_px;
    assert!(
        error < 0.02,
        "f {} vs {} ({:.1} %)",
        found.f_px,
        truth.f_px,
        error * 100.0
    );
}

#[test]
fn a_dropped_step_is_detected_and_the_grid_prior_moves_back() {
    let grid = small_grid();
    let (source, _) = session(320, 160, 90.0, &grid, &Perturbation::hard(&grid));
    let pyramids = levels::Pyramids::build(
        &source,
        &grid,
        Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap();
    let drops = dropped::detect(&pyramids);
    let twin = pyramids.index_of(4, 7).unwrap();
    let duplicate = pyramids.index_of(4, 8).unwrap();
    assert_eq!(drops.pairs, vec![(twin, duplicate)]);
    assert_eq!(drops.drops[duplicate], 1);
    assert_eq!(drops.drops[pyramids.index_of(4, 14).unwrap()], 1);
    assert_eq!(drops.drops[pyramids.index_of(4, 6).unwrap()], 0);
    assert_eq!(drops.drops[pyramids.index_of(5, 8).unwrap()], 0);
    let params = grid::GridParams::nominal(&pyramids, &grid, &drops.drops, 1.0);
    assert!((params.yaw[duplicate] - params.yaw[twin]).abs() < 1e-9);
    let step = std::f64::consts::TAU / 14.0;
    assert!((params.yaw[pyramids.index_of(4, 9).unwrap()] - params.yaw[twin] - step).abs() < 1e-9);
}

#[test]
fn the_pyramids_keep_the_shots_grey_and_small() {
    let grid = small_grid();
    let (source, _) = session(320, 160, 90.0, &grid, &Perturbation::none(&grid));
    let reports = std::sync::Mutex::new(Vec::new());
    let pyramids = levels::Pyramids::build(
        &source,
        &grid,
        Limits::default(),
        &|p| reports.lock().unwrap().push(p),
        &CancelFlag::new(),
    )
    .unwrap();
    let reports = reports.into_inner().unwrap();
    assert_eq!(pyramids.count(), 98);
    let s = pyramids.shot(0);
    assert_eq!(
        (s.half.w, s.half.h, s.quarter.w, s.eighth.w),
        (160, 80, 80, 40)
    );
    assert_eq!(pyramids.neighbour_pairs(&grid).len(), 98 + 84);
    assert!(reports.first().is_some_and(|p| p.overall == 0.0));
    assert!(
        reports
            .last()
            .is_some_and(|p| (p.stage_fraction - 1.0).abs() < 1e-6)
    );
}

struct Raising {
    inner: MemorySource,
    cancel: CancelFlag,
}

impl ShotSource for Raising {
    fn count(&self) -> usize {
        self.inner.count()
    }
    fn position(&self, index: usize) -> (u32, u32) {
        self.inner.position(index)
    }
    fn shot(&self, index: usize) -> Result<Frame, StoreError> {
        self.cancel.raise();
        self.inner.shot(index)
    }
}

#[test]
fn a_cancel_during_read_stops_the_stitch() {
    let grid = small_grid();
    let (inner, _) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    let cancel = CancelFlag::new();
    let source = Raising {
        inner,
        cancel: cancel.clone(),
    };
    let result = stitch(&source, &grid, &Limits::default(), &no_progress, &cancel);
    assert_eq!(result.unwrap_err(), StitchError::Cancelled);
}

#[test]
fn a_session_that_does_not_cover_its_grid_is_refused() {
    let grid = small_grid();
    let (mut source, _) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    source.shots.pop();
    let err = stitch(
        &source,
        &grid,
        &Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(matches!(err, StitchError::Grid(m) if m.contains("97 Shots")));
    let (mut source, _) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    source.shots[3].2 = Frame::solid(80, 80, [0, 0, 0]);
    let err = stitch(
        &source,
        &grid,
        &Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(matches!(err, StitchError::Grid(m) if m.contains("80 × 80")));
}

#[test]
fn a_budget_too_small_is_refused_before_reading() {
    let grid = small_grid();
    let (source, _) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    let err = stitch(
        &source,
        &grid,
        &Limits { memory_bytes: 1000 },
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(matches!(err, StitchError::Memory { .. }));
}

/// Peak signal-to-noise ratio of the Panorama against the world it was
/// rendered from, in dB, over the latitudes `lat_min..lat_max`.
fn psnr_against_world(panorama: &Frame, lat_min: f64, lat_max: f64) -> f64 {
    let (w, h) = (panorama.width as usize, panorama.height as usize);
    let (wf, hf) = (f64::from(panorama.width), f64::from(panorama.height));
    let mut se = 0f64;
    let mut n = 0f64;
    for y in 0..h {
        let lat = (0.5 - (f64::from(u32::try_from(y).unwrap()) + 0.5) / hf) * 180.0;
        if lat < lat_min || lat > lat_max {
            continue;
        }
        for x in 0..w {
            let lon = ((f64::from(u32::try_from(x).unwrap()) + 0.5) / wf - 0.5) * 360.0;
            let truth = crate::synthetic::world(lon, lat);
            let px = &panorama.rgb[(y * w + x) * 3..(y * w + x) * 3 + 3];
            for (p, t) in px.iter().zip(truth) {
                let d = f64::from(*p) - f64::from(t);
                se += d * d;
                n += 1.0;
            }
        }
    }
    10.0 * (255.0f64 * 255.0 / (se / n).max(1e-9)).log10()
}

#[test]
fn a_capture_on_its_grid_renders_its_world() {
    let grid = small_grid();
    let (source, truth) = session(640, 320, 90.0, &grid, &Perturbation::none(&grid));
    let started = std::time::Instant::now();
    let reports = std::sync::Mutex::new(Vec::new());
    let stitched = stitch(
        &source,
        &grid,
        &Limits::default(),
        &|p| reports.lock().unwrap().push(p),
        &CancelFlag::new(),
    )
    .unwrap();
    eprintln!(
        "stitched 98 Shots of 640 × 320 into {} × {} in {:.2} s",
        stitched.panorama.width,
        stitched.panorama.height,
        started.elapsed().as_secs_f64()
    );
    let (w, h) = crate::render::canvas_size(truth.f_px);
    assert_eq!(
        (
            stitched.panorama.width as usize,
            stitched.panorama.height as usize
        ),
        (w, h)
    );
    assert!(w.is_multiple_of(64) && h * 2 == w);
    assert!(stitched.panorama.is_well_formed());
    // The world comes back: the alignment is sub-pixel and the seams land on
    // agreeing pixels, so the only loss is resampling.
    let psnr = psnr_against_world(&stitched.panorama, -60.0, 60.0);
    eprintln!("PSNR against the world: {psnr:.1} dB");
    assert!(psnr > 30.0, "PSNR {psnr:.1} dB");
    // The preview is the Panorama itself under 8192 px.
    assert_eq!(stitched.preview.width, stitched.panorama.width);
    let r = &stitched.report;
    assert_eq!(
        (r.width, r.height),
        (stitched.panorama.width, stitched.panorama.height)
    );
    assert!(r.unaligned.is_empty() && r.dropped_steps.is_empty() && r.anti_ghost);
    assert!(
        r.durations.read_ms
            + r.durations.overlaps_ms
            + r.durations.align_ms
            + r.durations.render_ms
            > 0
    );
    // Progress never goes backwards and ends on the Blend stage; the Save
    // stage is the store's.
    let reports = reports.into_inner().unwrap();
    let overall: Vec<f32> = reports.iter().map(|p| p.overall).collect();
    assert!(
        overall.windows(2).all(|w| w[1] >= w[0] - 1e-6),
        "progress went backwards: {overall:?}"
    );
    assert!(
        overall.last().copied().unwrap_or(0.0) >= 0.96,
        "{overall:?}"
    );
    assert!(
        reports
            .iter()
            .any(|p| matches!(p.stage, crate::Stage::Render))
    );
}

#[test]
fn the_hard_capture_renders_and_reports_its_defects() {
    let grid = small_grid();
    let (source, _) = session(640, 320, 90.0, &grid, &Perturbation::hard(&grid));
    let stitched = stitch(
        &source,
        &grid,
        &Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap();
    let psnr = psnr_against_world(&stitched.panorama, -60.0, 60.0);
    eprintln!("hard capture PSNR against the world: {psnr:.1} dB");
    assert!(psnr > 28.0, "PSNR {psnr:.1} dB");
    let r = &stitched.report;
    assert_eq!(
        r.unaligned
            .iter()
            .map(|s| (s.row, s.col))
            .collect::<Vec<_>>(),
        vec![(7, 2), (7, 5)]
    );
    assert_eq!(r.dropped_steps.len(), 1);
    assert_eq!(
        (
            r.dropped_steps[0][0].row,
            r.dropped_steps[0][0].col,
            r.dropped_steps[0][1].col
        ),
        (4, 7, 8)
    );
}

#[test]
fn a_tight_budget_renders_by_narrow_bands() {
    let grid = small_grid();
    let (source, _) = session(320, 160, 90.0, &grid, &Perturbation::none(&grid));
    // Enough for the pyramids and a few frames, not for a tall band.
    let limits = Limits {
        memory_bytes: 24 * 1024 * 1024,
    };
    let stitched = stitch(&source, &grid, &limits, &no_progress, &CancelFlag::new()).unwrap();
    assert!(stitched.panorama.is_well_formed());
    let psnr = psnr_against_world(&stitched.panorama, -60.0, 60.0);
    eprintln!("narrow bands PSNR against the world: {psnr:.1} dB");
    assert!(psnr > 28.0, "PSNR {psnr:.1} dB");
}

#[test]
fn the_preview_is_reduced_to_at_most_8192_wide() {
    let big = Frame::solid(9000, 4500, [10, 20, 30]);
    let p = crate::render::preview_of(&big);
    assert!(p.width <= 8192 && p.width == p.height * 2 && p.rgb[..3] == [10, 20, 30]);
    let small = Frame::solid(640, 320, [1, 2, 3]);
    assert_eq!(crate::render::preview_of(&small), small);
}

fn aligned(
    width: u32,
    height: u32,
    perturbation: &Perturbation,
) -> (align::Alignment, crate::synthetic::Truth, levels::Pyramids) {
    let grid = small_grid();
    let (source, truth) = session(width, height, 90.0, &grid, perturbation);
    let pyramids = levels::Pyramids::build(
        &source,
        &grid,
        Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap();
    let drops = dropped::detect(&pyramids);
    let a = align::align(&pyramids, &grid, &drops, &no_progress, &CancelFlag::new()).unwrap();
    (a, truth, pyramids)
}

/// The rotation that best maps our optical axes onto the truth's (Kabsch),
/// over the Shots `keep` marks: the heading the anchor fixes is arbitrary.
fn best_fix(
    ours: &[crate::geom::Rot],
    truth: &[crate::geom::Rot],
    keep: &[bool],
) -> crate::geom::Rot {
    let mut h = nalgebra::Matrix3::<f64>::zeros();
    for (i, (o, t)) in ours.iter().zip(truth).enumerate() {
        if keep[i] {
            h += crate::geom::axis(t) * crate::geom::axis(o).transpose();
        }
    }
    let svd = h.svd(true, true);
    let (u, v_t) = (svd.u.unwrap(), svd.v_t.unwrap());
    let mut d = nalgebra::Matrix3::<f64>::identity();
    if (u * v_t).determinant() < 0.0 {
        d[(2, 2)] = -1.0;
    }
    crate::geom::Rot::from_matrix_unchecked(u * d * v_t)
}

/// Angle between two poses' optical axes, degrees. A pole Shot's roll is
/// not in it: flat sky cannot pin a roll, and the pole hides it.
fn apart(a: &crate::geom::Rot, b: &crate::geom::Rot) -> f64 {
    let (za, zb) = (crate::geom::axis(a), crate::geom::axis(b));
    za.dot(&zb).clamp(-1.0, 1.0).acos().to_degrees()
}

/// Full rotation between two poses, degrees: roll included.
fn apart_with_roll(a: &crate::geom::Rot, b: &crate::geom::Rot) -> f64 {
    (a * b.inverse()).angle().to_degrees()
}

#[test]
fn a_capture_on_its_grid_aligns_to_the_truth() {
    let started = std::time::Instant::now();
    let (a, truth, _) = aligned(640, 320, &Perturbation::none(&small_grid()));
    eprintln!(
        "aligned 98 Shots of 640 × 320 in {:.2} s",
        started.elapsed().as_secs_f64()
    );
    assert!(a.unaligned.is_empty(), "{:?}", a.unaligned);
    // The synthetic world is point-sampled, so its edges alias: three
    // pixels is the noise floor of the patch correlation on it.
    assert!(a.residual_px < 3.0, "residual {}", a.residual_px);
    assert!(
        (a.measured.field_of_view - 90.0).abs() < 1.0,
        "{:?}",
        a.measured
    );
    assert!(
        (a.measured.turn_step - 360.0 / 14.0).abs() < 0.3,
        "{:?}",
        a.measured
    );
    assert!(
        (a.measured.pitch_step - 30.0).abs() < 0.3,
        "{:?}",
        a.measured
    );
    assert!(a.measured.start_pole_offset.abs() < 0.5, "{:?}", a.measured);
    // Every pose within half a degree of the truth, up to the global heading
    // the anchor fixes.
    let fix = best_fix(&a.poses, &truth.poses, &vec![true; a.poses.len()]);
    let worst = a
        .poses
        .iter()
        .zip(&truth.poses)
        .map(|(p, t)| apart(&(fix * p), t))
        .fold(0.0, f64::max);
    assert!(worst < 0.5, "worst axis error {worst:.2}°");
    // Away from the poles the roll is pinned too.
    let worst_roll = a
        .poses
        .iter()
        .zip(&truth.poses)
        .filter(|(_, t)| crate::geom::yaw_pitch(t).1.to_degrees().abs() < 75.0)
        .map(|(p, t)| apart_with_roll(&(fix * p), t))
        .fold(0.0, f64::max);
    assert!(
        worst_roll < 0.7,
        "worst pose error with roll {worst_roll:.2}°"
    );
}

#[test]
fn the_hard_capture_is_measured_not_trusted() {
    let grid = small_grid();
    let (a, truth, pyramids) = aligned(640, 320, &Perturbation::hard(&grid));
    // The two flat sky Shots have no control points; everything else links.
    let mut unaligned = align::shot_refs(&pyramids, &a.unaligned);
    unaligned.sort_by_key(|s| (s.row, s.col));
    assert_eq!(
        unaligned.iter().map(|s| (s.row, s.col)).collect::<Vec<_>>(),
        vec![(7, 2), (7, 5)]
    );
    assert!(a.residual_px < 3.0, "residual {}", a.residual_px);
    assert!(
        (a.measured.turn_step - 26.7).abs() < 0.3,
        "{:?}",
        a.measured
    );
    assert!(
        (a.measured.pitch_step - 29.0).abs() < 0.3,
        "{:?}",
        a.measured
    );
    assert!(
        (a.measured.start_pole_offset - 4.0).abs() < 0.5,
        "{:?}",
        a.measured
    );
    // Row 5 starts 40° further round than the grid says: recovered.
    let keep: Vec<bool> = (0..a.poses.len())
        .map(|i| !a.unaligned.contains(&i))
        .collect();
    let fix = best_fix(&a.poses, &truth.poses, &keep);
    let worst_linked = a
        .poses
        .iter()
        .zip(&truth.poses)
        .enumerate()
        .filter(|(i, _)| !a.unaligned.contains(i))
        .map(|(_, (p, t))| apart(&(fix * p), t))
        .fold(0.0, f64::max);
    for (i, (p, t)) in a.poses.iter().zip(&truth.poses).enumerate() {
        let e = apart(&(fix * p), t);
        let (row, col) = pyramids.position(i);
        if e > 2.0 {
            let (y, pt) = crate::geom::yaw_pitch(&(fix * p));
            let (ty, tp) = crate::geom::yaw_pitch(t);
            eprintln!(
                "r{row:02}_c{col:02}: axis off by {e:.1}° (yaw {:.1} vs {:.1}, pitch {:.1} vs {:.1}){}",
                y.to_degrees(),
                ty.to_degrees(),
                pt.to_degrees(),
                tp.to_degrees(),
                if a.unaligned.contains(&i) {
                    " unaligned"
                } else {
                    ""
                }
            );
        }
    }
    // The zenith Shots look at flat sky and are pinned by their bottom edge
    // alone: one degree there is the pole's noise floor.
    assert!(
        worst_linked < 1.0,
        "worst linked axis error {worst_linked:.2}°"
    );
    let worst_roll = a
        .poses
        .iter()
        .zip(&truth.poses)
        .filter(|(_, t)| crate::geom::yaw_pitch(t).1.to_degrees().abs() < 75.0)
        .map(|(p, t)| apart_with_roll(&(fix * p), t))
        .fold(0.0, f64::max);
    assert!(
        worst_roll < 1.0,
        "worst pose error with roll {worst_roll:.2}°"
    );
    insta::assert_debug_snapshot!(
        "hard_capture_measured",
        (
            (a.measured.field_of_view * 10.0).round() / 10.0,
            (a.measured.turn_step * 10.0).round() / 10.0,
            (a.measured.pitch_step * 10.0).round() / 10.0,
            (a.measured.start_pole_offset * 10.0).round() / 10.0,
            unaligned.len(),
        )
    );
}

#[test]
fn an_all_flat_capture_has_too_few_links() {
    let grid = small_grid();
    let mut p = Perturbation::none(&grid);
    p.flat_shots = (1..=grid.rows)
        .flat_map(|r| (1..=grid.shots_per_row).map(move |c| (r, c)))
        .collect();
    let (source, _) = session(160, 80, 90.0, &grid, &p);
    let err = stitch(
        &source,
        &grid,
        &Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(
        matches!(err, StitchError::TooFewLinked { total: 98, .. }),
        "{err:?}"
    );
}

#[test]
fn the_synthetic_truth_matches_its_own_camera() {
    let grid = small_grid();
    let (source, truth) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    assert_eq!(source.count(), 98);
    assert!((Cam::new(truth.f_px, 160, 80).hfov_deg() - 90.0).abs() < 1e-9);
}
