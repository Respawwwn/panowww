//! The ports `core` consumes, implemented by the `input`, `capture` and
//! `store` crates (spec §4.2). Adapters are deliberately dumb: they perform
//! one primitive per call and never wait; `core` decides every sequence
//! (ADR 0001).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

pub use crate::game::DragButton as MouseButton;
use crate::game::PhysicalKey;
use crate::runner::{RunOutcome, SessionStart, ShotRecord};

/// A position in display coordinates: points on macOS, pixels on Windows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A rectangle in display coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    /// The middle of the rectangle, where a drag Step parks the cursor.
    pub fn centre(&self) -> Point {
        Point {
            x: f64::from(self.x) + f64::from(self.w) / 2.0,
            y: f64::from(self.y) + f64::from(self.h) / 2.0,
        }
    }
}

/// The OS's stable identifier of a display.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DisplayId(pub String);

/// The Capture zone of a Game on this machine (spec §6): the rectangle, and
/// the display it was drawn on so a changed display is detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Zone {
    pub display_id: DisplayId,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub display_w: u32,
    pub display_h: u32,
    pub scale: f64,
}

impl Zone {
    /// The captured rectangle.
    pub fn rect(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }
}

/// An RGB8 image without alpha, rows top to bottom, `width × height × 3`
/// bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

impl Frame {
    /// A frame of one colour.
    pub fn solid(width: u32, height: u32, rgb: [u8; 3]) -> Self {
        let pixels = (width as usize) * (height as usize);
        Self {
            width,
            height,
            rgb: rgb.repeat(pixels),
        }
    }

    /// Whether the buffer holds exactly `width × height` RGB pixels.
    pub fn is_well_formed(&self) -> bool {
        (self.width as usize) * (self.height as usize) * 3 == self.rgb.len()
    }
}

/// Encoded JPEG bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jpeg(pub Vec<u8>);

/// One Display preview image from the capture adapter (spec §9): the whole
/// display reduced, with the display size and the zone so the UI draws the
/// zone frame over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewFrame {
    pub jpeg: Jpeg,
    pub display_w: u32,
    pub display_h: u32,
    pub zone: Rect,
}

/// Where the capture adapter sends preview frames, from its own thread.
pub type PreviewSink = Box<dyn Fn(PreviewFrame) + Send + 'static>;

/// Why an input primitive failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InputError {
    /// macOS Accessibility not granted, or Windows UIPI blocking us.
    #[error("Panowww is not allowed to control the keyboard and mouse.")]
    PermissionDenied,
    /// The virtual gamepad driver is missing (ViGEmBus).
    #[error("The virtual gamepad driver is not installed.")]
    DeviceMissing,
    #[error("{0}")]
    Failed(String),
}

/// Why a capture primitive failed. Maps onto the `error.kind` set of
/// `session.json`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CaptureError {
    #[error("Screen capture is unavailable: {0}")]
    Unavailable(String),
    #[error("The display of the Capture zone is gone.")]
    DisplayGone,
    #[error("Panowww is no longer allowed to record the screen.")]
    PermissionRevoked,
    /// This one Shot or Sample failed; the Runner retries once.
    #[error("A Shot failed: {0}")]
    ShotFailed(String),
}

/// Why the store could not take a Session or a Shot.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StoreError {
    #[error("{0}")]
    Failed(String),
}

/// Input device primitives. Waits never happen here: `core` uses [`Clock`].
pub trait Camera {
    fn key_down(&mut self, key: PhysicalKey) -> Result<(), InputError>;
    fn key_up(&mut self, key: PhysicalKey) -> Result<(), InputError>;
    /// One relative sub-move.
    fn mouse_delta(&mut self, dx: i32, dy: i32) -> Result<(), InputError>;
    fn button_down(&mut self, button: MouseButton) -> Result<(), InputError>;
    fn button_up(&mut self, button: MouseButton) -> Result<(), InputError>;
    /// Absolute move, once per drag Step.
    fn park_cursor(&mut self, at: Point) -> Result<(), InputError>;
    /// Right stick, each axis in −1..1. Windows only.
    fn stick(&mut self, x: f64, y: f64) -> Result<(), InputError>;
    /// Releases every held key, button and stick. Called first on every stop.
    fn release_all(&mut self) -> Result<(), InputError>;
}

/// An open capture of one display, cropped to the Capture zone for Shots.
pub trait Stream {
    /// A fresh frame whose timestamp is after `not_before`, cropped to the
    /// zone.
    fn shot(&mut self, not_before: Instant) -> Result<Frame, CaptureError>;
    /// Turns the Display preview source on or off, immediately.
    fn set_preview(&mut self, on: bool);
    fn stop(self: Box<Self>);
}

/// Opens a [`Stream`]. Called by `app` before the countdown, never by
/// `core` (spec §4.5).
pub trait Grabber {
    fn start(
        &mut self,
        display: DisplayId,
        zone: Rect,
        preview: PreviewSink,
    ) -> Result<Box<dyn Stream>, CaptureError>;
}

/// Where a Session's Shots go. Only Sessions call it.
pub trait ShotSink {
    /// Creates the Session folder and writes `session.json` as running.
    fn begin(&mut self, start: &SessionStart) -> Result<(), StoreError>;
    /// Queues the PNG; the Runner never encodes.
    fn shot(&mut self, record: ShotRecord, frame: Frame) -> Result<(), StoreError>;
    /// Flushes and rewrites `session.json` with the outcome.
    fn finish(&mut self, outcome: &RunOutcome) -> Result<(), StoreError>;
}

/// Time, so the Runner is testable without sleeping.
pub trait Clock {
    fn now(&self) -> Instant;
    /// Sleeps `duration`, or returns early with `false` when `cancel` is
    /// raised.
    fn wait(&self, duration: Duration, cancel: &CancelFlag) -> bool;
}

/// Shared flag raised to stop a run. Cloning shares the flag.
#[derive(Debug, Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raise(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_raised(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Lowers the flag so the same handle serves the next run.
    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// The real clock: sleeps in short slices so a cancel is seen promptly.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    /// Longest uninterrupted sleep, so a stop lands within this delay.
    pub const SLICE: Duration = Duration::from_millis(5);
}

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn wait(&self, duration: Duration, cancel: &CancelFlag) -> bool {
        let end = Instant::now() + duration;
        loop {
            if cancel.is_raised() {
                return false;
            }
            let now = Instant::now();
            if now >= end {
                return true;
            }
            std::thread::sleep((end - now).min(Self::SLICE));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_centre_is_its_middle() {
        let rect = Rect {
            x: 10,
            y: 20,
            w: 100,
            h: 51,
        };
        assert_eq!(rect.centre(), Point { x: 60.0, y: 45.5 });
        let zone = Zone {
            display_id: DisplayId("main".to_owned()),
            x: 10,
            y: 20,
            w: 100,
            h: 51,
            display_w: 1920,
            display_h: 1080,
            scale: 2.0,
        };
        assert_eq!(zone.rect(), rect);
    }

    #[test]
    fn solid_frames_are_well_formed() {
        let frame = Frame::solid(3, 2, [1, 2, 3]);
        assert!(frame.is_well_formed());
        assert_eq!(
            frame.rgb,
            [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3]
        );
        assert!(
            !Frame {
                width: 2,
                height: 2,
                rgb: vec![0; 11]
            }
            .is_well_formed()
        );
    }

    #[test]
    fn cancel_flag_is_shared_between_clones() {
        let flag = CancelFlag::new();
        let other = flag.clone();
        assert!(!other.is_raised());
        flag.raise();
        assert!(other.is_raised());
        other.reset();
        assert!(!flag.is_raised());
    }

    #[test]
    fn system_clock_waits_and_stops_on_cancel() {
        let clock = SystemClock;
        let cancel = CancelFlag::new();
        let start = clock.now();
        assert!(clock.wait(Duration::from_millis(12), &cancel));
        assert!(clock.now() - start >= Duration::from_millis(12));

        cancel.raise();
        let start = clock.now();
        assert!(!clock.wait(Duration::from_secs(5), &cancel));
        assert!(clock.now() - start < Duration::from_secs(1));
    }

    #[test]
    fn zone_serialises_with_the_local_games_keys() {
        let zone = Zone {
            display_id: DisplayId("69734272".to_owned()),
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
            display_w: 2560,
            display_h: 1440,
            scale: 2.0,
        };
        let json = serde_json::to_string(&zone).unwrap();
        assert_eq!(
            json,
            r#"{"displayId":"69734272","x":0,"y":0,"w":1920,"h":1080,"displayW":2560,"displayH":1440,"scale":2.0}"#
        );
    }
}
