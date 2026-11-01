//! Fixtures for the UI pass: fake Games and Sessions on disk, through the
//! same store code the app uses, so every visual case of the Sessions
//! tables can be reviewed without shooting anything.
//!
//! ```text
//! cargo run -p panowww-store --example fixtures -- [--data DIR] [--sessions DIR]
//!     [--game NAME]... [--no-games] [--clean]
//! ```
//!
//! Writes three Games (keyboard, mouse drag, gamepad) and one damaged file
//! into the app's Games folder, then for each of them, and for every
//! `--game NAME` already on the machine, one Session per case: completed
//! today, interrupted today, failed yesterday (display gone), completed
//! three days ago, failed ten days ago (capture unavailable), completed
//! last month and last year, and one still running. Shots are small
//! gradient PNG files. `--clean` removes the fixture Games and their Sessions
//! first; the named Games are never deleted.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use jiff::{ToSpan, Zoned};
use panowww_core::game::{DragButton, Game, Input, MouseMode, NewGameDefaults, PhysicalKey};
use panowww_core::ports::{DisplayId, Frame, ShotSink, Zone};
use panowww_core::runner::{RunError, RunErrorKind, RunOutcome, SessionStart, ShotRecord};
use panowww_store::AppPaths;
use panowww_store::games::GamesFolder;
use panowww_store::session::{SessionRecord, SessionStatus};
use panowww_store::settings::Settings;
use panowww_store::sink::{SessionSink, SessionSinkConfig, WallClock};

const FIXTURE_PREFIX: &str = "Fixture · ";
const DAMAGED_FILE: &str = "Fixture · Damaged.panowww.json";

struct Args {
    data: PathBuf,
    sessions: Option<PathBuf>,
    games: Vec<String>,
    no_games: bool,
    clean: bool,
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let mut args = Args {
        data: default_data_dir()?,
        sessions: None,
        games: Vec::new(),
        no_games: false,
        clean: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--data" => args.data = PathBuf::from(it.next().ok_or("--data needs a path")?),
            "--sessions" => {
                args.sessions = Some(PathBuf::from(it.next().ok_or("--sessions needs a path")?));
            }
            "--game" => args.games.push(it.next().ok_or("--game needs a name")?),
            "--no-games" => args.no_games = true,
            "--clean" => args.clean = true,
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok(args)
}

/// Where the app keeps its data: Tauri's `app_data_dir` for the bundle id.
fn default_data_dir() -> Result<PathBuf, Box<dyn Error>> {
    if cfg!(target_os = "macos") {
        let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
        Ok(PathBuf::from(home)
            .join("Library/Application Support")
            .join("com.respawwwn.panowww"))
    } else if cfg!(windows) {
        let appdata = std::env::var_os("APPDATA").ok_or("APPDATA is not set")?;
        Ok(PathBuf::from(appdata).join("com.respawwwn.panowww"))
    } else {
        let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
        Ok(PathBuf::from(home)
            .join(".local/share")
            .join("com.respawwwn.panowww"))
    }
}

/// A clock set by hand, so Sessions land on the dates of each case.
struct FixedClock(Mutex<Zoned>);

impl FixedClock {
    fn new(at: Zoned) -> Self {
        Self(Mutex::new(at))
    }

    fn set(&self, at: Zoned) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = at;
    }
}

impl WallClock for FixedClock {
    fn now(&self) -> Zoned {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// How one fixture Session ends.
enum Ending {
    Completed,
    Interrupted {
        taken: u32,
    },
    Failed {
        taken: u32,
        kind: RunErrorKind,
        message: &'static str,
    },
    Running {
        taken: u32,
    },
}

struct Case {
    started: Zoned,
    /// Seconds between two Shots, so the end time follows.
    pace_secs: i64,
    ending: Ending,
}

fn cases(now: &Zoned) -> Result<Vec<Case>, Box<dyn Error>> {
    let at = |z: Zoned, h: i8, m: i8| -> Result<Zoned, Box<dyn Error>> {
        Ok(z.with().hour(h).minute(m).second(0).build()?)
    };
    Ok(vec![
        Case {
            started: now.checked_sub(2.hours())?,
            pace_secs: 3,
            ending: Ending::Completed,
        },
        Case {
            started: now.checked_sub(45.minutes())?,
            pace_secs: 3,
            ending: Ending::Interrupted { taken: 31 },
        },
        Case {
            started: at(now.yesterday()?, 22, 5)?,
            pace_secs: 3,
            ending: Ending::Failed {
                taken: 52,
                kind: RunErrorKind::DisplayGone,
                message: "Display 1 was disconnected during the Session.",
            },
        },
        Case {
            started: at(now.checked_sub(3.days())?, 19, 48)?,
            pace_secs: 4,
            ending: Ending::Completed,
        },
        Case {
            started: at(now.checked_sub(10.days())?, 11, 20)?,
            pace_secs: 3,
            ending: Ending::Failed {
                taken: 12,
                kind: RunErrorKind::CaptureUnavailable,
                message: "The display could not be captured.",
            },
        },
        Case {
            started: at(now.checked_sub(40.days())?, 9, 2)?,
            pace_secs: 3,
            ending: Ending::Completed,
        },
        Case {
            started: at(now.checked_sub(400.days())?, 16, 30)?,
            pace_secs: 3,
            ending: Ending::Completed,
        },
        Case {
            started: now.checked_sub(4.minutes())?,
            pace_secs: 3,
            ending: Ending::Running { taken: 17 },
        },
    ])
}

fn fixture_games() -> Vec<Game> {
    let defaults = NewGameDefaults::default();
    let mut keyboard = Game::new(&format!("{FIXTURE_PREFIX}Keyboard"), defaults);
    keyboard.notes = Some("Photo mode, HUD off, borderless 2560 × 1440.".to_owned());
    keyboard.input = Input::Keyboard {
        turn_right_key: PhysicalKey::new("KeyD"),
        look_up_key: PhysicalKey::new("KeyW"),
        look_down_key: PhysicalKey::new("KeyS"),
        turn_hold_ms: 100,
        pitch_hold_ms: 100,
    };
    let mut mouse = Game::new(&format!("{FIXTURE_PREFIX}Mouse drag"), defaults);
    mouse.notes = Some("Cinematic camera off. Wait for the weather to settle.".to_owned());
    mouse.input = Input::Mouse {
        mode: MouseMode::Drag,
        drag_button: Some(DragButton::Left),
        turn_delta: 840,
        pitch_delta: 610,
        invert_vertical: false,
    };
    mouse.shots_per_row = 12;
    mouse.rows = 5;
    let mut gamepad = Game::new(&format!("{FIXTURE_PREFIX}Gamepad"), defaults);
    gamepad.notes = Some("Lock-on off. Stand still in a safe spot.".to_owned());
    gamepad.input = Input::Gamepad {
        stick_deflection: 0.8,
        turn_hold_ms: 120,
        pitch_hold_ms: 120,
        invert_vertical: true,
    };
    vec![keyboard, mouse, gamepad]
}

fn zone() -> Zone {
    Zone {
        display_id: DisplayId("fixture-display".to_owned()),
        x: 320,
        y: 180,
        w: 960,
        h: 540,
        display_w: 2560,
        display_h: 1440,
        scale: 2.0,
    }
}

/// A gradient that drifts with the Shot index, so thumbnails differ.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "values are kept inside 0..=255 before the cast"
)]
fn frame(width: u32, height: u32, index: u32) -> Frame {
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    let shift = (index * 23) % 256;
    for y in 0..height {
        for x in 0..width {
            let r = ((x * 255) / width.max(1) + shift) % 256;
            let g = (y * 255) / height.max(1);
            let b = (255 - r + shift / 2) % 256;
            rgb.extend_from_slice(&[r as u8, g as u8, b as u8]);
        }
    }
    Frame { width, height, rgb }
}

fn write_session(
    sessions_folder: &Path,
    game: &Game,
    case: &Case,
) -> Result<PathBuf, Box<dyn Error>> {
    let clock = std::sync::Arc::new(FixedClock::new(case.started.clone()));
    let expected = game.shots_per_row * game.rows;
    let taken = match case.ending {
        Ending::Completed => expected,
        Ending::Interrupted { taken }
        | Ending::Failed { taken, .. }
        | Ending::Running { taken } => taken.min(expected),
    };
    let mut sink = SessionSink::new(
        SessionSinkConfig::new(sessions_folder, "fixtures"),
        Box::new(ClockHandle(std::sync::Arc::clone(&clock))),
    );
    let zone = zone();
    sink.begin(&SessionStart {
        game: game.clone(),
        zone: zone.clone(),
        shots_expected: expected,
    })?;
    for i in 0..taken {
        let row = i / game.shots_per_row + 1;
        let col = i % game.shots_per_row + 1;
        sink.shot(
            ShotRecord {
                index: i + 1,
                row,
                col,
                capture_ms: 38 + (i % 7),
            },
            frame(zone.w, zone.h, i),
        )?;
    }
    clock.set(
        case.started
            .checked_add((case.pace_secs * i64::from(taken.max(1))).seconds())?,
    );
    let outcome = match &case.ending {
        Ending::Completed => RunOutcome::Completed { shots: taken },
        Ending::Interrupted { .. } | Ending::Running { .. } => {
            RunOutcome::Interrupted { shots: taken }
        }
        Ending::Failed { kind, message, .. } => RunOutcome::Failed {
            shots: taken,
            error: RunError {
                kind: *kind,
                message: (*message).to_owned(),
            },
        },
    };
    sink.finish(&outcome)?;
    let folder = sink
        .folder()
        .ok_or("the sink made no folder")?
        .to_path_buf();
    if matches!(case.ending, Ending::Running { .. }) {
        // As a crash would leave it: running, no end.
        let mut record = SessionRecord::read(&folder)?;
        record.status = SessionStatus::Running;
        record.ended_at = None;
        record.write(&folder)?;
    }
    Ok(folder)
}

/// `Box<dyn WallClock>` over a shared clock the writer keeps setting.
struct ClockHandle(std::sync::Arc<FixedClock>);

impl WallClock for ClockHandle {
    fn now(&self) -> Zoned {
        self.0.now()
    }
}

fn clean(folder: &GamesFolder, sessions_folder: &Path) -> Result<(), Box<dyn Error>> {
    for game in folder.games()? {
        if game.name.starts_with(FIXTURE_PREFIX) {
            folder.delete(game.id)?;
            let dir = sessions_folder.join(panowww_core::game::name::sanitize_stem(&game.name));
            if dir.exists() {
                std::fs::remove_dir_all(&dir)?;
            }
            println!("removed {}", game.name);
        }
    }
    let damaged = folder.dir().join(DAMAGED_FILE);
    if damaged.exists() {
        std::fs::remove_file(&damaged)?;
        println!("removed {DAMAGED_FILE}");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;
    let paths = AppPaths::new(args.data.clone());
    paths.ensure()?;
    let settings = Settings::load_or_default(&paths);
    let sessions_folder = args.sessions.unwrap_or(settings.sessions_folder);
    let folder = GamesFolder::new(&paths);
    println!("Games: {}", folder.dir().display());
    println!("Sessions: {}", sessions_folder.display());

    if args.clean {
        clean(&folder, &sessions_folder)?;
    }

    let mut targets: Vec<Game> = Vec::new();
    if !args.no_games {
        let existing = folder.games()?;
        for game in fixture_games() {
            if let Some(found) = existing.iter().find(|g| g.name == game.name) {
                targets.push(found.clone());
            } else {
                folder.save(&game)?;
                println!("created {}", game.name);
                targets.push(game);
            }
        }
        let damaged = folder.dir().join(DAMAGED_FILE);
        if !damaged.exists() {
            std::fs::write(
                &damaged,
                r#"{"format": "panowww-game", "version": 1, "id": "not-a-uuid", "name": "#,
            )?;
            println!("created {DAMAGED_FILE}");
        }
    }
    for name in &args.games {
        let game = folder
            .games()?
            .into_iter()
            .find(|g| &g.name == name)
            .ok_or_else(|| format!("no Game named {name:?}"))?;
        targets.push(game);
    }

    let now = Zoned::now();
    for game in &targets {
        for case in cases(&now)? {
            let written = write_session(&sessions_folder, game, &case)?;
            println!("  {}", written.display());
        }
        println!("{}: 8 Sessions", game.name);
    }
    Ok(())
}
