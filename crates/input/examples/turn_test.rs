//! The hands-on check of issues 9 and 23: runs a Turn test against the game
//! in front, through the real `Camera` of this OS, with fakes for everything
//! else.
//!
//! ```sh
//! cargo run -p panowww-input --example turn_test -- mouse drag right 840 13
//! cargo run -p panowww-input --example turn_test -- mouse move 840 13
//! cargo run -p panowww-input --example turn_test -- keyboard KeyD 120 13
//! cargo run -p panowww-input --example turn_test -- gamepad 0.8 120 13   # Windows
//! cargo run -p panowww-input --example turn_test -- check
//! ```
//!
//! Pick a landmark in the game, start the example, switch to the game
//! within 5 s. After the turn the camera must point at the landmark again.
//! Record the result in `docs/manual-test.md`.

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod shared {
    use std::time::Duration;

    use panowww_core::fakes::{FakeStream, MemorySink};
    use panowww_core::game::{DragButton, Game, Input, MouseMode, NewGameDefaults, PhysicalKey};
    use panowww_core::plan::{RunKind, plan};
    use panowww_core::ports::{Camera, CancelFlag, Rect, SystemClock};
    use panowww_core::runner::{Progress, RunRequest, run};

    /// The Game described by the command line, or the usage on stderr.
    pub fn game_from_args(args: &[String]) -> Game {
        let arg = |i: usize| args.get(i).map(String::as_str);
        let mut game = Game::new("turn_test", NewGameDefaults::default());
        game.input = match (arg(0), arg(1)) {
            (Some("keyboard"), Some(code)) => Input::Keyboard {
                turn_right_key: PhysicalKey::new(code),
                look_up_key: PhysicalKey::new("ArrowUp"),
                look_down_key: PhysicalKey::new("ArrowDown"),
                turn_hold_ms: arg(2).and_then(|s| s.parse().ok()).unwrap_or(120),
                pitch_hold_ms: 120,
            },
            (Some("mouse"), Some("drag")) => Input::Mouse {
                mode: MouseMode::Drag,
                drag_button: Some(match arg(2) {
                    Some("left") => DragButton::Left,
                    Some("middle") => DragButton::Middle,
                    _ => DragButton::Right,
                }),
                turn_delta: arg(3).and_then(|s| s.parse().ok()).unwrap_or(840),
                pitch_delta: 610,
                invert_vertical: false,
            },
            (Some("mouse"), Some("move")) => Input::Mouse {
                mode: MouseMode::Move,
                drag_button: None,
                turn_delta: arg(2).and_then(|s| s.parse().ok()).unwrap_or(840),
                pitch_delta: 610,
                invert_vertical: false,
            },
            (Some("gamepad"), _) => Input::Gamepad {
                stick_deflection: arg(1).and_then(|s| s.parse().ok()).unwrap_or(0.8),
                turn_hold_ms: arg(2).and_then(|s| s.parse().ok()).unwrap_or(120),
                pitch_hold_ms: 120,
                invert_vertical: false,
            },
            _ => {
                eprintln!(
                    "usage: turn_test (keyboard <code> [holdMs] | mouse drag [button] [delta] | mouse move [delta] | gamepad [deflection] [holdMs]) [shotsPerRow] | check"
                );
                std::process::exit(2);
            }
        };
        let shots_per_row: u32 = args.last().and_then(|s| s.parse().ok()).unwrap_or(13);
        game.shots_per_row = shots_per_row.clamp(3, 72);
        game.wait_after_move_ms = 300;
        game
    }

    /// Counts down, then runs the Turn test of `game` through `camera`; the
    /// drag Step parks the cursor at the centre of `zone`.
    pub fn turn(camera: &mut dyn Camera, game: &Game, zone: Rect) {
        let plan = plan(game, RunKind::TurnTest);
        println!("{plan}");
        for n in (1..=5).rev() {
            println!("{n}… switch to the game");
            std::thread::sleep(Duration::from_secs(1));
        }
        let request = RunRequest {
            plan: &plan,
            zone,
            session: None,
        };
        let mut stream = FakeStream::new(2, 2).moving();
        let mut sink = MemorySink::new();
        let outcome = run(
            &request,
            camera,
            &mut stream,
            &mut sink,
            &SystemClock,
            &CancelFlag::new(),
            &|event| match event {
                Progress::Moved { col, .. } => println!("step {col}"),
                Progress::Finished(outcome) => println!("{outcome:?}"),
                _ => {}
            },
        );
        println!("done: {outcome:?}. Back on the landmark?");
    }
}

#[cfg(target_os = "macos")]
fn main() {
    use panowww_core::ports::Rect;
    use panowww_input::macos::{CgCamera, accessibility};

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("check") {
        println!("accessibility granted: {}", accessibility::check());
        if !accessibility::check() {
            println!("requesting: {}", accessibility::request());
            println!(
                "opening System Settings: {:?}",
                accessibility::open_settings()
            );
        }
        return;
    }
    let game = shared::game_from_args(&args);
    if !accessibility::check() {
        eprintln!("Accessibility is not granted: run `turn_test check` first.");
        std::process::exit(1);
    }
    // Park the cursor near the middle of a 1440p main display; adjust to
    // your Capture zone when drag parking matters.
    let zone = Rect {
        x: 320,
        y: 180,
        w: 1920,
        h: 1080,
    };
    let mut camera = CgCamera::new();
    shared::turn(&mut camera, &game, zone);
}

#[cfg(target_os = "windows")]
fn main() {
    use panowww_core::ports::Rect;
    use panowww_input::windows::{SendInputCamera, elevation, gamepad};

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("check") {
        println!("elevated: {}", elevation::is_elevated());
        match gamepad::check() {
            Ok(()) => println!("ViGEmBus: installed"),
            Err(e) => println!("ViGEmBus: {e}"),
        }
        return;
    }
    let game = shared::game_from_args(&args);
    // Park the cursor at the middle of a 1080p primary display; adjust to
    // your Capture zone when drag parking matters.
    let zone = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
    };
    let mut camera = SendInputCamera::new();
    shared::turn(&mut camera, &game, zone);
    println!("virtual pad plugged: {}", camera.pad_plugged());
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() {
    eprintln!("turn_test needs macOS or Windows.");
}
