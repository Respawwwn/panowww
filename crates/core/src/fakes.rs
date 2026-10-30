//! Fakes for the ports, behind the `test-support` feature: a whole Session
//! runs through them in tests, without a game, a screen or a disk.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use crate::game::PhysicalKey;
use crate::ports::{
    Camera, CancelFlag, CaptureError, Clock, Frame, InputError, MouseButton, Point, ShotSink,
    StoreError, Stream,
};
use crate::runner::{RunOutcome, SessionStart, ShotRecord};

/// One primitive a [`RecordingCamera`] received.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraCall {
    KeyDown(PhysicalKey),
    KeyUp(PhysicalKey),
    MouseDelta(i32, i32),
    ButtonDown(MouseButton),
    ButtonUp(MouseButton),
    ParkCursor(Point),
    Stick(f64, f64),
    ReleaseAll,
}

impl fmt::Display for CameraCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyDown(key) => write!(f, "key_down {}", key.as_str()),
            Self::KeyUp(key) => write!(f, "key_up {}", key.as_str()),
            Self::MouseDelta(dx, dy) => write!(f, "mouse_delta {dx} {dy}"),
            Self::ButtonDown(button) => write!(f, "button_down {button:?}"),
            Self::ButtonUp(button) => write!(f, "button_up {button:?}"),
            Self::ParkCursor(at) => write!(f, "park_cursor {} {}", at.x, at.y),
            Self::Stick(x, y) => write!(f, "stick {x} {y}"),
            Self::ReleaseAll => f.write_str("release_all"),
        }
    }
}

/// A `Camera` that records every primitive, and can fail on demand.
#[derive(Debug, Default)]
pub struct RecordingCamera {
    pub calls: Vec<CameraCall>,
    /// Fail the call with this 1-based number.
    pub fail_on_call: Option<usize>,
}

impl RecordingCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// The calls, one per line, with runs of identical calls collapsed as
    /// `mouse_delta 20 0 ×42`.
    pub fn transcript(&self) -> String {
        let mut lines = String::new();
        let mut calls = self.calls.iter().peekable();
        while let Some(call) = calls.next() {
            let mut repeats = 1;
            while calls.next_if(|next| *next == call).is_some() {
                repeats += 1;
            }
            lines.push_str(&call.to_string());
            if repeats > 1 {
                write!(lines, " ×{repeats}").expect("writing to a String never fails");
            }
            lines.push('\n');
        }
        lines
    }

    fn record(&mut self, call: CameraCall) -> Result<(), InputError> {
        self.calls.push(call);
        if self.fail_on_call == Some(self.calls.len()) {
            return Err(InputError::Failed("fake input failure".to_owned()));
        }
        Ok(())
    }
}

impl Camera for RecordingCamera {
    fn key_down(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        self.record(CameraCall::KeyDown(key))
    }

    fn key_up(&mut self, key: PhysicalKey) -> Result<(), InputError> {
        self.record(CameraCall::KeyUp(key))
    }

    fn mouse_delta(&mut self, dx: i32, dy: i32) -> Result<(), InputError> {
        self.record(CameraCall::MouseDelta(dx, dy))
    }

    fn button_down(&mut self, button: MouseButton) -> Result<(), InputError> {
        self.record(CameraCall::ButtonDown(button))
    }

    fn button_up(&mut self, button: MouseButton) -> Result<(), InputError> {
        self.record(CameraCall::ButtonUp(button))
    }

    fn park_cursor(&mut self, at: Point) -> Result<(), InputError> {
        self.record(CameraCall::ParkCursor(at))
    }

    fn stick(&mut self, x: f64, y: f64) -> Result<(), InputError> {
        self.record(CameraCall::Stick(x, y))
    }

    fn release_all(&mut self) -> Result<(), InputError> {
        self.record(CameraCall::ReleaseAll)
    }
}

/// A `Stream` serving numbered frames: the n-th call fills its frame with
/// the byte `n`, so a test can tell which capture produced a Sample. Those
/// frames differ by one level, which the screen-unchanged detector reads as
/// a still screen; [`FakeStream::moving`] makes them differ widely instead.
#[derive(Debug)]
pub struct FakeStream {
    pub width: u32,
    pub height: u32,
    /// How many `shot` calls were made, failed ones included.
    pub calls: u32,
    /// 1-based `shot` calls that fail with `CaptureError::ShotFailed`.
    pub fail_on_calls: Vec<u32>,
    /// The `not_before` of every call.
    pub freshness_marks: Vec<Instant>,
    pub preview_on: bool,
    /// Fill frames with `64 × n` instead of `n`, so consecutive frames
    /// look like a moving screen.
    pub moving: bool,
}

impl FakeStream {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            calls: 0,
            fail_on_calls: Vec::new(),
            freshness_marks: Vec::new(),
            preview_on: false,
            moving: false,
        }
    }

    /// The same stream, failing the given 1-based calls.
    #[must_use]
    pub fn failing_on(mut self, calls: impl IntoIterator<Item = u32>) -> Self {
        self.fail_on_calls = calls.into_iter().collect();
        self
    }

    /// The same stream, with frames that change a lot from call to call.
    /// [`FakeStream::frame_number`] no longer applies.
    #[must_use]
    pub fn moving(mut self) -> Self {
        self.moving = true;
        self
    }

    /// Which call produced `frame`, from its fill byte.
    pub fn frame_number(frame: &Frame) -> u8 {
        frame.rgb.first().copied().unwrap_or(0)
    }
}

impl Stream for FakeStream {
    fn shot(&mut self, not_before: Instant) -> Result<Frame, CaptureError> {
        self.calls += 1;
        self.freshness_marks.push(not_before);
        if self.fail_on_calls.contains(&self.calls) {
            return Err(CaptureError::ShotFailed(format!(
                "fake failure on call {}",
                self.calls
            )));
        }
        let step = if self.moving { 64 } else { 1 };
        let fill = u8::try_from(self.calls * step % 256).unwrap_or(0);
        Ok(Frame::solid(self.width, self.height, [fill; 3]))
    }

    fn set_preview(&mut self, on: bool) {
        self.preview_on = on;
    }

    fn stop(self: Box<Self>) {}
}

/// A `ShotSink` keeping everything in memory.
#[derive(Debug, Default)]
pub struct MemorySink {
    pub begun: Option<SessionStart>,
    pub shots: Vec<(ShotRecord, Frame)>,
    pub finished: Option<RunOutcome>,
    /// Fail the Shot with this 1-based index.
    pub fail_on_shot: Option<u32>,
}

impl MemorySink {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShotSink for MemorySink {
    fn begin(&mut self, start: &SessionStart) -> Result<(), StoreError> {
        self.begun = Some(start.clone());
        Ok(())
    }

    fn shot(&mut self, record: ShotRecord, frame: Frame) -> Result<(), StoreError> {
        if self.fail_on_shot == Some(record.index) {
            return Err(StoreError::Failed(format!(
                "fake store failure on Shot {}",
                record.index
            )));
        }
        self.shots.push((record, frame));
        Ok(())
    }

    fn finish(&mut self, outcome: &RunOutcome) -> Result<(), StoreError> {
        self.finished = Some(outcome.clone());
        Ok(())
    }
}

/// A `Clock` whose waits pass instantly, advancing a virtual `now`, and
/// which can raise the cancel flag during a chosen wait.
#[derive(Debug)]
pub struct FakeClock {
    now: Cell<Instant>,
    waits: RefCell<Vec<Duration>>,
    /// Raise the cancel flag during the wait with this 1-based number.
    pub cancel_on_wait: Option<usize>,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeClock {
    pub fn new() -> Self {
        Self {
            now: Cell::new(Instant::now()),
            waits: RefCell::new(Vec::new()),
            cancel_on_wait: None,
        }
    }

    /// The same clock, raising the cancel flag during the `n`-th wait.
    #[must_use]
    pub fn cancelling_on_wait(mut self, n: usize) -> Self {
        self.cancel_on_wait = Some(n);
        self
    }

    /// Every wait requested, completed or not.
    pub fn waits(&self) -> Vec<Duration> {
        self.waits.borrow().clone()
    }

    /// The sum of every wait requested.
    pub fn total_waited(&self) -> Duration {
        self.waits.borrow().iter().sum()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn wait(&self, duration: Duration, cancel: &CancelFlag) -> bool {
        let count = {
            let mut waits = self.waits.borrow_mut();
            waits.push(duration);
            waits.len()
        };
        if self.cancel_on_wait == Some(count) {
            cancel.raise();
        }
        if cancel.is_raised() {
            return false;
        }
        self.now.set(self.now.get() + duration);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_collapses_repeats() {
        let mut camera = RecordingCamera::new();
        camera.mouse_delta(20, 0).unwrap();
        camera.mouse_delta(20, 0).unwrap();
        camera.mouse_delta(10, 0).unwrap();
        camera.release_all().unwrap();
        assert_eq!(
            camera.transcript(),
            "mouse_delta 20 0 ×2\nmouse_delta 10 0\nrelease_all\n"
        );
    }

    #[test]
    fn fake_stream_numbers_frames_and_fails_on_schedule() {
        let mut stream = FakeStream::new(4, 2).failing_on([2]);
        let now = Instant::now();
        assert_eq!(FakeStream::frame_number(&stream.shot(now).unwrap()), 1);
        assert!(matches!(stream.shot(now), Err(CaptureError::ShotFailed(_))));
        let third = stream.shot(now).unwrap();
        assert_eq!(FakeStream::frame_number(&third), 3);
        assert!(third.is_well_formed());
        assert_eq!(stream.calls, 3);
    }

    #[test]
    fn fake_clock_advances_instantly_and_cancels_on_the_chosen_wait() {
        let clock = FakeClock::new().cancelling_on_wait(2);
        let cancel = CancelFlag::new();
        let start = clock.now();
        assert!(clock.wait(Duration::from_secs(3), &cancel));
        assert_eq!(clock.now() - start, Duration::from_secs(3));
        assert!(!clock.wait(Duration::from_secs(1), &cancel));
        assert!(cancel.is_raised());
        assert_eq!(
            clock.now() - start,
            Duration::from_secs(3),
            "a cancelled wait does not pass"
        );
        assert_eq!(clock.waits().len(), 2);
        assert_eq!(clock.total_waited(), Duration::from_secs(4));
    }
}
