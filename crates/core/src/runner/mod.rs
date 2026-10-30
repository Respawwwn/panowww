//! The Runner: performs a capture plan through the ports (spec §4.4, §7.2,
//! §7.3). Blocking and meant for a dedicated OS thread; the cancel flag is
//! checked before every primitive and inside every wait. Movement logic
//! lives here, never in an adapter (ADR 0001).

pub mod thumbnail;

use std::time::Instant;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::constants::{DRAG_SETTLE, MOUSE_SUB_MOVE_INTERVAL};
use crate::game::Game;
use crate::plan::{Axis, CapturePlan, Motion, MouseDelta, Op, SampleSlot, Step};
use crate::ports::{
    Camera, CancelFlag, CaptureError, Clock, Frame, InputError, Jpeg, PreviewFrame, Rect, ShotSink,
    StoreError, Stream, Zone,
};

/// Everything a run needs besides its ports.
#[derive(Debug, Clone, PartialEq)]
pub struct RunRequest<'a> {
    pub plan: &'a CapturePlan,
    /// The Capture zone, where a drag Step parks the cursor.
    pub zone: Rect,
    /// For a Session: what the store needs to create its folder. `None` for
    /// Tests, Step once and Check overlap, which write nothing.
    pub session: Option<SessionStart>,
}

/// What a Session folder records about its start (spec §6). The store adds
/// the id, the app version and the timestamps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionStart {
    /// The Game as it was when the run started.
    pub game: Game,
    pub zone: Zone,
    pub shots_expected: u32,
}

/// One Shot of a Session, 1-based like the file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ShotRecord {
    pub index: u32,
    pub row: u32,
    pub col: u32,
    pub capture_ms: u32,
}

/// The `error.kind` set of `session.json`, plus the two sources the spec's
/// list implies: the input device and the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunErrorKind {
    CaptureUnavailable,
    DisplayGone,
    PermissionRevoked,
    ShotFailed,
    Input,
    Store,
}

/// Why a run failed, in plain English.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RunError {
    pub kind: RunErrorKind,
    pub message: String,
}

impl From<CaptureError> for RunError {
    fn from(error: CaptureError) -> Self {
        let kind = match &error {
            CaptureError::Unavailable(_) => RunErrorKind::CaptureUnavailable,
            CaptureError::DisplayGone => RunErrorKind::DisplayGone,
            CaptureError::PermissionRevoked => RunErrorKind::PermissionRevoked,
            CaptureError::ShotFailed(_) => RunErrorKind::ShotFailed,
        };
        Self {
            kind,
            message: error.to_string(),
        }
    }
}

impl From<InputError> for RunError {
    fn from(error: InputError) -> Self {
        Self {
            kind: RunErrorKind::Input,
            message: error.to_string(),
        }
    }
}

impl From<StoreError> for RunError {
    fn from(error: StoreError) -> Self {
        Self {
            kind: RunErrorKind::Store,
            message: error.to_string(),
        }
    }
}

/// How a run ended. Every variant keeps the Shots taken so far (§7.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "camelCase")]
#[ts(export)]
pub enum RunOutcome {
    Completed {
        shots: u32,
    },
    /// Stopped by the user or the app.
    Interrupted {
        shots: u32,
    },
    Failed {
        shots: u32,
        error: RunError,
    },
}

impl RunOutcome {
    pub fn shots(&self) -> u32 {
        match self {
            Self::Completed { shots }
            | Self::Interrupted { shots }
            | Self::Failed { shots, .. } => *shots,
        }
    }
}

/// Events of a run, in order, on the progress channel. Preview frames from
/// the capture adapter travel on the same channel.
#[derive(Debug)]
pub enum Progress {
    Started {
        expected: u32,
    },
    /// A Shot was taken and handed to the store.
    Shot {
        index: u32,
        row: u32,
        col: u32,
        capture_ms: u32,
        /// `None` only when the frame could not be encoded.
        thumbnail: Option<Jpeg>,
    },
    /// A Step was performed; the camera now sits at `row`, `col`.
    Moved {
        row: u32,
        col: u32,
    },
    /// A Sample was taken; Check overlap keeps `First` and `Second`.
    Sample {
        slot: SampleSlot,
        frame: Frame,
    },
    /// The screen-unchanged detector found nothing moved (spec §4.6).
    ScreenUnchanged,
    Preview(PreviewFrame),
    Finished(RunOutcome),
}

/// Why `Runner::execute` stopped early.
enum Stop {
    Cancelled,
    Failed(RunError),
}

impl<E: Into<RunError>> From<E> for Stop {
    fn from(error: E) -> Self {
        Self::Failed(error.into())
    }
}

/// Performs `request.plan` through the ports and reports how it ended. On a
/// stop or a failure the camera's `release_all` runs before anything else
/// (§7.2); a Session's sink is begun at the first Shot and finished with
/// the outcome.
pub fn run(
    request: &RunRequest<'_>,
    camera: &mut dyn Camera,
    stream: &mut dyn Stream,
    sink: &mut dyn ShotSink,
    clock: &dyn Clock,
    cancel: &CancelFlag,
    progress: &dyn Fn(Progress),
) -> RunOutcome {
    let mut runner = Runner {
        request,
        camera,
        stream,
        sink,
        clock,
        cancel,
        progress,
        shots: 0,
        row: 1,
        col: 1,
        not_before: clock.now(),
        session_begun: false,
    };
    progress(Progress::Started {
        expected: expected_shots(request.plan),
    });
    let result = runner.execute();
    let outcome = runner.finish(result);
    progress(Progress::Finished(outcome.clone()));
    outcome
}

fn expected_shots(plan: &CapturePlan) -> u32 {
    u32::try_from(plan.expected_shots()).unwrap_or(u32::MAX)
}

struct Runner<'r> {
    request: &'r RunRequest<'r>,
    camera: &'r mut dyn Camera,
    stream: &'r mut dyn Stream,
    sink: &'r mut dyn ShotSink,
    clock: &'r dyn Clock,
    cancel: &'r CancelFlag,
    progress: &'r dyn Fn(Progress),
    shots: u32,
    row: u32,
    col: u32,
    /// When the last Step ended: a Shot's frame must be newer than this.
    not_before: Instant,
    session_begun: bool,
}

impl Runner<'_> {
    fn execute(&mut self) -> Result<(), Stop> {
        for op in &self.request.plan.ops {
            match op {
                Op::Move(step) => {
                    self.perform(step)?;
                    self.not_before = self.clock.now();
                    self.advance(step.axis);
                    (self.progress)(Progress::Moved {
                        row: self.row,
                        col: self.col,
                    });
                }
                Op::Wait(wait) => self.wait(*wait)?,
                Op::Shot { row, col } => self.shot(*row, *col)?,
                Op::Sample(slot) => {
                    let (frame, _) = self.capture()?;
                    (self.progress)(Progress::Sample { slot: *slot, frame });
                }
            }
        }
        Ok(())
    }

    /// Releases the input device, closes the Session if one was begun and
    /// turns the stop reason into the outcome.
    fn finish(&mut self, result: Result<(), Stop>) -> RunOutcome {
        let released = self.camera.release_all();
        let shots = self.shots;
        let mut outcome = match result {
            Ok(()) => match released {
                Ok(()) => RunOutcome::Completed { shots },
                Err(error) => RunOutcome::Failed {
                    shots,
                    error: error.into(),
                },
            },
            Err(Stop::Cancelled) => RunOutcome::Interrupted { shots },
            Err(Stop::Failed(error)) => RunOutcome::Failed { shots, error },
        };
        if self.session_begun
            && let Err(error) = self.sink.finish(&outcome)
            && !matches!(outcome, RunOutcome::Failed { .. })
        {
            outcome = RunOutcome::Failed {
                shots,
                error: error.into(),
            };
        }
        outcome
    }

    /// Expands a Step into primitives and waits (spec §7.1).
    fn perform(&mut self, step: &Step) -> Result<(), Stop> {
        match &step.motion {
            Motion::KeyHold { key, hold } => {
                let key = key.clone();
                self.primitive(|c| c.key_down(key.clone()))?;
                self.wait(*hold)?;
                self.primitive(|c| c.key_up(key))
            }
            Motion::MouseMove(delta) => self.sub_moves(*delta),
            Motion::MouseDrag { button, delta } => {
                let (button, at) = (*button, self.request.zone.centre());
                self.primitive(|c| c.park_cursor(at))?;
                self.primitive(|c| c.button_down(button))?;
                self.wait(DRAG_SETTLE)?;
                self.sub_moves(*delta)?;
                self.wait(DRAG_SETTLE)?;
                self.primitive(|c| c.button_up(button))
            }
            Motion::StickHold { x, y, hold } => {
                let (x, y) = (*x, *y);
                self.primitive(|c| c.stick(x, y))?;
                self.wait(*hold)?;
                self.primitive(|c| c.stick(0.0, 0.0))
            }
        }
    }

    fn sub_moves(&mut self, delta: MouseDelta) -> Result<(), Stop> {
        for sub in delta.sub_moves() {
            self.primitive(|c| c.mouse_delta(sub.dx, sub.dy))?;
            self.wait(MOUSE_SUB_MOVE_INTERVAL)?;
        }
        Ok(())
    }

    /// One camera call, preceded by the cancel check.
    fn primitive(
        &mut self,
        call: impl FnOnce(&mut dyn Camera) -> Result<(), InputError>,
    ) -> Result<(), Stop> {
        if self.cancel.is_raised() {
            return Err(Stop::Cancelled);
        }
        call(self.camera)?;
        Ok(())
    }

    fn wait(&mut self, duration: std::time::Duration) -> Result<(), Stop> {
        if self.clock.wait(duration, self.cancel) {
            Ok(())
        } else {
            Err(Stop::Cancelled)
        }
    }

    /// Takes the Shot at `row`, `col`: begins the Session on the first one,
    /// hands the frame to the store and reports it with its thumbnail.
    fn shot(&mut self, row: u32, col: u32) -> Result<(), Stop> {
        if !self.session_begun
            && let Some(start) = &self.request.session
        {
            self.sink.begin(start)?;
            self.session_begun = true;
        }
        let (frame, capture_ms) = self.capture()?;
        let index = self.shots + 1;
        let thumbnail = thumbnail::thumbnail(&frame);
        if self.session_begun {
            let record = ShotRecord {
                index,
                row,
                col,
                capture_ms,
            };
            self.sink.shot(record, frame)?;
        }
        // Counted once the store has it, so the outcome never claims a Shot
        // that was not kept.
        self.shots = index;
        (self.progress)(Progress::Shot {
            index,
            row,
            col,
            capture_ms,
            thumbnail,
        });
        Ok(())
    }

    /// A fresh frame and how long it took. A failure is retried once after a
    /// fresh `waitAfterShot`; a second failure fails the run (§7.3).
    fn capture(&mut self) -> Result<(Frame, u32), Stop> {
        if self.cancel.is_raised() {
            return Err(Stop::Cancelled);
        }
        match self.timed_shot() {
            Ok(taken) => Ok(taken),
            Err(_first_failure) => {
                self.wait(self.request.plan.wait_after_shot)?;
                Ok(self.timed_shot()?)
            }
        }
    }

    fn timed_shot(&mut self) -> Result<(Frame, u32), CaptureError> {
        let started = self.clock.now();
        let frame = self.stream.shot(self.not_before)?;
        let elapsed = self.clock.now().saturating_duration_since(started);
        let capture_ms = u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX);
        Ok((frame, capture_ms))
    }

    /// Where the camera sits after a Step: a turn advances the column and
    /// the closing turn wraps it; a pitch starts the next Row.
    fn advance(&mut self, axis: Axis) {
        match axis {
            Axis::Turn => {
                self.col = if self.col >= self.request.plan.grid.shots_per_row {
                    1
                } else {
                    self.col + 1
                };
            }
            Axis::Pitch => self.row += 1,
        }
    }
}
