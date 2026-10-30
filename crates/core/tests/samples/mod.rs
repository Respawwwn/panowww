//! One valid sample Game per Input device, with fixed ids so their JSON is
//! stable for snapshots.

use panowww_core::game::{
    CURRENT_VERSION, DragButton, Game, GameFormat, Input, MouseMode, PhysicalKey, StartPole,
};
use uuid::Uuid;

fn base(id: u128, name: &str, input: Input) -> Game {
    Game {
        format: GameFormat::PanowwwGame,
        version: CURRENT_VERSION,
        id: Uuid::from_u128(id),
        name: name.to_owned(),
        notes: None,
        start_pole: StartPole::Nadir,
        shots_per_row: 13,
        rows: 6,
        wait_after_move_ms: 300,
        wait_after_shot_ms: 300,
        input,
    }
}

pub fn keyboard() -> Game {
    base(
        0x3f6c_1e2a_0000_4000_8000_0000_0000_0001,
        "Outer Wilds – keyboard",
        Input::Keyboard {
            turn_right_key: PhysicalKey::new("KeyD"),
            look_up_key: PhysicalKey::new("ArrowUp"),
            look_down_key: PhysicalKey::new("ArrowDown"),
            turn_hold_ms: 120,
            pitch_hold_ms: 90,
        },
    )
}

pub fn mouse() -> Game {
    let mut game = base(
        0x3f6c_1e2a_0000_4000_8000_0000_0000_0002,
        "Elden Ring – photo mode",
        Input::Mouse {
            mode: MouseMode::Drag,
            drag_button: Some(DragButton::Right),
            turn_delta: 840,
            pitch_delta: 610,
            invert_vertical: false,
        },
    );
    game.notes = Some("FOV 90, HUD off".to_owned());
    game
}

pub fn gamepad() -> Game {
    let mut game = base(
        0x3f6c_1e2a_0000_4000_8000_0000_0000_0003,
        "Forza Horizon 5 – gamepad",
        Input::Gamepad {
            stick_deflection: 0.8,
            turn_hold_ms: 100,
            pitch_hold_ms: 100,
            invert_vertical: true,
        },
    );
    game.start_pole = StartPole::Zenith;
    game.shots_per_row = 16;
    game.rows = 8;
    game
}

pub fn all() -> [(&'static str, Game); 3] {
    [
        ("keyboard", keyboard()),
        ("mouse", mouse()),
        ("gamepad", gamepad()),
    ]
}
