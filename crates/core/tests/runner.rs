//! The Runner through the fakes: whole Sessions of the sample Games, stop
//! semantics, Shot retry and the Samples of Check overlap.

mod samples;

use std::cell::RefCell;
use std::time::Duration;

use panowww_core::constants::CAPTURE_COST;
use panowww_core::fakes::{CameraCall, FakeClock, FakeStream, MemorySink, RecordingCamera};
use panowww_core::game::Game;
use panowww_core::plan::{Axis, CapturePlan, Op, RunKind, SampleSlot, plan};
use panowww_core::ports::{CancelFlag, DisplayId, Rect, Zone};
use panowww_core::runner::{Progress, RunErrorKind, RunOutcome, RunRequest, SessionStart, run};

fn zone() -> Zone {
    Zone {
        display_id: DisplayId("main".to_owned()),
        x: 100,
        y: 50,
        w: 1600,
        h: 900,
        display_w: 2560,
        display_h: 1440,
        scale: 2.0,
    }
}

fn session_start(game: &Game, plan: &CapturePlan) -> SessionStart {
    SessionStart {
        game: game.clone(),
        zone: zone(),
        shots_expected: u32::try_from(plan.expected_shots()).unwrap(),
    }
}

/// The fakes of one run and what came out of it.
struct Run {
    camera: RecordingCamera,
    stream: FakeStream,
    sink: MemorySink,
    clock: FakeClock,
    events: Vec<Progress>,
    outcome: RunOutcome,
}

fn run_with(
    plan: &CapturePlan,
    session: Option<SessionStart>,
    mut camera: RecordingCamera,
    mut stream: FakeStream,
    mut sink: MemorySink,
    clock: FakeClock,
) -> Run {
    let request = RunRequest {
        plan,
        zone: zone().rect(),
        session,
    };
    let events = RefCell::new(Vec::new());
    let outcome = run(
        &request,
        &mut camera,
        &mut stream,
        &mut sink,
        &clock,
        &CancelFlag::new(),
        &|event| events.borrow_mut().push(event),
    );
    Run {
        camera,
        stream,
        sink,
        clock,
        events: events.into_inner(),
        outcome,
    }
}

fn run_session(game: &Game, stream: FakeStream, clock: FakeClock) -> (CapturePlan, Run) {
    let plan = plan(game, RunKind::Session);
    let start = session_start(game, &plan);
    let run = run_with(
        &plan,
        Some(start),
        RecordingCamera::new(),
        stream,
        MemorySink::new(),
        clock,
    );
    (plan, run)
}

fn run_tool(game: &Game, kind: RunKind) -> (CapturePlan, Run) {
    let plan = plan(game, kind);
    let run = run_with(
        &plan,
        None,
        RecordingCamera::new(),
        FakeStream::new(64, 36),
        MemorySink::new(),
        FakeClock::new(),
    );
    (plan, run)
}

fn grid_order(plan: &CapturePlan) -> Vec<(u32, u32)> {
    plan.ops
        .iter()
        .filter_map(|op| match op {
            Op::Shot { row, col } => Some((*row, *col)),
            _ => None,
        })
        .collect()
}

fn moves(plan: &CapturePlan) -> usize {
    plan.ops
        .iter()
        .filter(|op| matches!(op, Op::Move(_)))
        .count()
}

#[test]
fn a_full_session_of_each_sample_game() {
    for (device, game) in samples::all() {
        let (plan, run) = run_session(&game, FakeStream::new(64, 36), FakeClock::new());
        let expected = u32::try_from(plan.expected_shots()).unwrap();
        assert_eq!(
            run.outcome,
            RunOutcome::Completed { shots: expected },
            "{device}"
        );

        // Everything reached the store, in grid order, and the Session was closed.
        assert_eq!(
            run.sink.begun.as_ref().map(|s| s.shots_expected),
            Some(expected)
        );
        let stored: Vec<(u32, u32)> = run.sink.shots.iter().map(|(r, _)| (r.row, r.col)).collect();
        assert_eq!(stored, grid_order(&plan), "{device}");
        assert!(
            run.sink
                .shots
                .iter()
                .enumerate()
                .all(|(i, (r, _))| r.index as usize == i + 1)
        );
        assert_eq!(run.sink.finished, Some(run.outcome.clone()));
        assert_eq!(run.stream.calls, expected);

        // The clock saw every wait of the plan and nothing else.
        assert_eq!(
            run.clock.total_waited(),
            plan.duration()
                .checked_sub(expected * CAPTURE_COST)
                .unwrap(),
            "{device}"
        );

        // Events: Started, one Shot with a thumbnail per Shot, one Moved per Step, Finished last.
        assert!(
            matches!(run.events.first(), Some(Progress::Started { expected: e }) if *e == expected)
        );
        let shots = run
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Progress::Shot {
                        thumbnail: Some(_),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(shots, expected as usize, "{device}");
        let moved = run
            .events
            .iter()
            .filter(|e| matches!(e, Progress::Moved { .. }))
            .count();
        assert_eq!(moved, moves(&plan), "{device}");
        assert!(matches!(run.events.last(), Some(Progress::Finished(o)) if *o == run.outcome));

        assert_eq!(run.camera.calls.last(), Some(&CameraCall::ReleaseAll));
        insta::assert_snapshot!(format!("{device}-session"), run.camera.transcript());
    }
}

#[test]
fn moved_events_track_the_grid_position() {
    let game = samples::keyboard();
    let (plan, run) = run_session(&game, FakeStream::new(8, 8), FakeClock::new());
    let positions: Vec<(u32, u32)> = run
        .events
        .iter()
        .filter_map(|e| match e {
            Progress::Moved { row, col } => Some((*row, *col)),
            _ => None,
        })
        .collect();
    let n = plan.grid.shots_per_row as usize;
    assert_eq!(positions[0], (1, 2));
    assert_eq!(
        positions[n - 1],
        (1, 1),
        "the closing turn wraps to the start heading"
    );
    assert_eq!(positions[n], (2, 1), "the pitch starts the next Row");
    assert_eq!(*positions.last().unwrap(), (plan.grid.rows, 1));
}

#[test]
fn cancel_during_a_wait_releases_everything_and_keeps_the_shots() {
    // Keyboard: Shot, wait(1), key_down, wait(2) hold, key_up, wait(3), Shot,
    // wait(4), key_down, wait(5) hold -> cancelled while the key is down.
    let game = samples::keyboard();
    let (_, run) = run_session(
        &game,
        FakeStream::new(8, 8),
        FakeClock::new().cancelling_on_wait(5),
    );
    assert_eq!(run.outcome, RunOutcome::Interrupted { shots: 2 });
    let calls = &run.camera.calls;
    assert_eq!(calls.last(), Some(&CameraCall::ReleaseAll));
    assert!(
        matches!(calls[calls.len() - 2], CameraCall::KeyDown(_)),
        "{calls:?}"
    );
    assert_eq!(run.sink.shots.len(), 2);
    assert_eq!(
        run.sink.finished,
        Some(RunOutcome::Interrupted { shots: 2 })
    );
    assert!(matches!(
        run.events.last(),
        Some(Progress::Finished(RunOutcome::Interrupted { shots: 2 }))
    ));
}

#[test]
fn one_failing_shot_is_retried_after_a_fresh_wait_after_shot() {
    let game = samples::mouse();
    let (plan, run) = run_session(
        &game,
        FakeStream::new(8, 8).failing_on([3]),
        FakeClock::new(),
    );
    let expected = u32::try_from(plan.expected_shots()).unwrap();
    assert_eq!(run.outcome, RunOutcome::Completed { shots: expected });
    assert_eq!(run.stream.calls, expected + 1, "one extra capture");
    assert_eq!(run.sink.shots.len(), expected as usize);
    // The third Shot holds the fourth frame, and the grid has no hole.
    assert_eq!(FakeStream::frame_number(&run.sink.shots[2].1), 4);
    let stored: Vec<(u32, u32)> = run.sink.shots.iter().map(|(r, _)| (r.row, r.col)).collect();
    assert_eq!(stored, grid_order(&plan));
    assert_eq!(
        run.clock.total_waited(),
        plan.duration()
            .checked_sub(expected * CAPTURE_COST)
            .unwrap()
            + plan.wait_after_shot,
        "one extra waitAfterShot before the retry"
    );
}

#[test]
fn two_failing_shots_fail_the_run_without_a_hole() {
    let game = samples::gamepad();
    let (_, run) = run_session(
        &game,
        FakeStream::new(8, 8).failing_on([3, 4]),
        FakeClock::new(),
    );
    let RunOutcome::Failed { shots, error } = &run.outcome else {
        panic!("{:?}", run.outcome);
    };
    assert_eq!(*shots, 2);
    assert_eq!(error.kind, RunErrorKind::ShotFailed);
    assert_eq!(run.sink.shots.len(), 2);
    assert_eq!(run.sink.finished.as_ref(), Some(&run.outcome));
    assert_eq!(run.camera.calls.last(), Some(&CameraCall::ReleaseAll));
    assert_eq!(run.stream.calls, 4);
}

#[test]
fn an_input_failure_fails_the_run() {
    let game = samples::keyboard();
    let plan = plan(&game, RunKind::Session);
    let mut camera = RecordingCamera::new();
    camera.fail_on_call = Some(1);
    let run = run_with(
        &plan,
        Some(session_start(&game, &plan)),
        camera,
        FakeStream::new(8, 8),
        MemorySink::new(),
        FakeClock::new(),
    );
    let RunOutcome::Failed { shots, error } = &run.outcome else {
        panic!("{:?}", run.outcome);
    };
    assert_eq!((*shots, error.kind), (1, RunErrorKind::Input));
    assert_eq!(
        run.camera.calls.len(),
        2,
        "the failing key_down, then release_all"
    );
}

#[test]
fn a_store_failure_fails_the_run_and_does_not_count_the_shot() {
    let game = samples::keyboard();
    let plan = plan(&game, RunKind::Session);
    let mut sink = MemorySink::new();
    sink.fail_on_shot = Some(2);
    let run = run_with(
        &plan,
        Some(session_start(&game, &plan)),
        RecordingCamera::new(),
        FakeStream::new(8, 8),
        sink,
        FakeClock::new(),
    );
    let RunOutcome::Failed { shots, error } = &run.outcome else {
        panic!("{:?}", run.outcome);
    };
    assert_eq!((*shots, error.kind), (1, RunErrorKind::Store));
    assert_eq!(run.sink.shots.len(), 1);
}

#[test]
fn check_overlap_returns_two_samples_and_writes_nothing() {
    let game = samples::mouse();
    let (_, run) = run_tool(&game, RunKind::CheckOverlap(Axis::Turn));
    assert_eq!(run.outcome, RunOutcome::Completed { shots: 0 });
    let samples: Vec<(SampleSlot, u8)> = run
        .events
        .iter()
        .filter_map(|e| match e {
            Progress::Sample { slot, frame } => Some((*slot, FakeStream::frame_number(frame))),
            _ => None,
        })
        .collect();
    assert_eq!(samples, [(SampleSlot::First, 1), (SampleSlot::Second, 2)]);
    assert!(run.sink.begun.is_none() && run.sink.finished.is_none() && run.sink.shots.is_empty());
    assert!(
        !run.events
            .iter()
            .any(|e| matches!(e, Progress::Shot { .. }))
    );
    insta::assert_snapshot!("mouse-check-overlap-turn", run.camera.transcript());
}

#[test]
fn tests_probe_the_first_step_and_touch_no_store() {
    for (device, game) in samples::all() {
        for kind in [
            RunKind::TurnTest,
            RunKind::PitchTest,
            RunKind::StepOnce(Axis::Turn),
            RunKind::StepOnce(Axis::Pitch),
        ] {
            let (plan, run) = run_tool(&game, kind);
            assert_eq!(
                run.outcome,
                RunOutcome::Completed { shots: 0 },
                "{device} {kind}"
            );
            assert!(
                run.sink.begun.is_none() && run.sink.finished.is_none(),
                "{device} {kind}"
            );
            let slots: Vec<SampleSlot> = run
                .events
                .iter()
                .filter_map(|e| match e {
                    Progress::Sample { slot, .. } => Some(*slot),
                    _ => None,
                })
                .collect();
            assert_eq!(
                slots,
                [SampleSlot::BeforeFirstStep, SampleSlot::AfterFirstStep],
                "{device} {kind}"
            );
            assert_eq!(
                run.clock.total_waited(),
                plan.duration().checked_sub(2 * CAPTURE_COST).unwrap(),
                "{device} {kind}"
            );
        }
    }
}

#[test]
fn a_drag_step_parks_the_cursor_at_the_zone_centre() {
    let game = samples::mouse();
    let (_, run) = run_tool(&game, RunKind::StepOnce(Axis::Pitch));
    let centre = Rect {
        x: 100,
        y: 50,
        w: 1600,
        h: 900,
    }
    .centre();
    assert_eq!(run.camera.calls[0], CameraCall::ParkCursor(centre));
    assert_eq!((centre.x, centre.y), (900.0, 500.0));
    insta::assert_snapshot!("mouse-step-once-pitch", run.camera.transcript());
}

#[test]
fn shots_are_asked_for_frames_newer_than_the_last_step() {
    let game = samples::keyboard();
    let (_, run) = run_session(&game, FakeStream::new(8, 8), FakeClock::new());
    // The second Shot's not_before is when the first turn ended: after
    // waitAfterShot and the hold, before waitAfterMove.
    let first = run.stream.freshness_marks[0];
    let second = run.stream.freshness_marks[1];
    assert_eq!(second - first, Duration::from_millis(300 + 120));
}

// The screen-unchanged detector (spec §4.6). The default FakeStream fills
// consecutive frames with bytes one apart, which the detector reads as the
// same screen; a moving stream makes them differ widely.

fn screen_unchanged_positions(events: &[Progress]) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| matches!(e, Progress::ScreenUnchanged).then_some(i))
        .collect()
}

#[test]
fn a_still_screen_is_reported_once_after_the_second_shot() {
    let game = samples::keyboard();
    let (_, run) = run_session(&game, FakeStream::new(64, 36), FakeClock::new());
    let positions = screen_unchanged_positions(&run.events);
    assert_eq!(positions.len(), 1, "{positions:?}");
    let second_shot = run
        .events
        .iter()
        .position(|e| matches!(e, Progress::Shot { index: 2, .. }))
        .unwrap();
    assert_eq!(positions[0], second_shot + 1, "right after the second Shot");
    assert_eq!(
        run.outcome,
        RunOutcome::Completed { shots: 78 },
        "no automatic stop"
    );
}

#[test]
fn a_moving_screen_is_not_reported() {
    let game = samples::keyboard();
    let (_, run) = run_session(&game, FakeStream::new(64, 36).moving(), FakeClock::new());
    assert_eq!(screen_unchanged_positions(&run.events), Vec::<usize>::new());
    assert_eq!(run.outcome, RunOutcome::Completed { shots: 78 });
}

#[test]
fn tests_report_a_still_screen_after_the_first_step_only() {
    for (device, game) in samples::all() {
        for kind in [
            RunKind::TurnTest,
            RunKind::PitchTest,
            RunKind::StepOnce(Axis::Turn),
        ] {
            let (_, run) = run_tool(&game, kind);
            let positions = screen_unchanged_positions(&run.events);
            assert_eq!(positions.len(), 1, "{device} {kind}: {positions:?}");
            let after_sample = run
                .events
                .iter()
                .position(|e| {
                    matches!(
                        e,
                        Progress::Sample {
                            slot: SampleSlot::AfterFirstStep,
                            ..
                        }
                    )
                })
                .unwrap();
            assert_eq!(positions[0], after_sample + 1, "{device} {kind}");
        }
    }
}

#[test]
fn check_overlap_never_runs_the_detector() {
    let game = samples::mouse();
    let (_, run) = run_tool(&game, RunKind::CheckOverlap(Axis::Pitch));
    assert_eq!(screen_unchanged_positions(&run.events), Vec::<usize>::new());
}

#[test]
fn a_moving_screen_in_a_test_is_not_reported() {
    let game = samples::gamepad();
    let plan = plan(&game, RunKind::TurnTest);
    let run = run_with(
        &plan,
        None,
        RecordingCamera::new(),
        FakeStream::new(64, 36).moving(),
        MemorySink::new(),
        FakeClock::new(),
    );
    assert_eq!(screen_unchanged_positions(&run.events), Vec::<usize>::new());
}
