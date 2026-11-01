//! A fake Session through the real sink into a temporary Sessions folder:
//! the folder tree, `session.json`, the listing, stale Sessions, zip,
//! thumbnails and delete.

use std::path::Path;
use std::sync::Mutex;

use panowww_core::fakes::{FakeClock, FakeStream, MemorySink, RecordingCamera};
use panowww_core::game::{Game, NewGameDefaults};
use panowww_core::plan::{RunKind, plan};
use panowww_core::ports::{CancelFlag, DisplayId, Zone};
use panowww_core::runner::{RunErrorKind, RunOutcome, RunRequest, SessionStart, run};
use panowww_store::session::{SessionRecord, SessionStatus};
use panowww_store::sessions::{
    delete_session, folder_size, list_all, list_for_game, mark_stale_running, zip_path, zip_session,
};
use panowww_store::sink::{SessionSink, SessionSinkConfig, WallClock};
use panowww_store::thumbnails::{history_thumbnail, remove_thumbnails};
use panowww_store::{AppPaths, png};

/// A clock that ticks one second per call from a fixed start.
struct TickingClock(Mutex<jiff::Zoned>);

impl TickingClock {
    fn starting(at: &str) -> Self {
        let zoned: jiff::Zoned = at.parse().unwrap();
        Self(Mutex::new(zoned))
    }
}

impl WallClock for TickingClock {
    fn now(&self) -> jiff::Zoned {
        let mut now = self.0.lock().unwrap();
        let current = now.clone();
        *now = now.checked_add(jiff::Span::new().seconds(1)).unwrap();
        current
    }
}

fn game(name: &str) -> Game {
    let mut game = Game::new(name, NewGameDefaults::default());
    game.shots_per_row = 3;
    game.rows = 2;
    game.wait_after_move_ms = 10;
    game.wait_after_shot_ms = 10;
    game
}

fn zone() -> Zone {
    Zone {
        display_id: DisplayId("main".to_owned()),
        x: 0,
        y: 0,
        w: 64,
        h: 36,
        display_w: 2560,
        display_h: 1440,
        scale: 2.0,
    }
}

fn sink(sessions_folder: &Path, start: &str) -> SessionSink {
    SessionSink::new(
        SessionSinkConfig::new(sessions_folder, "0.0.1-test"),
        Box::new(TickingClock::starting(start)),
    )
}

/// Runs a Session of `game` through the fakes into `sink`.
fn run_session(
    game: &Game,
    sink: &mut SessionSink,
    mut stream: FakeStream,
    clock: &FakeClock,
) -> RunOutcome {
    let plan = plan(game, RunKind::Session);
    let request = RunRequest {
        plan: &plan,
        zone: zone().rect(),
        session: Some(SessionStart {
            game: game.clone(),
            zone: zone(),
            shots_expected: u32::try_from(plan.expected_shots()).unwrap(),
        }),
    };
    run(
        &request,
        &mut RecordingCamera::new(),
        &mut stream,
        sink,
        clock,
        &CancelFlag::new(),
        &|_| {},
    )
}

#[test]
fn a_completed_session_produces_the_expected_tree_and_record() {
    let root = tempfile::tempdir().unwrap();
    let sessions = root.path().join("Pictures/Panowww");
    let game = game("Elden Ring: photo mode");
    let mut sink = sink(&sessions, "2026-10-30T15:04:05+01:00[+01:00]");

    let outcome = run_session(&game, &mut sink, FakeStream::new(64, 36), &FakeClock::new());
    assert_eq!(outcome, RunOutcome::Completed { shots: 6 });

    let folder = sink.folder().unwrap().to_owned();
    assert_eq!(
        folder.strip_prefix(&sessions).unwrap(),
        Path::new("Elden Ring- photo mode/2026-10-30_15-04-05")
    );
    let mut files: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "r01_c01.png",
            "r01_c02.png",
            "r01_c03.png",
            "r02_c01.png",
            "r02_c02.png",
            "r02_c03.png",
            "session.json"
        ]
    );
    let frame = png::read(&folder.join("r02_c03.png")).unwrap();
    assert_eq!((frame.width, frame.height), (64, 36));
    assert_eq!(frame.rgb[0], 6, "the sixth capture");

    let record = SessionRecord::read(&folder).unwrap();
    assert_eq!(record.status, SessionStatus::Completed);
    assert_eq!(record.app_version, "0.0.1-test");
    assert_eq!(record.started_at, "2026-10-30T15:04:06+01:00");
    assert!(record.ended_at.is_some());
    assert_eq!((record.shots_expected, record.shots_taken), (6, 6));
    assert_eq!(record.shots.len(), 6);
    assert_eq!(record.shots[5].file, "r02_c03.png");
    assert_eq!((record.shots[5].row, record.shots[5].col), (2, 3));
    assert_eq!(record.game, game);
    assert_eq!(record.zone, zone());
    assert_eq!(record.error, None);

    let text = std::fs::read_to_string(folder.join("session.json")).unwrap();
    assert!(text.contains(r#""format": "panowww-session""#));
    assert!(text.contains(r#""status": "completed""#));
    assert!(
        text.contains(r#""startedAt""#) && text.contains(r#""shotsExpected": 6"#),
        "{text}"
    );
}

#[test]
fn two_sessions_in_the_same_second_get_distinct_folders() {
    let root = tempfile::tempdir().unwrap();
    let game = game("Same second");
    let frozen = "2026-01-01T00:00:00+00:00[UTC]";
    let mut first = SessionSink::new(
        SessionSinkConfig::new(root.path(), "t"),
        Box::new(FrozenClock(frozen.parse().unwrap())),
    );
    let mut second = SessionSink::new(
        SessionSinkConfig::new(root.path(), "t"),
        Box::new(FrozenClock(frozen.parse().unwrap())),
    );
    run_session(&game, &mut first, FakeStream::new(8, 8), &FakeClock::new());
    run_session(&game, &mut second, FakeStream::new(8, 8), &FakeClock::new());
    assert!(first.folder().unwrap().ends_with("2026-01-01_00-00-00"));
    assert!(
        second
            .folder()
            .unwrap()
            .ends_with("2026-01-01_00-00-00 (2)")
    );
}

struct FrozenClock(jiff::Zoned);

impl WallClock for FrozenClock {
    fn now(&self) -> jiff::Zoned {
        self.0.clone()
    }
}

#[test]
fn an_interrupted_session_keeps_its_shots() {
    let root = tempfile::tempdir().unwrap();
    let game = game("Stopped");
    let mut sink = sink(root.path(), "2026-10-30T15:04:05+01:00[+01:00]");
    // Shot, wait(1), key_down, wait(2), key_up, wait(3), Shot, wait(4), key_down, wait(5) -> cancel.
    let outcome = run_session(
        &game,
        &mut sink,
        FakeStream::new(8, 8),
        &FakeClock::new().cancelling_on_wait(5),
    );
    assert_eq!(outcome, RunOutcome::Interrupted { shots: 2 });
    let record = SessionRecord::read(sink.folder().unwrap()).unwrap();
    assert_eq!(record.status, SessionStatus::Interrupted);
    assert_eq!(record.shots_taken, 2);
    assert_eq!(record.shots.len(), 2);
    assert!(sink.folder().unwrap().join("r01_c02.png").exists());
    assert!(!sink.folder().unwrap().join("r01_c03.png").exists());
}

#[test]
fn a_failed_session_records_the_error() {
    let root = tempfile::tempdir().unwrap();
    let game = game("Failed");
    let mut sink = sink(root.path(), "2026-10-30T15:04:05+01:00[+01:00]");
    let outcome = run_session(
        &game,
        &mut sink,
        FakeStream::new(8, 8).failing_on([3, 4]),
        &FakeClock::new(),
    );
    assert!(matches!(outcome, RunOutcome::Failed { shots: 2, .. }));
    let record = SessionRecord::read(sink.folder().unwrap()).unwrap();
    assert_eq!(record.status, SessionStatus::Failed);
    assert_eq!(
        record.error.as_ref().map(|e| e.kind),
        Some(RunErrorKind::ShotFailed)
    );
    assert_eq!(record.shots_taken, 2);
}

#[test]
fn tests_never_touch_the_sessions_folder() {
    let root = tempfile::tempdir().unwrap();
    let game = game("Test only");
    let plan = plan(&game, RunKind::TurnTest);
    let request = RunRequest {
        plan: &plan,
        zone: zone().rect(),
        session: None,
    };
    let mut sink = sink(root.path(), "2026-10-30T15:04:05+01:00[+01:00]");
    run(
        &request,
        &mut RecordingCamera::new(),
        &mut FakeStream::new(8, 8),
        &mut sink,
        &FakeClock::new(),
        &CancelFlag::new(),
        &|_| {},
    );
    assert_eq!(sink.folder(), None);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let _ = MemorySink::new();
}

#[test]
fn listing_is_newest_first_per_game_or_global_and_sizes_add_up() {
    let root = tempfile::tempdir().unwrap();
    let a = game("Alpha");
    let b = game("Beta");
    let mut sink_a1 = sink(root.path(), "2026-10-30T10:00:00+01:00[+01:00]");
    let mut sink_b = sink(root.path(), "2026-10-30T11:00:00+01:00[+01:00]");
    let mut sink_a2 = sink(root.path(), "2026-10-30T12:00:00+01:00[+01:00]");
    run_session(&a, &mut sink_a1, FakeStream::new(8, 8), &FakeClock::new());
    run_session(&b, &mut sink_b, FakeStream::new(8, 8), &FakeClock::new());
    run_session(&a, &mut sink_a2, FakeStream::new(8, 8), &FakeClock::new());

    let all = list_all(root.path()).unwrap();
    let names: Vec<&str> = all.iter().map(|s| s.record.game.name.as_str()).collect();
    assert_eq!(names, ["Alpha", "Beta", "Alpha"]);
    assert_eq!(all[0].folder, sink_a2.folder().unwrap());

    let alphas = list_for_game(root.path(), a.id).unwrap();
    assert_eq!(alphas.len(), 2);
    assert!(alphas.iter().all(|s| s.record.game.id == a.id));

    let size = folder_size(&all[0].folder).unwrap();
    let json_len = std::fs::metadata(all[0].folder.join("session.json"))
        .unwrap()
        .len();
    assert!(size > json_len, "{size} bytes should include the PNG files");
}

#[test]
fn a_running_session_is_marked_interrupted_at_the_next_launch() {
    let root = tempfile::tempdir().unwrap();
    let game = game("Crashed");
    let mut sink = sink(root.path(), "2026-10-30T15:04:05+01:00[+01:00]");
    run_session(&game, &mut sink, FakeStream::new(8, 8), &FakeClock::new());
    let folder = sink.folder().unwrap().to_owned();
    // Rewind the record to how a crash leaves it: running, no Shots listed,
    // files on disk.
    let mut record = SessionRecord::read(&folder).unwrap();
    record.status = SessionStatus::Running;
    record.ended_at = None;
    record.shots_taken = 0;
    record.shots.clear();
    record.write(&folder).unwrap();
    std::fs::remove_file(folder.join("r02_c03.png")).unwrap();

    let clock = FrozenClock("2026-10-31T09:00:00+01:00[+01:00]".parse().unwrap());
    assert_eq!(mark_stale_running(root.path(), &clock).unwrap(), 1);
    let record = SessionRecord::read(&folder).unwrap();
    assert_eq!(record.status, SessionStatus::Interrupted);
    assert_eq!(
        record.ended_at.as_deref(),
        Some("2026-10-31T09:00:00+01:00")
    );
    assert_eq!(record.shots_taken, 5, "the Shots found on disk");
    assert_eq!(record.shots.len(), 5);
    assert_eq!(record.shots[0].file, "r01_c01.png");
    assert_eq!(
        mark_stale_running(root.path(), &clock).unwrap(),
        0,
        "idempotent"
    );
    assert_eq!(
        list_all(root.path()).unwrap()[0].record.status,
        SessionStatus::Interrupted
    );
}

#[test]
fn zip_thumbnails_and_delete() {
    let root = tempfile::tempdir().unwrap();
    let data = AppPaths::new(root.path().join("data"));
    let sessions = root.path().join("sessions");
    let game = game("Zipped");
    let mut sink = sink(&sessions, "2026-10-30T15:04:05+01:00[+01:00]");
    run_session(
        &game,
        &mut sink,
        FakeStream::new(640, 360),
        &FakeClock::new(),
    );
    let folder = sink.folder().unwrap().to_owned();
    let record = SessionRecord::read(&folder).unwrap();

    let zip = zip_session(&folder).unwrap();
    assert_eq!(zip, zip_path(&folder));
    assert!(zip.ends_with("2026-10-30_15-04-05.zip"));
    let archive = zip::ZipArchive::new(std::fs::File::open(&zip).unwrap()).unwrap();
    assert_eq!(archive.len(), 7, "6 Shots and session.json");
    assert!(
        archive
            .file_names()
            .any(|n| n.as_deref().ok() == Some("r01_c01.png"))
    );

    let thumb = history_thumbnail(&data, &folder, &record, "r01_c02.png").unwrap();
    assert!(thumb.starts_with(data.thumbnails_dir().join(record.id.to_string())));
    assert!(thumb.ends_with("r01_c02.jpg"));
    let decoded = image::load_from_memory(&std::fs::read(&thumb).unwrap()).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (192, 108));
    let again = history_thumbnail(&data, &folder, &record, "r01_c02.png").unwrap();
    assert_eq!(again, thumb, "cached");

    delete_session(&folder).unwrap();
    remove_thumbnails(&data, &record).unwrap();
    assert!(!folder.exists() && !zip.exists() && !thumb.exists());
    assert_eq!(list_all(&sessions).unwrap(), []);
    delete_session(&folder).unwrap_err();
}
