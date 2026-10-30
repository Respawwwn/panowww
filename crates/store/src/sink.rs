//! The `ShotSink` of a real Session (spec §6, §9): the folder
//! `<sessionsFolder>/<Game name>/<timestamp>/` created at the first Shot,
//! `session.json` written `running` then rewritten with the outcome, every
//! Shot handed to the PNG worker.

use std::path::{Path, PathBuf};

use panowww_core::game::name::sanitize_stem;
use panowww_core::ports::{Frame, ShotSink};
use panowww_core::runner::{RunError, RunErrorKind, RunOutcome, SessionStart, ShotRecord};
use uuid::Uuid;

use crate::StoreError;
use crate::png::{DEFAULT_QUEUE_CAPACITY, PngJob, PngWorker};
use crate::session::{
    CURRENT_VERSION, FOLDER_TIMESTAMP, SessionFormat, SessionRecord, SessionStatus, ShotEntry,
    TIMESTAMP, shot_file_name,
};

/// Wall-clock time for folder names and timestamps; injected so tests are
/// deterministic.
pub trait WallClock: Send + Sync {
    fn now(&self) -> jiff::Zoned;
}

/// The machine's clock in its local time zone.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemWallClock;

impl WallClock for SystemWallClock {
    fn now(&self) -> jiff::Zoned {
        jiff::Zoned::now()
    }
}

/// What a sink needs to know besides the run itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSinkConfig {
    /// Settings → Sessions folder.
    pub sessions_folder: PathBuf,
    /// Written into `session.json` as `appVersion`.
    pub app_version: String,
    pub queue_capacity: usize,
}

impl SessionSinkConfig {
    pub fn new(sessions_folder: impl Into<PathBuf>, app_version: &str) -> Self {
        Self {
            sessions_folder: sessions_folder.into(),
            app_version: app_version.to_owned(),
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
        }
    }
}

/// One sink serves one Session; `app` makes a new one per run.
pub struct SessionSink {
    config: SessionSinkConfig,
    clock: Box<dyn WallClock>,
    open: Option<OpenSession>,
    /// The folder of the Session this sink wrote, once begun.
    folder: Option<PathBuf>,
}

struct OpenSession {
    folder: PathBuf,
    record: SessionRecord,
    worker: PngWorker,
}

impl std::fmt::Debug for SessionSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionSink")
            .field("config", &self.config)
            .field("folder", &self.folder)
            .finish_non_exhaustive()
    }
}

impl SessionSink {
    pub fn new(config: SessionSinkConfig, clock: Box<dyn WallClock>) -> Self {
        Self {
            config,
            clock,
            open: None,
            folder: None,
        }
    }

    /// The Session folder, once the first Shot created it.
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    fn timestamp(&self) -> String {
        self.clock.now().strftime(TIMESTAMP).to_string()
    }

    /// `<sessionsFolder>/<Game name>/<timestamp>`, with ` (2)`, ` (3)`… when
    /// a Session of the same second already exists.
    fn new_folder(&self, game_name: &str) -> Result<PathBuf, StoreError> {
        let parent = self.config.sessions_folder.join(sanitize_stem(game_name));
        let stamp = self.clock.now().strftime(FOLDER_TIMESTAMP).to_string();
        let mut candidate = parent.join(&stamp);
        let mut n = 2;
        while candidate.exists() {
            candidate = parent.join(format!("{stamp} ({n})"));
            n += 1;
        }
        std::fs::create_dir_all(&candidate).map_err(|e| StoreError::io("create", &candidate, e))?;
        Ok(candidate)
    }

    fn open_mut(&mut self) -> Result<&mut OpenSession, StoreError> {
        self.open.as_mut().ok_or_else(|| StoreError::Png {
            path: PathBuf::new(),
            message: "no Session was begun".to_owned(),
        })
    }
}

impl ShotSink for SessionSink {
    fn begin(&mut self, start: &SessionStart) -> Result<(), panowww_core::ports::StoreError> {
        let folder = self.new_folder(&start.game.name)?;
        let record = SessionRecord {
            format: SessionFormat::PanowwwSession,
            version: CURRENT_VERSION,
            id: Uuid::new_v4(),
            app_version: self.config.app_version.clone(),
            started_at: self.timestamp(),
            ended_at: None,
            status: SessionStatus::Running,
            error: None,
            game: start.game.clone(),
            zone: start.zone.clone(),
            shots_expected: start.shots_expected,
            shots_taken: 0,
            shots: Vec::new(),
        };
        record.write(&folder)?;
        tracing::info!(folder = %folder.display(), "Session folder created");
        self.folder = Some(folder.clone());
        self.open = Some(OpenSession {
            folder,
            record,
            worker: PngWorker::start(self.config.queue_capacity),
        });
        Ok(())
    }

    fn shot(
        &mut self,
        record: ShotRecord,
        frame: Frame,
    ) -> Result<(), panowww_core::ports::StoreError> {
        let taken_at = self.timestamp();
        let open = self.open_mut()?;
        let file = shot_file_name(record.row, record.col);
        open.worker.submit(PngJob {
            path: open.folder.join(&file),
            frame,
        })?;
        open.record.shots.push(ShotEntry {
            file,
            row: record.row,
            col: record.col,
            taken_at,
            capture_ms: record.capture_ms,
        });
        Ok(())
    }

    fn finish(&mut self, outcome: &RunOutcome) -> Result<(), panowww_core::ports::StoreError> {
        let ended_at = self.timestamp();
        let Some(OpenSession {
            folder,
            mut record,
            worker,
        }) = self.open.take()
        else {
            return Ok(());
        };
        let flushed = worker.finish();
        record.ended_at = Some(ended_at);
        record.shots_taken = outcome.shots();
        (record.status, record.error) = match outcome {
            RunOutcome::Completed { .. } => (SessionStatus::Completed, None),
            RunOutcome::Interrupted { .. } => (SessionStatus::Interrupted, None),
            RunOutcome::Failed { error, .. } => (SessionStatus::Failed, Some(error.clone())),
        };
        if let Err(error) = &flushed {
            // A Shot never reached the disk: the folder has a hole, so the
            // Session is failed whatever the Runner concluded.
            record.status = SessionStatus::Failed;
            record.error = Some(RunError {
                kind: RunErrorKind::Store,
                message: error.to_string(),
            });
            record.shots_taken = record.shots_taken.min(written_shots(&folder, &record));
        }
        record.write(&folder)?;
        tracing::info!(folder = %folder.display(), status = ?record.status, shots = record.shots_taken, "Session closed");
        flushed?;
        Ok(())
    }
}

/// How many of the recorded Shots are on disk.
fn written_shots(folder: &Path, record: &SessionRecord) -> u32 {
    let count = record
        .shots
        .iter()
        .filter(|s| folder.join(&s.file).exists())
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}
