//! The stages of this issue against the synthetic Session.

use panowww_core::game::StartPole;
use panowww_core::ports::{CancelFlag, Frame, StoreError};

use crate::geom::Cam;
use crate::synthetic::{MemorySource, Perturbation, session};
use crate::{
    Grid, Limits, ShotSource, StitchError, StitchProgress, dropped, focal, grid, levels, stitch,
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

#[test]
fn the_stages_after_the_field_of_view_are_not_there_yet() {
    let grid = small_grid();
    let (source, _) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    let err = stitch(
        &source,
        &grid,
        &Limits::default(),
        &no_progress,
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(matches!(err, StitchError::Internal(_)));
}

#[test]
fn the_synthetic_truth_matches_its_own_camera() {
    let grid = small_grid();
    let (source, truth) = session(160, 80, 90.0, &grid, &Perturbation::none(&grid));
    assert_eq!(source.count(), 98);
    assert!((Cam::new(truth.f_px, 160, 80).hfov_deg() - 90.0).abs() < 1e-9);
}
