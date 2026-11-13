//! The single Runner owner (spec §4.4, §4.5, §7.2): `Idle → Countdown →
//! Running → Stopping → Idle` on a dedicated thread. Shortcuts, UI buttons
//! and window close all go through it. Start while not Idle is ignored.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use base64::Engine as _;
use panowww_core::game::Game;
use panowww_core::plan::{SampleSlot, plan};
use panowww_core::ports::{
    CancelFlag, Frame, Jpeg, PreviewFrame, Rect, ShotSink, SystemClock, Zone,
};
use panowww_core::runner::{Progress, RunOutcome, RunRequest, SessionStart, run};
use panowww_store::local::{ZoneStatus, zone_status};
use panowww_store::sink::{SessionSink, SessionSinkConfig, SystemWallClock};

use crate::backend::SharedBackend;
use crate::dto::{RunEvent, RunKindDto, RunnerPhase, SampleSlotDto, StartFailure};
use crate::overlay::SharedOverlay;

/// Countdown after the Start shortcut (spec Appendix A).
pub const COUNTDOWN_SHORTCUT: Duration = Duration::from_secs(3);
/// Countdown after a button.
pub const COUNTDOWN_BUTTON: Duration = Duration::from_secs(5);

/// Everything one run needs, gathered by the command layer.
#[derive(Debug, Clone)]
pub struct RunSpec {
    pub game: Game,
    pub kind: RunKindDto,
    pub zone: Zone,
    pub countdown: Duration,
    /// For a Session: where the folder goes and the app version to record.
    pub session: SessionSinkConfig,
    pub preview: bool,
}

/// Where the run's events go: the window, or a test's channel.
pub type EventSink = Arc<dyn Fn(RunEvent) + Send + Sync>;

/// Flips the Display preview of the run in progress; `None` while Idle.
type PreviewSwitch = Arc<Mutex<Option<Box<dyn Fn(bool) + Send>>>>;

pub struct Runner {
    backend: SharedBackend,
    emit: EventSink,
    overlay: SharedOverlay,
    phase: Arc<AtomicU8>,
    cancel: Mutex<Option<CancelFlag>>,
    preview_switch: PreviewSwitch,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner")
            .field("phase", &self.phase())
            .finish_non_exhaustive()
    }
}

const IDLE: u8 = 0;
const COUNTDOWN: u8 = 1;
const RUNNING: u8 = 2;
const STOPPING: u8 = 3;

impl Runner {
    pub fn new(backend: SharedBackend, emit: EventSink, overlay: SharedOverlay) -> Self {
        Self {
            backend,
            emit,
            overlay,
            phase: Arc::new(AtomicU8::new(IDLE)),
            cancel: Mutex::new(None),
            preview_switch: Arc::new(Mutex::new(None)),
            thread: Mutex::new(None),
        }
    }

    pub fn phase(&self) -> RunnerPhase {
        match self.phase.load(Ordering::SeqCst) {
            COUNTDOWN => RunnerPhase::Countdown,
            RUNNING => RunnerPhase::Running,
            STOPPING => RunnerPhase::Stopping,
            _ => RunnerPhase::Idle,
        }
    }

    /// Starts the countdown of `spec` on the Runner thread. Returns `false`
    /// when a run is already going on: the request is ignored (spec §4.4).
    pub fn start(&self, spec: RunSpec) -> bool {
        if self
            .phase
            .compare_exchange(IDLE, COUNTDOWN, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            tracing::info!("start ignored: a run is in progress");
            return false;
        }
        self.join_finished();
        let cancel = CancelFlag::new();
        *self
            .cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cancel.clone());
        let worker = Worker {
            backend: Arc::clone(&self.backend),
            emit: Arc::clone(&self.emit),
            overlay: Arc::clone(&self.overlay),
            phase: Arc::clone(&self.phase),
            preview_switch: Arc::clone(&self.preview_switch),
            cancel,
        };
        let handle = std::thread::Builder::new()
            .name("panowww-runner".to_owned())
            .spawn(move || worker.execute(&spec))
            .expect("spawning the Runner thread");
        *self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(handle);
        true
    }

    /// Cancels the countdown or stops the run; `release_all` happens first
    /// on the Runner thread (spec §7.2). Ignored while Idle.
    pub fn stop(&self) {
        if self.phase.load(Ordering::SeqCst) == IDLE {
            return;
        }
        let _ = self
            .phase
            .compare_exchange(RUNNING, STOPPING, Ordering::SeqCst, Ordering::SeqCst);
        if let Some(cancel) = self
            .cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            cancel.raise();
        }
    }

    /// Turns the Display preview source on or off during a countdown or a
    /// run; a no-op otherwise.
    pub fn set_preview(&self, on: bool) {
        if let Some(switch) = self
            .preview_switch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            switch(on);
        }
    }

    /// Stops and waits for the thread: the quit path, which marks a
    /// running Session `interrupted` through the sink.
    pub fn shutdown(&self) {
        self.stop();
        if let Some(handle) = self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _ = handle.join();
        }
    }

    /// Waits for the current run to end, for tests.
    #[cfg(test)]
    pub fn wait(&self) {
        if let Some(handle) = self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _ = handle.join();
        }
    }

    fn join_finished(&self) {
        if let Some(handle) = self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            && handle.is_finished()
        {
            let _ = handle.join();
        }
    }
}

/// The run itself, on its own thread.
struct Worker {
    backend: SharedBackend,
    emit: EventSink,
    overlay: SharedOverlay,
    phase: Arc<AtomicU8>,
    preview_switch: PreviewSwitch,
    cancel: CancelFlag,
}

impl Worker {
    fn execute(self, spec: &RunSpec) {
        let closing = self.countdown_then_run(spec);
        *self
            .preview_switch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        self.phase.store(IDLE, Ordering::SeqCst);
        (self.emit)(closing);
    }

    /// Pre-flight, stream, countdown, run. Returns the closing event. The
    /// overlay follows (spec §10): the digit, the frame with the progress,
    /// the end pill of a Session; hidden on failure or cancel.
    fn countdown_then_run(&self, spec: &RunSpec) -> RunEvent {
        if let Err(failure) = self.preflight(spec) {
            self.overlay.hide();
            return RunEvent::CannotStart(failure);
        }
        let mut stream = match self.open_stream(spec) {
            Ok(stream) => stream,
            Err(failure) => {
                self.overlay.hide();
                return RunEvent::CannotStart(failure);
            }
        };
        // The stream stays on this thread; the command layer flips a shared
        // flag that is applied between ticks and after the run.
        let preview_flag = Arc::new(Mutex::new(spec.preview));
        {
            let flag = Arc::clone(&preview_flag);
            *self
                .preview_switch
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Box::new(move |on| {
                *flag
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = on;
            }));
        }
        stream.set_preview(spec.preview);

        if !self.countdown(spec, &preview_flag, stream.as_mut()) {
            stream.stop();
            self.overlay.hide();
            return RunEvent::Cancelled;
        }

        self.phase.store(RUNNING, Ordering::SeqCst);
        self.overlay.show_frame(&spec.zone);
        let plan = plan(&spec.game, spec.kind.into());
        let expected = u32::try_from(plan.expected_shots()).unwrap_or(u32::MAX);
        let origin = self.backend.display_origin(&spec.zone.display_id);
        let request = RunRequest {
            plan: &plan,
            zone: global_rect(&spec.zone, origin),
            session: spec.kind.is_session().then(|| SessionStart {
                game: spec.game.clone(),
                zone: spec.zone.clone(),
                shots_expected: expected,
            }),
        };
        let mut camera = self.backend.camera();
        let mut session_sink = SessionSink::new(spec.session.clone(), Box::new(SystemWallClock));
        let mut no_sink = NoSink;
        let sink: &mut dyn ShotSink = if spec.kind.is_session() {
            &mut session_sink
        } else {
            &mut no_sink
        };
        let emit = Arc::clone(&self.emit);
        let overlay = Arc::clone(&self.overlay);
        let outcome = run(
            &request,
            camera.as_mut(),
            stream.as_mut(),
            sink,
            &SystemClock,
            &self.cancel,
            &|progress| {
                match &progress {
                    Progress::Started { .. } => overlay.progress(0, expected),
                    Progress::Shot { index, .. } => overlay.progress(*index, expected),
                    _ => {}
                }
                if let Some(event) = translate(progress) {
                    emit(event);
                }
            },
        );
        stream.stop();
        if spec.kind.is_session() {
            let stopped = !matches!(outcome, RunOutcome::Completed { .. });
            self.overlay.end_pill(stopped, outcome.shots(), expected);
        } else {
            self.overlay.hide();
        }
        let folder = session_sink.folder().map(|p| p.display().to_string());
        RunEvent::Finished { outcome, folder }
    }

    /// Spec §4.5 step 2: the capture stream, with the preview sink feeding
    /// the window.
    fn open_stream(
        &self,
        spec: &RunSpec,
    ) -> Result<Box<dyn panowww_core::ports::Stream>, StartFailure> {
        let emit = Arc::clone(&self.emit);
        let zone = spec.zone.clone();
        let sink: panowww_core::ports::PreviewSink = Box::new(move |frame: PreviewFrame| {
            emit(RunEvent::Preview {
                image: data_url("image/jpeg", &frame.jpeg.0),
                display_w: frame.display_w,
                display_h: frame.display_h,
                zone: zone.clone(),
            });
        });
        let mut grabber = self.backend.grabber();
        grabber
            .start(spec.zone.display_id.clone(), spec.zone.rect(), sink)
            .map_err(|error| match error {
                panowww_core::ports::CaptureError::PermissionRevoked => {
                    StartFailure::ScreenRecording
                }
                panowww_core::ports::CaptureError::DisplayGone => StartFailure::DisplayGone {
                    display: spec.zone.display_id.0.clone(),
                },
                other => StartFailure::CaptureUnavailable {
                    message: other.to_string(),
                },
            })
    }

    /// Spec §4.5 step 1: permissions and the zone's display.
    fn preflight(&self, spec: &RunSpec) -> Result<(), StartFailure> {
        if !self.backend.accessibility_granted() {
            return Err(StartFailure::Accessibility);
        }
        if spec.game.device() == panowww_core::game::Device::Gamepad && !self.backend.plug_gamepad()
        {
            return Err(StartFailure::Vigem);
        }
        if !self.backend.screen_recording_granted() {
            return Err(StartFailure::ScreenRecording);
        }
        let displays = self
            .backend
            .displays()
            .map_err(|e| StartFailure::CaptureUnavailable {
                message: e.to_string(),
            })?;
        match zone_status(&spec.zone, &displays) {
            ZoneStatus::Ready => Ok(()),
            ZoneStatus::DisplayChanged | ZoneStatus::DisplayGone => {
                Err(StartFailure::DisplayGone {
                    display: spec.zone.display_id.0.clone(),
                })
            }
        }
    }

    /// Ticks the countdown once a second, in the window and on the overlay;
    /// `false` when cancelled. The preview switch is applied between ticks.
    fn countdown(
        &self,
        spec: &RunSpec,
        preview_flag: &Arc<Mutex<bool>>,
        stream: &mut dyn panowww_core::ports::Stream,
    ) -> bool {
        let seconds = u32::try_from(spec.countdown.as_secs()).unwrap_or(5);
        let mut applied = *preview_flag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for left in (1..=seconds).rev() {
            (self.emit)(RunEvent::Countdown { seconds_left: left });
            self.overlay.show_countdown(&spec.zone, left);
            let tick = Instant::now();
            while tick.elapsed() < Duration::from_secs(1) {
                if self.cancel.is_raised() {
                    return false;
                }
                let wanted = *preview_flag
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if wanted != applied {
                    stream.set_preview(wanted);
                    applied = wanted;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        !self.cancel.is_raised()
    }
}

/// `Progress` → `RunEvent`; `None` for the events the UI does not need
/// (`Finished` is sent by the worker with the folder).
fn translate(progress: Progress) -> Option<RunEvent> {
    Some(match progress {
        Progress::Started { expected } => RunEvent::Started { expected },
        Progress::Shot {
            index,
            row,
            col,
            capture_ms,
            thumbnail,
        } => RunEvent::Shot {
            index,
            row,
            col,
            capture_ms,
            thumbnail: thumbnail.map(|Jpeg(bytes)| data_url("image/jpeg", &bytes)),
        },
        Progress::Moved { row, col } => RunEvent::Moved { row, col },
        Progress::Sample { slot, frame } => {
            let slot = match slot {
                SampleSlot::First => SampleSlotDto::First,
                SampleSlot::Second => SampleSlotDto::Second,
                SampleSlot::BeforeFirstStep | SampleSlot::AfterFirstStep => return None,
            };
            RunEvent::Sample {
                slot,
                image: encode_jpeg(&frame)
                    .map_or_else(String::new, |bytes| data_url("image/jpeg", &bytes)),
            }
        }
        Progress::ScreenUnchanged => RunEvent::ScreenUnchanged,
        Progress::Preview(_) | Progress::Finished(_) => return None,
    })
}

/// Full-size JPEG of a Sample for the Check overlap window.
fn encode_jpeg(frame: &Frame) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 85)
        .encode(
            &frame.rgb,
            frame.width,
            frame.height,
            image::ExtendedColorType::Rgb8,
        )
        .ok()?;
    Some(bytes)
}

pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// The sink of Tests, Step once and Check overlap: nothing is written.
struct NoSink;

impl ShotSink for NoSink {
    fn begin(&mut self, _: &SessionStart) -> Result<(), panowww_core::ports::StoreError> {
        Ok(())
    }

    fn shot(
        &mut self,
        _: panowww_core::runner::ShotRecord,
        _: Frame,
    ) -> Result<(), panowww_core::ports::StoreError> {
        Ok(())
    }

    fn finish(&mut self, _: &RunOutcome) -> Result<(), panowww_core::ports::StoreError> {
        Ok(())
    }
}

/// The zone moved from its display's coordinates to global ones, where the
/// cursor is parked (issue 10): a Zone is stored relative to its display,
/// the cursor APIs take the whole desktop. A display without an origin
/// (gone, or the Linux stub) leaves it as is; the preflight has already
/// refused a gone display.
fn global_rect(zone: &Zone, origin: Option<(f64, f64)>) -> Rect {
    let rect = zone.rect();
    let Some((ox, oy)) = origin else {
        tracing::warn!(
            display = zone.display_id.0,
            "no display origin, drag parks display-local"
        );
        return rect;
    };
    let global = Rect {
        x: rect.x.saturating_add(round(ox)),
        y: rect.y.saturating_add(round(oy)),
        ..rect
    };
    tracing::debug!(?rect, ?global, "Capture zone in global coordinates");
    global
}

/// Rounds a display origin to whole units.
#[expect(
    clippy::cast_possible_truncation,
    reason = "display origins are far inside i32 once rounded"
)]
fn round(value: f64) -> i32 {
    value.round() as i32
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use panowww_core::game::{Game, NewGameDefaults};
    use panowww_core::ports::{DisplayId, DisplayInfo};

    use super::*;
    use crate::backend::fake::FakeBackend;
    use crate::overlay::Overlay;

    /// Records every overlay call, one line each.
    #[derive(Default)]
    struct RecordingOverlay(Mutex<Vec<String>>);

    impl RecordingOverlay {
        fn calls(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }
    }

    impl Overlay for RecordingOverlay {
        fn show_frame(&self, zone: &Zone) {
            self.0
                .lock()
                .unwrap()
                .push(format!("frame {}", zone.display_id.0));
        }
        fn show_countdown(&self, _: &Zone, seconds_left: u32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("countdown {seconds_left}"));
        }
        fn progress(&self, taken: u32, expected: u32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("progress {taken}/{expected}"));
        }
        fn end_pill(&self, stopped: bool, taken: u32, expected: u32) {
            let word = if stopped { "stopped" } else { "done" };
            self.0
                .lock()
                .unwrap()
                .push(format!("end {word} {taken}/{expected}"));
        }
        fn hide(&self) {
            self.0.lock().unwrap().push("hide".to_owned());
        }
        fn current(&self) -> Option<crate::dto::OverlayState> {
            None
        }
    }

    fn spec(sessions: &std::path::Path, kind: RunKindDto, countdown_secs: u64) -> RunSpec {
        let mut game = Game::new("Test", NewGameDefaults::default());
        game.shots_per_row = 3;
        game.rows = 2;
        game.wait_after_move_ms = 1;
        game.wait_after_shot_ms = 1;
        if let panowww_core::game::Input::Keyboard {
            turn_hold_ms,
            pitch_hold_ms,
            ..
        } = &mut game.input
        {
            *turn_hold_ms = 1;
            *pitch_hold_ms = 1;
        }
        RunSpec {
            game,
            kind,
            zone: Zone {
                display_id: DisplayId("main".to_owned()),
                x: 0,
                y: 0,
                w: 10,
                h: 10,
                display_w: 2560,
                display_h: 1440,
                scale: 2.0,
            },
            countdown: Duration::from_secs(countdown_secs),
            session: SessionSinkConfig::new(sessions, "test"),
            preview: true,
        }
    }

    fn runner(backend: Arc<FakeBackend>) -> (Runner, mpsc::Receiver<RunEvent>) {
        let (runner, rx, _) = runner_with_overlay(backend);
        (runner, rx)
    }

    fn runner_with_overlay(
        backend: Arc<FakeBackend>,
    ) -> (Runner, mpsc::Receiver<RunEvent>, Arc<RecordingOverlay>) {
        let (tx, rx) = mpsc::channel();
        let overlay = Arc::new(RecordingOverlay::default());
        let runner = Runner::new(
            backend,
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
            Arc::clone(&overlay) as SharedOverlay,
        );
        (runner, rx, overlay)
    }

    fn events(rx: &mpsc::Receiver<RunEvent>) -> Vec<String> {
        rx.try_iter()
            .map(|e| match e {
                RunEvent::Preview { .. } => "preview".to_owned(),
                RunEvent::Countdown { seconds_left } => format!("countdown {seconds_left}"),
                RunEvent::Shot { index, .. } => format!("shot {index}"),
                RunEvent::Moved { .. } => "moved".to_owned(),
                RunEvent::Finished { outcome, folder } => {
                    format!("finished {outcome:?} {}", folder.is_some())
                }
                other => format!("{other:?}"),
            })
            .collect()
    }

    #[test]
    fn a_session_streams_countdown_shots_and_finished_with_its_folder() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, rx) = runner(FakeBackend::new());
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 1)));
        assert_ne!(runner.phase(), RunnerPhase::Idle);
        runner.wait();
        assert_eq!(runner.phase(), RunnerPhase::Idle);
        let seen = events(&rx);
        assert_eq!(seen[0], "preview");
        assert_eq!(seen[1], "countdown 1");
        assert!(
            seen.contains(&"shot 1".to_owned()) && seen.contains(&"shot 6".to_owned()),
            "{seen:?}"
        );
        assert!(
            seen.last()
                .unwrap()
                .starts_with("finished Completed { shots: 6 } true"),
            "{seen:?}"
        );
        assert_eq!(
            std::fs::read_dir(dir.path().join("Test")).unwrap().count(),
            1,
            "one Session folder"
        );
    }

    #[test]
    fn start_while_running_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, rx) = runner(FakeBackend::new());
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 2)));
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            !runner.start(spec(dir.path(), RunKindDto::TurnTest, 1)),
            "second start ignored"
        );
        runner.wait();
        let seen = events(&rx);
        assert_eq!(seen.iter().filter(|e| e.starts_with("finished")).count(), 1);
    }

    #[test]
    fn stop_during_the_countdown_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, rx) = runner(FakeBackend::new());
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 5)));
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(runner.phase(), RunnerPhase::Countdown);
        runner.stop();
        runner.wait();
        assert_eq!(runner.phase(), RunnerPhase::Idle);
        let seen = events(&rx);
        assert_eq!(seen.last().unwrap(), "Cancelled", "{seen:?}");
        assert!(!seen.iter().any(|e| e.starts_with("shot")));
        assert!(!dir.path().join("Test").exists(), "no Session folder");
    }

    #[test]
    fn stop_during_the_run_interrupts_and_keeps_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, rx) = runner(FakeBackend::new());
        let mut s = spec(dir.path(), RunKindDto::Session, 1);
        s.game.wait_after_shot_ms = 200;
        assert!(runner.start(s));
        std::thread::sleep(Duration::from_millis(1300));
        assert_eq!(runner.phase(), RunnerPhase::Running);
        runner.stop();
        runner.wait();
        let seen = events(&rx);
        let last = seen.last().unwrap();
        assert!(last.starts_with("finished Interrupted"), "{seen:?}");
        assert!(dir.path().join("Test").exists());
    }

    #[test]
    fn missing_permissions_cannot_start() {
        let dir = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new();
        *backend.accessibility.lock().unwrap() = false;
        let (runner, rx) = runner(backend.clone());
        assert!(runner.start(spec(dir.path(), RunKindDto::TurnTest, 1)));
        runner.wait();
        assert_eq!(events(&rx), ["CannotStart(Accessibility)"]);

        *backend.accessibility.lock().unwrap() = true;
        *backend.screen_recording.lock().unwrap() = false;
        assert!(runner.start(spec(dir.path(), RunKindDto::TurnTest, 1)));
        runner.wait();
        assert_eq!(events(&rx), ["CannotStart(ScreenRecording)"]);
    }

    #[test]
    fn a_gone_display_or_a_failed_stream_cannot_start() {
        let dir = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new();
        backend.displays.lock().unwrap().clear();
        let (runner, rx) = runner(backend.clone());
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 1)));
        runner.wait();
        assert!(matches!(
            rx.try_recv(),
            Ok(RunEvent::CannotStart(StartFailure::DisplayGone { .. }))
        ));
        assert!(!dir.path().join("Test").exists());

        backend.displays.lock().unwrap().push(DisplayInfo {
            id: DisplayId("main".to_owned()),
            width: 2560,
            height: 1440,
            scale: 2.0,
            hdr: false,
        });
        *backend.capture_failure.lock().unwrap() = Some(
            panowww_core::ports::CaptureError::Unavailable("busy".to_owned()),
        );
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 1)));
        runner.wait();
        assert!(matches!(
            rx.try_recv(),
            Ok(RunEvent::CannotStart(StartFailure::CaptureUnavailable { message })) if message.contains("busy")
        ));
    }

    #[test]
    fn a_test_writes_nothing_and_check_overlap_sends_two_samples() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, rx) = runner(FakeBackend::new());
        assert!(runner.start(spec(dir.path(), RunKindDto::TurnTest, 1)));
        runner.wait();
        assert!(!dir.path().join("Test").exists());
        let seen = events(&rx);
        assert!(
            seen.last()
                .unwrap()
                .starts_with("finished Completed { shots: 0 } false"),
            "{seen:?}"
        );

        assert!(runner.start(spec(
            dir.path(),
            RunKindDto::CheckOverlap {
                axis: crate::dto::AxisDto::Turn
            },
            1
        )));
        runner.wait();
        let samples: Vec<RunEvent> = rx
            .try_iter()
            .filter(|e| matches!(e, RunEvent::Sample { .. }))
            .collect();
        assert_eq!(samples.len(), 2);
        assert!(
            matches!(&samples[0], RunEvent::Sample { slot: SampleSlotDto::First, image } if image.starts_with("data:image/jpeg;base64,"))
        );
    }

    #[test]
    fn the_overlay_follows_a_session_from_the_digit_to_the_end_pill() {
        let dir = tempfile::tempdir().unwrap();
        let (runner, _rx, overlay) = runner_with_overlay(FakeBackend::new());
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 1)));
        runner.wait();
        let calls = overlay.calls();
        assert_eq!(calls[0], "countdown 1");
        assert_eq!(calls[1], "frame main");
        assert_eq!(calls[2], "progress 0/6");
        assert!(calls.contains(&"progress 6/6".to_owned()), "{calls:?}");
        assert_eq!(calls.last().unwrap(), "end done 6/6");
    }

    #[test]
    fn the_overlay_hides_after_a_test_a_cancel_and_a_failed_start() {
        let dir = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new();
        let (runner, _rx, overlay) = runner_with_overlay(backend.clone());
        assert!(runner.start(spec(dir.path(), RunKindDto::TurnTest, 1)));
        runner.wait();
        assert_eq!(overlay.calls().last().unwrap(), "hide");
        assert!(!overlay.calls().iter().any(|c| c.starts_with("end")));

        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 5)));
        std::thread::sleep(Duration::from_millis(150));
        runner.stop();
        runner.wait();
        assert_eq!(overlay.calls().last().unwrap(), "hide");

        *backend.accessibility.lock().unwrap() = false;
        assert!(runner.start(spec(dir.path(), RunKindDto::Session, 1)));
        runner.wait();
        let calls = overlay.calls();
        assert_eq!(&calls[calls.len() - 2..], ["hide", "hide"]);
    }

    #[test]
    fn data_urls_carry_the_mime_and_base64() {
        assert_eq!(
            data_url("image/jpeg", b"\xFF\xD8"),
            "data:image/jpeg;base64,/9g="
        );
    }

    fn zone_at(x: i32, y: i32) -> Zone {
        Zone {
            display_id: DisplayId("second".to_owned()),
            x,
            y,
            w: 1000,
            h: 600,
            display_w: 2560,
            display_h: 1440,
            scale: 1.25,
        }
    }

    #[test]
    fn global_rect_adds_the_display_origin() {
        let zone = zone_at(100, 50);
        assert_eq!(
            global_rect(&zone, Some((-1920.0, 0.0))),
            Rect {
                x: -1820,
                y: 50,
                w: 1000,
                h: 600
            },
            "secondary on the left"
        );
        assert_eq!(
            global_rect(&zone, Some((1920.0, -180.0))),
            Rect {
                x: 2020,
                y: -130,
                w: 1000,
                h: 600
            },
            "secondary on the right, higher up"
        );
        assert_eq!(global_rect(&zone, Some((0.0, 0.0))), zone.rect(), "primary");
        assert_eq!(global_rect(&zone, None), zone.rect(), "no origin");
    }

    #[test]
    fn the_drag_parks_on_the_zone_display() {
        let centre = global_rect(&zone_at(780, 420), Some((1920.0, 0.0))).centre();
        assert_eq!((centre.x, centre.y), (3200.0, 720.0));
    }
}
