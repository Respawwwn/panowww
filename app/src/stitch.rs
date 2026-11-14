//! The single Stitch owner (spec §16.5): `Idle → Running → Idle` on a
//! thread named `panowww-stitch`. It reads the Session folder, runs the
//! stitch crate under the machine's memory budget, hands the result to the
//! store, and streams every step to the window as the `stitch` event. A
//! Stitch is refused while the Runner is not Idle; a run that starts
//! cancels the Stitch; so does quitting.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use panowww_core::ports::CancelFlag;
use panowww_stitch::{Grid, Limits, Stage, StitchError, StitchProgress, stitch};
use panowww_store::panorama::{FolderShotSource, write_panorama};
use panowww_store::session::{SessionRecord, SessionStatus};
use panowww_store::sink::SystemWallClock;

use crate::backend::SharedBackend;
use crate::dto::{StitchCancelReason, StitchEvent, StitchFailure, StitchPhase};

/// Where the Stitch's events go: the window, or a test's channel.
pub type StitchEventSink = Arc<dyn Fn(StitchEvent) + Send + Sync>;
/// Says whether the Runner is Idle.
pub type IdleProbe = Arc<dyn Fn() -> bool + Send + Sync>;

/// Progress events are at least this far apart, unless the stage changes
/// or a stage completes: the stitch crate reports far more often than a
/// bar can move.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(80);

/// The share of the RAM a Stitch may use (spec §16.5), between a floor a
/// small Session always gets and a cap that keeps a 128 GB machine from
/// holding every Shot at once.
const MEMORY_SHARE: u64 = 4;
const MEMORY_FLOOR: u64 = 1024 * 1024 * 1024;
const MEMORY_CAP: u64 = 8 * 1024 * 1024 * 1024;

/// What one Stitch needs, gathered by the command layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StitchSpec {
    pub folder: PathBuf,
    /// Written into the `panorama` block and the XMP.
    pub app_version: String,
}

pub struct StitchOwner {
    emit: StitchEventSink,
    runner_idle: IdleProbe,
    backend: SharedBackend,
    running: Arc<AtomicBool>,
    /// The Session being stitched, while one is.
    folder: Mutex<Option<PathBuf>>,
    /// The flag of the Stitch in progress and why it was raised.
    cancel: Arc<Mutex<Option<Cancellation>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Clone)]
struct Cancellation {
    flag: CancelFlag,
    reason: Arc<Mutex<Option<StitchCancelReason>>>,
}

impl std::fmt::Debug for StitchOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StitchOwner")
            .field("phase", &self.phase())
            .finish_non_exhaustive()
    }
}

impl StitchOwner {
    pub fn new(emit: StitchEventSink, runner_idle: IdleProbe, backend: SharedBackend) -> Self {
        Self {
            emit,
            runner_idle,
            backend,
            running: Arc::new(AtomicBool::new(false)),
            folder: Mutex::new(None),
            cancel: Arc::new(Mutex::new(None)),
            thread: Mutex::new(None),
        }
    }

    pub fn phase(&self) -> StitchPhase {
        if self.running.load(Ordering::SeqCst)
            && let Some(folder) = lock(&self.folder).as_ref()
        {
            StitchPhase::Running {
                folder: folder.display().to_string(),
            }
        } else {
            StitchPhase::Idle
        }
    }

    /// Starts the Stitch of `spec` on its thread. Returns `false`, and does
    /// nothing, while the Runner is not Idle or a Stitch is in progress.
    pub fn start(&self, spec: StitchSpec) -> bool {
        if !(self.runner_idle)() {
            tracing::info!("Stitch refused: a run is in progress");
            return false;
        }
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            tracing::info!("Stitch refused: one is in progress");
            return false;
        }
        self.join_finished();
        *lock(&self.folder) = Some(spec.folder.clone());
        let cancellation = Cancellation {
            flag: CancelFlag::new(),
            reason: Arc::new(Mutex::new(None)),
        };
        *lock(&self.cancel) = Some(cancellation.clone());
        let worker = Worker {
            emit: Arc::clone(&self.emit),
            running: Arc::clone(&self.running),
            cancellation,
            limits: Limits {
                memory_bytes: memory_budget(self.backend.total_memory_bytes()),
            },
        };
        let handle = std::thread::Builder::new()
            .name("panowww-stitch".to_owned())
            .spawn(move || worker.execute(&spec))
            .expect("spawning the Stitch thread");
        *lock(&self.thread) = Some(handle);
        true
    }

    /// Stops the Stitch in progress for `reason`; nothing is written. A
    /// no-op while Idle. The first reason wins.
    pub fn cancel(&self, reason: StitchCancelReason) {
        if let Some(cancellation) = lock(&self.cancel).as_ref() {
            lock(&cancellation.reason).get_or_insert(reason);
            cancellation.flag.raise();
        }
    }

    /// Cancels and waits for the thread: the quit path.
    pub fn shutdown(&self) {
        self.cancel(StitchCancelReason::Quit);
        if let Some(handle) = lock(&self.thread).take() {
            let _ = handle.join();
        }
    }

    /// Waits for the current Stitch to end, for tests.
    #[cfg(test)]
    pub fn wait(&self) {
        if let Some(handle) = lock(&self.thread).take() {
            let _ = handle.join();
        }
    }

    fn join_finished(&self) {
        if let Some(handle) = lock(&self.thread).take()
            && handle.is_finished()
        {
            let _ = handle.join();
        }
    }
}

/// The Stitch's share of `total` bytes of RAM; the default of the stitch
/// crate when the OS does not say.
pub fn memory_budget(total: Option<u64>) -> usize {
    let Some(total) = total else {
        return Limits::default().memory_bytes;
    };
    let share = (total / MEMORY_SHARE).clamp(MEMORY_FLOOR, MEMORY_CAP);
    usize::try_from(share).unwrap_or(usize::MAX)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The Stitch itself, on its own thread.
struct Worker {
    emit: StitchEventSink,
    running: Arc<AtomicBool>,
    cancellation: Cancellation,
    limits: Limits,
}

impl Worker {
    fn execute(self, spec: &StitchSpec) {
        let closing = self.read_stitch_write(spec);
        self.running.store(false, Ordering::SeqCst);
        (self.emit)(closing);
    }

    /// Reads the record, stitches, writes. Returns the closing event.
    fn read_stitch_write(&self, spec: &StitchSpec) -> StitchEvent {
        let started = Instant::now();
        let record = match SessionRecord::read(&spec.folder) {
            Ok(record) => record,
            Err(error) => return Self::failed(error.to_string()),
        };
        if record.status != SessionStatus::Completed {
            return StitchEvent::Failed {
                error: StitchFailure::NotCompleted,
            };
        }
        let source = FolderShotSource::from_record(&spec.folder, &record);
        let grid = Grid::from_game(&record.game);
        tracing::info!(
            folder = %spec.folder.display(),
            rows = grid.rows,
            shots_per_row = grid.shots_per_row,
            budget_mb = self.limits.memory_bytes / (1024 * 1024),
            "Stitch starting"
        );
        let progress = Throttle::new(Arc::clone(&self.emit));
        let stitched = match stitch(
            &source,
            &grid,
            &self.limits,
            &|p| progress.report(p),
            &self.cancellation.flag,
        ) {
            Ok(stitched) => stitched,
            Err(StitchError::Cancelled) => return self.cancelled(),
            Err(StitchError::TooFewLinked {
                linked,
                total,
                unlinked,
            }) => {
                return StitchEvent::Failed {
                    error: StitchFailure::TooFewLinked {
                        linked: u32::try_from(linked).unwrap_or(u32::MAX),
                        total: u32::try_from(total).unwrap_or(u32::MAX),
                        unlinked,
                    },
                };
            }
            Err(error) => return Self::failed(error.to_string()),
        };
        // Cancel lands between the stages too: nothing is written after it.
        if self.cancellation.flag.is_raised() {
            return self.cancelled();
        }
        (self.emit)(StitchEvent::Progress(StitchProgress::new(Stage::Save, 0.0)));
        match write_panorama(
            &spec.folder,
            &stitched,
            &spec.app_version,
            started,
            &SystemWallClock,
        ) {
            Ok(entry) => {
                tracing::info!(
                    width = entry.width,
                    height = entry.height,
                    duration_ms = entry.duration_ms,
                    "Panorama written"
                );
                (self.emit)(StitchEvent::Progress(StitchProgress::new(Stage::Save, 1.0)));
                StitchEvent::Finished { entry }
            }
            Err(error) => Self::failed(error.to_string()),
        }
    }

    fn cancelled(&self) -> StitchEvent {
        let reason = lock(&self.cancellation.reason).unwrap_or(StitchCancelReason::Button);
        tracing::info!(?reason, "Stitch stopped");
        StitchEvent::Cancelled { reason }
    }

    fn failed(message: String) -> StitchEvent {
        tracing::warn!(%message, "Stitch failed");
        StitchEvent::Failed {
            error: StitchFailure::Other { message },
        }
    }
}

/// Lets a progress report through when the stage changed, when a stage
/// completes, when it carries a measurement, or when [`PROGRESS_INTERVAL`]
/// has passed since the last one.
struct Throttle {
    emit: StitchEventSink,
    last: Mutex<Option<(Stage, Instant)>>,
}

impl Throttle {
    fn new(emit: StitchEventSink) -> Self {
        Self {
            emit,
            last: Mutex::new(None),
        }
    }

    fn report(&self, progress: StitchProgress) {
        let now = Instant::now();
        let mut last = lock(&self.last);
        let due = match *last {
            None => true,
            Some((stage, at)) => {
                stage != progress.stage
                    || progress.stage_fraction >= 1.0
                    || progress.measured.is_some()
                    || now.duration_since(at) >= PROGRESS_INTERVAL
            }
        };
        if !due {
            return;
        }
        *last = Some((progress.stage, now));
        drop(last);
        (self.emit)(StitchEvent::Progress(progress));
    }
}

/// `<folder>/panorama-preview.jpg`, for the preview window.
pub fn preview_path(folder: &Path) -> PathBuf {
    folder.join(panowww_store::panorama::PREVIEW_FILE)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use panowww_core::game::{Game, NewGameDefaults, StartPole};
    use panowww_core::ports::{DisplayId, Zone};
    use panowww_stitch::ShotSource;
    use panowww_stitch::synthetic::{Perturbation, session};
    use panowww_store::session::{
        CURRENT_VERSION, SessionFormat, SessionRecord, SessionStatus, ShotEntry, shot_file_name,
    };
    use panowww_store::{panorama, png};

    use super::*;
    use crate::backend::fake::FakeBackend;

    /// A synthetic Session written as the sink would have, `status` as
    /// asked. `shots_per_row` sizes it: 8 is quick, 14 takes seconds.
    fn write_session(root: &Path, shots_per_row: u32, status: SessionStatus) -> PathBuf {
        let grid = Grid {
            rows: if shots_per_row > 8 { 7 } else { 5 },
            shots_per_row,
            start_pole: StartPole::Nadir,
        };
        let (width, height) = if shots_per_row > 8 {
            (640, 320)
        } else {
            (320, 160)
        };
        let (source, _) = session(width, height, 90.0, &grid, &Perturbation::none(&grid));
        let folder = root.join("Synthetic").join("2026-11-14_15-00-00");
        std::fs::create_dir_all(&folder).unwrap();
        let mut game = Game::new("Synthetic", NewGameDefaults::default());
        game.rows = grid.rows;
        game.shots_per_row = grid.shots_per_row;
        game.start_pole = grid.start_pole;
        let mut shots = Vec::new();
        for i in 0..source.count() {
            let (row, col) = source.position(i);
            let file = shot_file_name(row, col);
            png::write(&folder.join(&file), &source.shot(i).unwrap()).unwrap();
            shots.push(ShotEntry {
                file,
                row,
                col,
                taken_at: format!("2026-11-14T15:00:{:02}+01:00", i % 60),
                capture_ms: 10,
            });
        }
        SessionRecord {
            format: SessionFormat::PanowwwSession,
            version: CURRENT_VERSION,
            id: uuid::Uuid::new_v4(),
            app_version: "0.0.1-test".to_owned(),
            started_at: "2026-11-14T15:00:00+01:00".to_owned(),
            ended_at: Some("2026-11-14T15:01:00+01:00".to_owned()),
            status,
            error: None,
            game,
            zone: Zone {
                display_id: DisplayId("main".to_owned()),
                x: 0,
                y: 0,
                w: width,
                h: height,
                display_w: 2560,
                display_h: 1440,
                scale: 1.0,
            },
            shots_expected: u32::try_from(shots.len()).unwrap(),
            shots_taken: u32::try_from(shots.len()).unwrap(),
            shots,
            panorama: None,
        }
        .write(&folder)
        .unwrap();
        folder
    }

    fn owner(runner_idle: bool) -> (StitchOwner, mpsc::Receiver<StitchEvent>) {
        let (tx, rx) = mpsc::channel();
        let owner = StitchOwner::new(
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
            Arc::new(move || runner_idle),
            FakeBackend::new(),
        );
        (owner, rx)
    }

    fn spec(folder: &Path) -> StitchSpec {
        StitchSpec {
            folder: folder.to_owned(),
            app_version: "1.1.0-test".to_owned(),
        }
    }

    fn names(rx: &mpsc::Receiver<StitchEvent>) -> Vec<String> {
        rx.try_iter()
            .map(|e| match e {
                StitchEvent::Progress(p) => format!("progress {:?} {:.2}", p.stage, p.overall),
                StitchEvent::Finished { entry } => {
                    format!("finished {}×{}", entry.width, entry.height)
                }
                StitchEvent::Failed { error } => format!("failed {error:?}"),
                StitchEvent::Cancelled { reason } => format!("cancelled {reason:?}"),
            })
            .collect()
    }

    #[test]
    fn a_stitch_is_refused_while_the_runner_is_not_idle() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write_session(dir.path(), 8, SessionStatus::Completed);
        let (owner, rx) = owner(false);
        assert!(!owner.start(spec(&folder)));
        assert_eq!(owner.phase(), StitchPhase::Idle);
        assert_eq!(names(&rx), Vec::<String>::new());
        assert!(!panorama::exists(&folder));
    }

    #[test]
    fn a_completed_stitch_streams_progress_then_finished_with_the_entry_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write_session(dir.path(), 8, SessionStatus::Completed);
        let (owner, rx) = owner(true);
        assert!(owner.start(spec(&folder)));
        assert_eq!(
            owner.phase(),
            StitchPhase::Running {
                folder: folder.display().to_string()
            }
        );
        assert!(!owner.start(spec(&folder)), "a second Stitch is refused");
        owner.wait();
        assert_eq!(owner.phase(), StitchPhase::Idle);
        let events: Vec<StitchEvent> = rx.try_iter().collect();
        let progress: Vec<&StitchProgress> = events
            .iter()
            .filter_map(|e| match e {
                StitchEvent::Progress(p) => Some(p),
                _ => None,
            })
            .collect();
        assert!(progress.len() >= 6, "{} progress events", progress.len());
        assert_eq!(progress[0].stage, Stage::Read);
        assert!(
            progress
                .windows(2)
                .all(|w| w[1].overall >= w[0].overall - 1e-6)
        );
        let last = progress.last().unwrap();
        assert_eq!((last.stage, last.stage_fraction), (Stage::Save, 1.0));
        let Some(StitchEvent::Finished { entry }) = events.last() else {
            panic!("{events:?}");
        };
        let record = SessionRecord::read(&folder).unwrap();
        assert_eq!(record.panorama.as_ref(), Some(entry));
        assert_eq!(entry.app_version, "1.1.0-test");
        assert!(panorama::exists(&folder));
        assert!(preview_path(&folder).is_file());
    }

    #[test]
    fn a_run_starting_mid_stitch_cancels_it_and_nothing_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write_session(dir.path(), 14, SessionStatus::Completed);
        let (owner, rx) = owner(true);
        assert!(owner.start(spec(&folder)));
        std::thread::sleep(Duration::from_millis(300));
        assert!(matches!(owner.phase(), StitchPhase::Running { .. }));
        // As Runner::start does through its hook; a later Cancel button
        // does not change the reason.
        owner.cancel(StitchCancelReason::SessionStarted);
        owner.cancel(StitchCancelReason::Button);
        owner.wait();
        assert_eq!(owner.phase(), StitchPhase::Idle);
        let seen = names(&rx);
        assert_eq!(seen.last().unwrap(), "cancelled SessionStarted", "{seen:?}");
        assert!(!panorama::exists(&folder));
        assert!(!preview_path(&folder).is_file());
        assert_eq!(SessionRecord::read(&folder).unwrap().panorama, None);
    }

    #[test]
    fn shutdown_cancels_for_quit_and_waits() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write_session(dir.path(), 14, SessionStatus::Completed);
        let (owner, rx) = owner(true);
        assert!(owner.start(spec(&folder)));
        std::thread::sleep(Duration::from_millis(200));
        owner.shutdown();
        assert_eq!(owner.phase(), StitchPhase::Idle);
        assert_eq!(names(&rx).last().unwrap(), "cancelled Quit");
        owner.shutdown();
    }

    #[test]
    fn an_interrupted_session_and_a_missing_folder_fail_without_a_thread_to_wait_for() {
        let dir = tempfile::tempdir().unwrap();
        let folder = write_session(dir.path(), 8, SessionStatus::Interrupted);
        let (owner, rx) = owner(true);
        assert!(owner.start(spec(&folder)));
        owner.wait();
        assert_eq!(names(&rx), ["failed NotCompleted"]);

        assert!(owner.start(spec(&dir.path().join("nowhere"))));
        owner.wait();
        let seen = names(&rx);
        assert!(
            seen[0].starts_with("failed Other") && seen[0].contains("session.json"),
            "{seen:?}"
        );
        assert_eq!(owner.phase(), StitchPhase::Idle);
    }

    #[test]
    fn the_memory_budget_is_a_quarter_between_the_floor_and_the_cap() {
        const GIB: u64 = 1024 * 1024 * 1024;
        let gib = |n: u64| usize::try_from(n * GIB).unwrap();
        assert_eq!(memory_budget(Some(16 * GIB)), gib(4));
        assert_eq!(memory_budget(Some(2 * GIB)), gib(1), "floor");
        assert_eq!(memory_budget(Some(128 * GIB)), gib(8), "cap");
        assert_eq!(memory_budget(None), Limits::default().memory_bytes);
    }

    #[test]
    fn the_throttle_passes_stage_changes_and_completions_at_once() {
        let (tx, rx) = mpsc::channel();
        let throttle = Throttle::new(Arc::new(move |e| {
            let _ = tx.send(e);
        }));
        throttle.report(StitchProgress::new(Stage::Read, 0.0));
        throttle.report(StitchProgress::new(Stage::Read, 0.1));
        throttle.report(StitchProgress::new(Stage::Read, 0.2));
        throttle.report(StitchProgress::new(Stage::Read, 1.0));
        throttle.report(StitchProgress::new(Stage::Overlaps, 0.0));
        throttle.report(StitchProgress::new(Stage::Overlaps, 0.1));
        let seen = names(&rx);
        assert_eq!(
            seen,
            [
                "progress Read 0.00",
                "progress Read 0.07",
                "progress Overlaps 0.07"
            ]
        );
    }
}
