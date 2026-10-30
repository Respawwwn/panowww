//! One Step and the device motion that performs it, both computed from the
//! Game. The Runner expands a [`Motion`] into primitives at run time: key
//! down and up, mouse sub-moves, button down and up, stick position.

use std::fmt;
use std::time::Duration;

use crate::constants::{DRAG_SETTLE, MOUSE_SUB_MOVE_INTERVAL, MOUSE_SUB_MOVE_MAX};
use crate::game::{DragButton, Game, Input, MouseMode, PhysicalKey, StartPole};

/// The two camera axes. A turn Step always rotates to the right; a pitch
/// Step moves from the start pole towards the opposite one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    Turn,
    Pitch,
}

/// One camera movement between two Shots.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub axis: Axis,
    pub motion: Motion,
}

/// The device-specific way to perform a Step. Direction and `invertVertical`
/// are already folded into the key choice or the signs.
#[derive(Debug, Clone, PartialEq)]
pub enum Motion {
    /// Press `key`, hold, release.
    KeyHold { key: PhysicalKey, hold: Duration },
    /// Relative mouse motion, performed as [`MouseDelta::sub_moves`].
    MouseMove(MouseDelta),
    /// Park the cursor at the Capture zone centre, press `button`, settle
    /// [`DRAG_SETTLE`], perform the sub-moves, settle again, release.
    MouseDrag {
        button: DragButton,
        delta: MouseDelta,
    },
    /// Right stick to (`x`, `y`), hold, back to centre. XInput convention:
    /// positive `x` is right, positive `y` is up.
    StickHold { x: f64, y: f64, hold: Duration },
}

/// Signed relative mouse motion in raw units. Screen convention: positive
/// `dx` is right, positive `dy` is down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseDelta {
    pub dx: i32,
    pub dy: i32,
}

/// Where a pitch Step looks, from the start pole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vertical {
    Up,
    Down,
}

impl Vertical {
    fn from_start_pole(pole: StartPole) -> Self {
        match pole {
            StartPole::Nadir => Self::Up,
            StartPole::Zenith => Self::Down,
        }
    }

    fn flipped_if(self, invert: bool) -> Self {
        match (self, invert) {
            (v, false) => v,
            (Self::Up, true) => Self::Down,
            (Self::Down, true) => Self::Up,
        }
    }
}

impl Step {
    /// The Step of `game` on `axis`. Games are validated before planning; a
    /// drag block that still lacks its button drags with the right one.
    pub fn for_game(game: &Game, axis: Axis) -> Self {
        let looking = Vertical::from_start_pole(game.start_pole);
        let motion = match &game.input {
            Input::Keyboard {
                turn_right_key,
                look_up_key,
                look_down_key,
                turn_hold_ms,
                pitch_hold_ms,
            } => match axis {
                Axis::Turn => Motion::KeyHold {
                    key: turn_right_key.clone(),
                    hold: millis(*turn_hold_ms),
                },
                Axis::Pitch => Motion::KeyHold {
                    key: match looking {
                        Vertical::Up => look_up_key.clone(),
                        Vertical::Down => look_down_key.clone(),
                    },
                    hold: millis(*pitch_hold_ms),
                },
            },
            Input::Mouse {
                mode,
                drag_button,
                turn_delta,
                pitch_delta,
                invert_vertical,
            } => {
                let delta = match axis {
                    Axis::Turn => MouseDelta {
                        dx: signed(*turn_delta),
                        dy: 0,
                    },
                    // On screen, up is negative.
                    Axis::Pitch => MouseDelta {
                        dx: 0,
                        dy: match looking.flipped_if(*invert_vertical) {
                            Vertical::Up => -signed(*pitch_delta),
                            Vertical::Down => signed(*pitch_delta),
                        },
                    },
                };
                match mode {
                    MouseMode::Move => Motion::MouseMove(delta),
                    MouseMode::Drag => Motion::MouseDrag {
                        button: drag_button.unwrap_or(DragButton::Right),
                        delta,
                    },
                }
            }
            Input::Gamepad {
                stick_deflection,
                turn_hold_ms,
                pitch_hold_ms,
                invert_vertical,
            } => match axis {
                Axis::Turn => Motion::StickHold {
                    x: *stick_deflection,
                    y: 0.0,
                    hold: millis(*turn_hold_ms),
                },
                Axis::Pitch => Motion::StickHold {
                    x: 0.0,
                    y: match looking.flipped_if(*invert_vertical) {
                        Vertical::Up => *stick_deflection,
                        Vertical::Down => -*stick_deflection,
                    },
                    hold: millis(*pitch_hold_ms),
                },
            },
        };
        Self { axis, motion }
    }

    /// Time the Step takes, see [`Motion::duration`].
    pub fn duration(&self) -> Duration {
        self.motion.duration()
    }
}

impl Motion {
    /// `stepTime` of spec §7.6: the hold for keys and sticks; sub-moves times
    /// their interval for the mouse, plus both settles in drag mode.
    pub fn duration(&self) -> Duration {
        match self {
            Self::KeyHold { hold, .. } | Self::StickHold { hold, .. } => *hold,
            Self::MouseMove(delta) => delta.duration(),
            Self::MouseDrag { delta, .. } => delta.duration() + 2 * DRAG_SETTLE,
        }
    }
}

impl MouseDelta {
    /// How many sub-moves of at most [`MOUSE_SUB_MOVE_MAX`] units perform
    /// this delta: `ceil(delta / 20)` on the longer axis.
    pub fn sub_move_count(self) -> u32 {
        self.dx
            .unsigned_abs()
            .max(self.dy.unsigned_abs())
            .div_ceil(MOUSE_SUB_MOVE_MAX)
    }

    /// The delta split into [`Self::sub_move_count`] even sub-moves, each at
    /// most [`MOUSE_SUB_MOVE_MAX`] units per axis, summing to the whole.
    pub fn sub_moves(self) -> Vec<MouseDelta> {
        let count = i64::from(self.sub_move_count());
        (0..count)
            .map(|i| MouseDelta {
                dx: share(self.dx, i, count),
                dy: share(self.dy, i, count),
            })
            .collect()
    }

    /// Sub-moves times their interval.
    pub fn duration(self) -> Duration {
        self.sub_move_count() * MOUSE_SUB_MOVE_INTERVAL
    }
}

/// The `i`-th of `count` even integer shares of `total`.
fn share(total: i32, i: i64, count: i64) -> i32 {
    let total = i64::from(total);
    let part = total * (i + 1) / count - total * i / count;
    // |part| <= ceil(|total| / count) <= MOUSE_SUB_MOVE_MAX, so this fits.
    i32::try_from(part).unwrap_or(0)
}

fn millis(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

/// A Game delta (1–100000) as a signed amount.
fn signed(delta: u32) -> i32 {
    i32::try_from(delta).unwrap_or(i32::MAX)
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Turn => "turn",
            Self::Pitch => "pitch",
        })
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.axis, self.motion)
    }
}

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyHold { key, hold } => write!(f, "key {} hold {hold:?}", key.as_str()),
            Self::MouseMove(delta) => write!(f, "mouse {delta}"),
            Self::MouseDrag { button, delta } => write!(f, "drag {button:?} {delta}"),
            Self::StickHold { x, y, hold } => write!(f, "stick x {x} y {y} hold {hold:?}"),
        }
    }
}

impl fmt::Display for MouseDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dx {} dy {} in {} sub-moves",
            self.dx,
            self.dy,
            self.sub_move_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Device, NewGameDefaults};

    fn game_with(input: Input, start_pole: StartPole) -> Game {
        let mut game = Game::new("x", NewGameDefaults::default());
        game.input = input;
        game.start_pole = start_pole;
        game
    }

    fn mouse(mode: MouseMode, invert_vertical: bool) -> Input {
        Input::Mouse {
            mode,
            drag_button: Some(DragButton::Middle),
            turn_delta: 840,
            pitch_delta: 610,
            invert_vertical,
        }
    }

    fn gamepad(invert_vertical: bool) -> Input {
        Input::Gamepad {
            stick_deflection: 0.8,
            turn_hold_ms: 150,
            pitch_hold_ms: 250,
            invert_vertical,
        }
    }

    #[test]
    fn keyboard_turns_with_the_turn_key_and_pitches_from_the_pole() {
        let up = game_with(Input::defaults_for(Device::Keyboard), StartPole::Nadir);
        assert_eq!(
            Step::for_game(&up, Axis::Turn).motion,
            Motion::KeyHold {
                key: PhysicalKey::new("ArrowRight"),
                hold: Duration::from_millis(100)
            }
        );
        let Motion::KeyHold { key, .. } = Step::for_game(&up, Axis::Pitch).motion else {
            panic!()
        };
        assert_eq!(key.as_str(), "ArrowUp");

        let down = game_with(Input::defaults_for(Device::Keyboard), StartPole::Zenith);
        let Motion::KeyHold { key, .. } = Step::for_game(&down, Axis::Pitch).motion else {
            panic!()
        };
        assert_eq!(key.as_str(), "ArrowDown");
    }

    #[test]
    fn mouse_turns_right_and_pitches_by_pole_and_inversion() {
        let turn = Step::for_game(
            &game_with(mouse(MouseMode::Move, false), StartPole::Nadir),
            Axis::Turn,
        );
        assert_eq!(
            turn.motion,
            Motion::MouseMove(MouseDelta { dx: 840, dy: 0 })
        );

        for (pole, invert, dy) in [
            (StartPole::Nadir, false, -610),
            (StartPole::Nadir, true, 610),
            (StartPole::Zenith, false, 610),
            (StartPole::Zenith, true, -610),
        ] {
            let step = Step::for_game(
                &game_with(mouse(MouseMode::Move, invert), pole),
                Axis::Pitch,
            );
            assert_eq!(
                step.motion,
                Motion::MouseMove(MouseDelta { dx: 0, dy }),
                "{pole:?} invert={invert}"
            );
        }
    }

    #[test]
    fn drag_carries_the_button_and_falls_back_to_right() {
        let step = Step::for_game(
            &game_with(mouse(MouseMode::Drag, false), StartPole::Nadir),
            Axis::Turn,
        );
        assert_eq!(
            step.motion,
            Motion::MouseDrag {
                button: DragButton::Middle,
                delta: MouseDelta { dx: 840, dy: 0 }
            }
        );
        let Input::Mouse { drag_button, .. } = &mut mouse(MouseMode::Drag, false) else {
            panic!()
        };
        *drag_button = None;
        let buttonless = Input::Mouse {
            mode: MouseMode::Drag,
            drag_button: None,
            turn_delta: 1,
            pitch_delta: 1,
            invert_vertical: false,
        };
        let step = Step::for_game(&game_with(buttonless, StartPole::Nadir), Axis::Turn);
        assert!(matches!(
            step.motion,
            Motion::MouseDrag {
                button: DragButton::Right,
                ..
            }
        ));
    }

    #[test]
    fn stick_pushes_right_and_up_or_down() {
        let turn = Step::for_game(&game_with(gamepad(false), StartPole::Nadir), Axis::Turn);
        assert_eq!(
            turn.motion,
            Motion::StickHold {
                x: 0.8,
                y: 0.0,
                hold: Duration::from_millis(150)
            }
        );
        for (pole, invert, y) in [
            (StartPole::Nadir, false, 0.8),
            (StartPole::Nadir, true, -0.8),
            (StartPole::Zenith, false, -0.8),
            (StartPole::Zenith, true, 0.8),
        ] {
            let step = Step::for_game(&game_with(gamepad(invert), pole), Axis::Pitch);
            assert_eq!(
                step.motion,
                Motion::StickHold {
                    x: 0.0,
                    y,
                    hold: Duration::from_millis(250)
                },
                "{pole:?} invert={invert}"
            );
        }
    }

    #[test]
    fn sub_moves_are_ceil_delta_over_twenty_and_sum_to_the_delta() {
        for (delta, count) in [
            (1, 1),
            (20, 1),
            (21, 2),
            (50, 3),
            (840, 42),
            (-610, 31),
            (100_000, 5000),
        ] {
            let d = MouseDelta { dx: delta, dy: 0 };
            assert_eq!(d.sub_move_count(), count, "{delta}");
            let moves = d.sub_moves();
            assert_eq!(moves.len(), count as usize);
            assert_eq!(moves.iter().map(|m| m.dx).sum::<i32>(), delta);
            assert!(moves.iter().all(|m| m.dx.unsigned_abs() <= 20 && m.dy == 0));
        }
        assert_eq!(
            MouseDelta { dx: 50, dy: 0 }.sub_moves(),
            [
                MouseDelta { dx: 16, dy: 0 },
                MouseDelta { dx: 17, dy: 0 },
                MouseDelta { dx: 17, dy: 0 }
            ]
        );
        assert_eq!(MouseDelta { dx: 0, dy: 0 }.sub_moves(), []);
    }

    #[test]
    fn step_time_follows_the_spec_formula() {
        let hold = Duration::from_millis(120);
        assert_eq!(
            Motion::KeyHold {
                key: PhysicalKey::new("KeyD"),
                hold
            }
            .duration(),
            hold
        );
        assert_eq!(
            Motion::StickHold {
                x: 0.8,
                y: 0.0,
                hold
            }
            .duration(),
            hold
        );
        let delta = MouseDelta { dx: 840, dy: 0 };
        assert_eq!(
            Motion::MouseMove(delta).duration(),
            Duration::from_millis(42 * 8)
        );
        assert_eq!(
            Motion::MouseDrag {
                button: DragButton::Right,
                delta
            }
            .duration(),
            Duration::from_millis(42 * 8 + 40)
        );
    }

    #[test]
    fn display_is_one_readable_line() {
        let step = Step::for_game(
            &game_with(mouse(MouseMode::Drag, false), StartPole::Nadir),
            Axis::Pitch,
        );
        assert_eq!(
            step.to_string(),
            "pitch: drag Middle dx 0 dy -610 in 31 sub-moves"
        );
    }
}
