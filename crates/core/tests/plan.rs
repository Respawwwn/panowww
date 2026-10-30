//! Capture plans: a snapshot of every run kind for every sample Game, and
//! the invariants of spec §7.1, §7.4 and §7.6 for any valid Game.

mod samples;
mod strategies;

use std::time::Duration;

use panowww_core::constants::CAPTURE_COST;
use panowww_core::game::Game;
use panowww_core::plan::{
    Axis, CapturePlan, Motion, MouseDelta, Op, RunKind, SampleSlot, Step, estimate,
    format_duration, plan,
};
use proptest::prelude::*;
use strategies::valid_game;

const KINDS: [(&str, RunKind); 7] = [
    ("session", RunKind::Session),
    ("turn-test", RunKind::TurnTest),
    ("pitch-test", RunKind::PitchTest),
    ("step-once-turn", RunKind::StepOnce(Axis::Turn)),
    ("step-once-pitch", RunKind::StepOnce(Axis::Pitch)),
    ("check-overlap-turn", RunKind::CheckOverlap(Axis::Turn)),
    ("check-overlap-pitch", RunKind::CheckOverlap(Axis::Pitch)),
];

fn millis(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

#[test]
fn every_plan_of_every_sample_game() {
    for (device, game) in samples::all() {
        for (kind, run_kind) in KINDS {
            insta::assert_snapshot!(
                format!("{device}-{kind}"),
                plan(&game, run_kind).to_string()
            );
        }
        let session = plan(&game, RunKind::Session);
        let total = estimate(&session, Duration::from_secs(5));
        insta::assert_snapshot!(
            format!("{device}-estimate"),
            format!("{} ({} ms)", format_duration(total), total.as_millis())
        );
    }
}

#[test]
fn the_spec_example_estimate() {
    // Elden Ring sample: drag 840 / 610, 13 × 6, 300 / 300 ms, 5 s countdown.
    let game = samples::mouse();
    let turn = 42 * 8 + 40;
    let pitch = 31 * 8 + 40;
    let expected = 5000 + 13 * 6 * (300 + turn + 300 + 50) + 5 * (pitch + 300);
    let total = estimate(&plan(&game, RunKind::Session), Duration::from_secs(5));
    assert_eq!(total, Duration::from_millis(expected));
    assert_eq!(format_duration(total), "≈ 1 min 28 s");
}

fn test_plans(game: &Game) -> impl Iterator<Item = CapturePlan> + '_ {
    KINDS
        .into_iter()
        .filter(|(_, kind)| *kind != RunKind::Session)
        .map(move |(_, kind)| plan(game, kind))
}

fn step_time(step: &Step) -> Duration {
    match &step.motion {
        Motion::KeyHold { hold, .. } | Motion::StickHold { hold, .. } => *hold,
        Motion::MouseMove(MouseDelta { dx, dy }) => Duration::from_millis(
            8 * u64::from(dx.unsigned_abs().max(dy.unsigned_abs()).div_ceil(20)),
        ),
        Motion::MouseDrag { delta, .. } => {
            step_time(&Step {
                axis: step.axis,
                motion: Motion::MouseMove(*delta),
            }) + Duration::from_millis(40)
        }
    }
}

proptest! {
    #[test]
    fn a_session_shoots_the_whole_grid_in_order(game in valid_game()) {
        let session = plan(&game, RunKind::Session);
        let shots: Vec<(u32, u32)> = session.ops.iter().filter_map(|op| match op {
            Op::Shot { row, col } => Some((*row, *col)),
            _ => None,
        }).collect();
        let expected: Vec<(u32, u32)> = (1..=game.rows)
            .flat_map(|row| (1..=game.shots_per_row).map(move |col| (row, col)))
            .collect();
        prop_assert_eq!(&shots, &expected);
        prop_assert_eq!(session.expected_shots(), (game.shots_per_row * game.rows) as usize);
        prop_assert!(session.ops.iter().all(|op| !matches!(op, Op::Sample(_))));
    }

    #[test]
    fn every_row_closes_and_pitches_sit_between_rows(game in valid_game()) {
        let session = plan(&game, RunKind::Session);
        // Turns per Row, split on the pitch Steps.
        let mut turns_per_row = vec![0u32];
        for op in &session.ops {
            match op {
                Op::Move(Step { axis: Axis::Turn, .. }) => *turns_per_row.last_mut().unwrap() += 1,
                Op::Move(Step { axis: Axis::Pitch, .. }) => turns_per_row.push(0),
                _ => {}
            }
        }
        prop_assert_eq!(turns_per_row.len(), game.rows as usize, "R − 1 pitch Steps");
        prop_assert!(turns_per_row.iter().all(|t| *t == game.shots_per_row), "{:?}", turns_per_row);
        // The run ends on the closing turn's wait, not on a pitch.
        let tail = &session.ops[session.ops.len() - 2..];
        let ends_on_closing_turn = matches!(tail, [Op::Move(Step { axis: Axis::Turn, .. }), Op::Wait(_)]);
        prop_assert!(ends_on_closing_turn, "{:?}", tail);
    }

    #[test]
    fn waits_follow_shots_and_moves_with_their_own_delay(game in valid_game()) {
        let after_shot = millis(game.wait_after_shot_ms);
        let after_move = millis(game.wait_after_move_ms);
        let session = plan(&game, RunKind::Session);
        for pair in session.ops.windows(2) {
            match pair {
                [Op::Shot { .. }, Op::Wait(w)] => prop_assert_eq!(*w, after_shot),
                [Op::Move(_), Op::Wait(w)] => prop_assert_eq!(*w, after_move),
                [Op::Shot { .. } | Op::Move(_), other] => prop_assert!(false, "{} must be followed by a wait, not {}", pair[0], other),
                [Op::Wait(_) | Op::Sample(_), Op::Wait(_)] => prop_assert!(false, "a wait only follows a Shot or a Step"),
                _ => {}
            }
        }
    }

    #[test]
    fn tests_take_no_shot_and_never_wait_after_shot(game in valid_game()) {
        let after_move = millis(game.wait_after_move_ms);
        for test in test_plans(&game) {
            prop_assert_eq!(test.expected_shots(), 0, "{}", test.kind);
            for pair in test.ops.windows(2) {
                match pair {
                    [Op::Move(_), Op::Wait(w)] => prop_assert_eq!(*w, after_move),
                    [_, Op::Wait(_)] => prop_assert!(false, "{}: a wait only follows a Step", test.kind),
                    _ => {}
                }
            }
            let samples = test.ops.iter().filter(|op| matches!(op, Op::Sample(_))).count();
            prop_assert_eq!(samples, 2, "{}", test.kind);
            let steps = test.ops.iter().filter(|op| matches!(op, Op::Move(_))).count();
            let expected_steps = match test.kind {
                RunKind::TurnTest => game.shots_per_row,
                RunKind::PitchTest => game.rows - 1,
                _ => 1,
            };
            prop_assert_eq!(steps, expected_steps as usize, "{}", test.kind);
        }
    }

    #[test]
    fn detector_samples_bracket_the_first_step_and_overlap_samples_the_only_one(game in valid_game()) {
        for test in test_plans(&game) {
            let (before, after) = match test.kind {
                RunKind::CheckOverlap(_) => (SampleSlot::First, SampleSlot::Second),
                _ => (SampleSlot::BeforeFirstStep, SampleSlot::AfterFirstStep),
            };
            prop_assert_eq!(&test.ops[0], &Op::Sample(before), "{}", test.kind);
            prop_assert!(matches!(test.ops[1], Op::Move(_)));
            prop_assert!(matches!(test.ops[2], Op::Wait(_)));
            prop_assert_eq!(&test.ops[3], &Op::Sample(after), "{}", test.kind);
        }
    }

    #[test]
    fn the_estimate_is_the_spec_formula(game in valid_game(), countdown_ms in 0u64..10_000) {
        let countdown = Duration::from_millis(countdown_ms);
        let session = plan(&game, RunKind::Session);
        let turn = step_time(&Step::for_game(&game, Axis::Turn));
        let pitch = step_time(&Step::for_game(&game, Axis::Pitch));
        let (n, r) = (game.shots_per_row, game.rows);
        let formula = countdown
            + n * r * (millis(game.wait_after_shot_ms) + turn + millis(game.wait_after_move_ms) + CAPTURE_COST)
            + (r - 1) * (pitch + millis(game.wait_after_move_ms));
        prop_assert_eq!(estimate(&session, countdown), formula);
        prop_assert_eq!(session.duration(), session.ops.iter().map(Op::cost).sum::<Duration>());
    }

    #[test]
    fn sub_moves_are_bounded_and_sum_to_the_delta(dx in -100_000i32..=100_000, dy in -100_000i32..=100_000) {
        let delta = MouseDelta { dx, dy };
        let moves = delta.sub_moves();
        let longest = dx.unsigned_abs().max(dy.unsigned_abs());
        prop_assert_eq!(moves.len(), longest.div_ceil(20) as usize);
        prop_assert_eq!(moves.iter().map(|m| m.dx).sum::<i32>(), dx);
        prop_assert_eq!(moves.iter().map(|m| m.dy).sum::<i32>(), dy);
        prop_assert!(moves.iter().all(|m| m.dx.unsigned_abs() <= 20 && m.dy.unsigned_abs() <= 20));
    }
}
