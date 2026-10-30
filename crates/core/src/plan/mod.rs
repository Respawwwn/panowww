//! The capture plan: the finite list of operations the Runner performs for
//! one run kind of a Game (spec §4.3, §7.1, §7.4). Sub-moves, holds, drag
//! press and release are expanded at run time from each [`Step`]; the plan
//! only says what happens in which order, and so is the duration estimate's
//! input ([`estimate()`]).

pub mod estimate;
pub mod step;

use std::fmt;
use std::time::Duration;

pub use estimate::{estimate, format_duration};
pub use step::{Axis, Motion, MouseDelta, Step};

use crate::constants::CAPTURE_COST;
use crate::game::Game;

/// What a run does. Only a Session takes Shots and creates a Session folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunKind {
    /// N × R Shots from the start pole.
    Session,
    /// N turn Steps; the camera must land back on its landmark.
    TurnTest,
    /// R − 1 pitch Steps; the camera must land on the opposite pole.
    PitchTest,
    /// One Step, to feel its size.
    StepOnce(Axis),
    /// Two Samples one Step apart, to judge the Overlap.
    CheckOverlap(Axis),
}

/// Which in-memory image a Sample fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleSlot {
    /// Screen-unchanged detector, just before the first Step of a Test.
    BeforeFirstStep,
    /// Screen-unchanged detector, after the first Step's `waitAfterMove`.
    AfterFirstStep,
    /// Check overlap, before the Step.
    First,
    /// Check overlap, after the Step.
    Second,
}

/// One operation of a plan.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Move(Step),
    Wait(Duration),
    /// A Shot of the Session grid, 1-based: Row 1 is the start pole, column 1
    /// the start heading.
    Shot {
        row: u32,
        col: u32,
    },
    Sample(SampleSlot),
}

impl Op {
    /// What this operation adds to the estimate: the Step time, the wait, or
    /// [`CAPTURE_COST`] for a Shot or a Sample.
    pub fn cost(&self) -> Duration {
        match self {
            Self::Move(step) => step.duration(),
            Self::Wait(wait) => *wait,
            Self::Shot { .. } | Self::Sample(_) => CAPTURE_COST,
        }
    }
}

/// The Session grid of the Game a plan was made from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    pub shots_per_row: u32,
    pub rows: u32,
}

impl Grid {
    pub fn of(game: &Game) -> Self {
        Self {
            shots_per_row: game.shots_per_row,
            rows: game.rows,
        }
    }
}

/// The operations of one run, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturePlan {
    pub kind: RunKind,
    pub grid: Grid,
    /// The Game's `waitAfterShot`, which the Runner also waits before
    /// retrying a failed Shot (§7.3).
    pub wait_after_shot: Duration,
    pub ops: Vec<Op>,
}

impl CapturePlan {
    /// How many Shots the run takes: N × R for a Session, 0 otherwise.
    pub fn expected_shots(&self) -> usize {
        self.ops
            .iter()
            .filter(|op| matches!(op, Op::Shot { .. }))
            .count()
    }

    /// The sum of every operation's [`Op::cost`], without the countdown.
    pub fn duration(&self) -> Duration {
        self.ops.iter().map(Op::cost).sum()
    }
}

/// The plan of `kind` for `game`.
pub fn plan(game: &Game, kind: RunKind) -> CapturePlan {
    match kind {
        RunKind::Session => plan_session(game),
        RunKind::TurnTest => plan_turn_test(game),
        RunKind::PitchTest => plan_pitch_test(game),
        RunKind::StepOnce(axis) => plan_step_once(game, axis),
        RunKind::CheckOverlap(axis) => plan_check_overlap(game, axis),
    }
}

fn plan_of(game: &Game, kind: RunKind, ops: Vec<Op>) -> CapturePlan {
    CapturePlan {
        kind,
        grid: Grid::of(game),
        wait_after_shot: millis(game.wait_after_shot_ms),
        ops,
    }
}

/// Spec §7.1: first Shot at once, then per Row N times `Shot → waitAfterShot
/// → turn → waitAfterMove` (the Nth turn closes the Row), and after every
/// Row but the last `pitch → waitAfterMove`.
pub fn plan_session(game: &Game) -> CapturePlan {
    let turn = Step::for_game(game, Axis::Turn);
    let pitch = Step::for_game(game, Axis::Pitch);
    let after_shot = millis(game.wait_after_shot_ms);
    let after_move = millis(game.wait_after_move_ms);
    let (n, r) = (game.shots_per_row, game.rows);

    let mut ops = Vec::with_capacity((n * r * 4 + r.saturating_sub(1) * 2) as usize);
    for row in 1..=r {
        for col in 1..=n {
            ops.push(Op::Shot { row, col });
            ops.push(Op::Wait(after_shot));
            ops.push(Op::Move(turn.clone()));
            ops.push(Op::Wait(after_move));
        }
        if row < r {
            ops.push(Op::Move(pitch.clone()));
            ops.push(Op::Wait(after_move));
        }
    }
    plan_of(game, RunKind::Session, ops)
}

/// N turn Steps with the detector's two Samples around the first.
pub fn plan_turn_test(game: &Game) -> CapturePlan {
    plan_of(
        game,
        RunKind::TurnTest,
        probed_steps(game, Axis::Turn, game.shots_per_row),
    )
}

/// R − 1 pitch Steps with the detector's two Samples around the first.
pub fn plan_pitch_test(game: &Game) -> CapturePlan {
    plan_of(
        game,
        RunKind::PitchTest,
        probed_steps(game, Axis::Pitch, game.rows.saturating_sub(1)),
    )
}

/// One Step with the detector's two Samples around it.
pub fn plan_step_once(game: &Game, axis: Axis) -> CapturePlan {
    plan_of(game, RunKind::StepOnce(axis), probed_steps(game, axis, 1))
}

/// `Sample(First) → Step → waitAfterMove → Sample(Second)`.
pub fn plan_check_overlap(game: &Game, axis: Axis) -> CapturePlan {
    plan_of(
        game,
        RunKind::CheckOverlap(axis),
        vec![
            Op::Sample(SampleSlot::First),
            Op::Move(Step::for_game(game, axis)),
            Op::Wait(millis(game.wait_after_move_ms)),
            Op::Sample(SampleSlot::Second),
        ],
    )
}

/// `count` Steps on `axis`, each followed by `waitAfterMove` and never by
/// `waitAfterShot` (§7.4), with the detector's Samples around the first.
fn probed_steps(game: &Game, axis: Axis, count: u32) -> Vec<Op> {
    let step = Step::for_game(game, axis);
    let after_move = millis(game.wait_after_move_ms);
    let mut ops = Vec::with_capacity((count * 2 + 2) as usize);
    for i in 0..count {
        if i == 0 {
            ops.push(Op::Sample(SampleSlot::BeforeFirstStep));
        }
        ops.push(Op::Move(step.clone()));
        ops.push(Op::Wait(after_move));
        if i == 0 {
            ops.push(Op::Sample(SampleSlot::AfterFirstStep));
        }
    }
    ops
}

fn millis(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

impl fmt::Display for RunKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Session => f.write_str("session"),
            Self::TurnTest => f.write_str("turn test"),
            Self::PitchTest => f.write_str("pitch test"),
            Self::StepOnce(axis) => write!(f, "step once ({axis})"),
            Self::CheckOverlap(axis) => write!(f, "check overlap ({axis})"),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Move(step) => write!(f, "move {step}"),
            Self::Wait(wait) => write!(f, "wait {wait:?}"),
            Self::Shot { row, col } => write!(f, "shot r{row:02} c{col:02}"),
            Self::Sample(slot) => write!(f, "sample {slot:?}"),
        }
    }
}

/// One line per operation, after a header with the kind, Shot count and
/// duration without countdown.
impl fmt::Display for CapturePlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "# {}: {} ops, {} shots, {:?}",
            self.kind,
            self.ops.len(),
            self.expected_shots(),
            self.duration()
        )?;
        for op in &self.ops {
            writeln!(f, "{op}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::NewGameDefaults;

    fn small_game() -> Game {
        let mut game = Game::new("x", NewGameDefaults::default());
        game.shots_per_row = 3;
        game.rows = 2;
        game.wait_after_move_ms = 10;
        game.wait_after_shot_ms = 20;
        game
    }

    fn kinds(plan: &CapturePlan) -> String {
        plan.ops
            .iter()
            .map(|op| match op {
                Op::Move(step) => match step.axis {
                    Axis::Turn => "T",
                    Axis::Pitch => "P",
                },
                Op::Wait(w) if *w == Duration::from_millis(20) => "s",
                Op::Wait(_) => "m",
                Op::Shot { .. } => "S",
                Op::Sample(_) => "x",
            })
            .collect()
    }

    #[test]
    fn session_follows_the_spec_sequence() {
        let plan = plan_session(&small_game());
        assert_eq!(kinds(&plan), "SsTmSsTmSsTmPmSsTmSsTmSsTm");
        assert_eq!(plan.expected_shots(), 6);
        let shots: Vec<(u32, u32)> = plan
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Shot { row, col } => Some((*row, *col)),
                _ => None,
            })
            .collect();
        assert_eq!(shots, [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3)]);
    }

    #[test]
    fn tests_probe_the_first_step_and_never_wait_after_shot() {
        let game = small_game();
        assert_eq!(kinds(&plan_turn_test(&game)), "xTmxTmTm");
        assert_eq!(kinds(&plan_pitch_test(&game)), "xPmx");
        assert_eq!(kinds(&plan_step_once(&game, Axis::Turn)), "xTmx");
        assert_eq!(kinds(&plan_step_once(&game, Axis::Pitch)), "xPmx");
        assert_eq!(kinds(&plan_check_overlap(&game, Axis::Turn)), "xTmx");
        let overlap = plan_check_overlap(&game, Axis::Pitch);
        assert_eq!(overlap.ops[0], Op::Sample(SampleSlot::First));
        assert_eq!(overlap.ops[3], Op::Sample(SampleSlot::Second));
        assert_eq!(overlap.expected_shots(), 0);
    }

    #[test]
    fn plan_dispatches_on_kind() {
        let game = small_game();
        for kind in [
            RunKind::Session,
            RunKind::TurnTest,
            RunKind::PitchTest,
            RunKind::StepOnce(Axis::Pitch),
            RunKind::CheckOverlap(Axis::Turn),
        ] {
            assert_eq!(plan(&game, kind).kind, kind);
        }
    }

    #[test]
    fn op_costs_match_appendix_a() {
        assert_eq!(Op::Shot { row: 1, col: 1 }.cost(), CAPTURE_COST);
        assert_eq!(Op::Sample(SampleSlot::First).cost(), CAPTURE_COST);
        assert_eq!(
            Op::Wait(Duration::from_millis(7)).cost(),
            Duration::from_millis(7)
        );
    }
}
