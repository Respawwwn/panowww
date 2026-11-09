//! The payloads crossing the command boundary that no library crate
//! defines. Every type is exported to `ui/src/lib/generated/`.

use panowww_core::plan::{Axis, RunKind};
use panowww_core::ports::{DisplayInfo, Rect, Zone};
use panowww_core::runner::RunOutcome;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum AxisDto {
    Turn,
    Pitch,
}

impl From<AxisDto> for Axis {
    fn from(axis: AxisDto) -> Self {
        match axis {
            AxisDto::Turn => Self::Turn,
            AxisDto::Pitch => Self::Pitch,
        }
    }
}

/// Which run the Start card or the Calibrate card asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum RunKindDto {
    Session,
    TurnTest,
    PitchTest,
    StepOnce { axis: AxisDto },
    CheckOverlap { axis: AxisDto },
}

impl From<RunKindDto> for RunKind {
    fn from(kind: RunKindDto) -> Self {
        match kind {
            RunKindDto::Session => Self::Session,
            RunKindDto::TurnTest => Self::TurnTest,
            RunKindDto::PitchTest => Self::PitchTest,
            RunKindDto::StepOnce { axis } => Self::StepOnce(axis.into()),
            RunKindDto::CheckOverlap { axis } => Self::CheckOverlap(axis.into()),
        }
    }
}

impl RunKindDto {
    pub fn is_session(self) -> bool {
        matches!(self, Self::Session)
    }
}

/// Why a countdown could not start (spec §4.5, §11.5 error state).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "reason", rename_all = "camelCase")]
#[ts(export)]
pub enum StartFailure {
    Accessibility,
    ScreenRecording,
    Vigem,
    #[serde(rename_all = "camelCase")]
    DisplayGone {
        display: String,
    },
    #[serde(rename_all = "camelCase")]
    CaptureUnavailable {
        message: String,
    },
    /// No Capture zone on this machine, or a damaged Game: the UI never
    /// asks in that state, so this is a guard.
    NoZone,
}

/// What a run streams to the UI as the `run` event (spec §4.4), from the
/// countdown to the end. Images travel as `data:` URLs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "event", rename_all = "camelCase")]
#[ts(export)]
pub enum RunEvent {
    /// The pre-flight or the stream failed: nothing was created.
    CannotStart(StartFailure),
    #[serde(rename_all = "camelCase")]
    Countdown {
        seconds_left: u32,
    },
    /// Cancelled during the countdown: nothing was created.
    Cancelled,
    Started {
        expected: u32,
    },
    #[serde(rename_all = "camelCase")]
    Shot {
        index: u32,
        row: u32,
        col: u32,
        capture_ms: u32,
        thumbnail: Option<String>,
    },
    Moved {
        row: u32,
        col: u32,
    },
    /// A Sample of Check overlap, full size.
    Sample {
        slot: SampleSlotDto,
        image: String,
    },
    ScreenUnchanged,
    #[serde(rename_all = "camelCase")]
    Preview {
        image: String,
        display_w: u32,
        display_h: u32,
        zone: Zone,
    },
    Finished {
        outcome: RunOutcome,
        folder: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SampleSlotDto {
    First,
    Second,
}

/// What the overlay page draws (spec §10): the frame always, plus one of
/// the digit, the progress bar with its pill, or the end pill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum OverlayView {
    Frame,
    #[serde(rename_all = "camelCase")]
    Countdown {
        seconds_left: u32,
    },
    Progress {
        taken: u32,
        expected: u32,
    },
    End {
        stopped: bool,
        taken: u32,
        expected: u32,
    },
}

/// The zone in CSS points, relative to the overlay window that covers
/// the display.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OverlayZone {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl From<&Zone> for OverlayZone {
    /// Zones are points on macOS and physical pixels on Windows (spec §6).
    fn from(zone: &Zone) -> Self {
        let per_point = if cfg!(target_os = "macos") {
            1.0
        } else {
            zone.scale.max(f64::EPSILON)
        };
        Self {
            x: f64::from(zone.x) / per_point,
            y: f64::from(zone.y) / per_point,
            w: f64::from(zone.w) / per_point,
            h: f64::from(zone.h) / per_point,
        }
    }
}

/// One message to the overlay page: everything it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OverlayState {
    pub zone: OverlayZone,
    pub view: OverlayView,
}

/// What one picker window shows (spec §11.4): its display, the dimmed
/// picture of it and the zone already drawn there, if any.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PickerSetup {
    pub display: DisplayInfo,
    /// 1-based position among the connected displays, for `Display {n}`.
    pub display_index: u32,
    /// The Game's zone when it sits on this display, in display points.
    pub zone: Option<Rect>,
    /// The app's appearance, so the bottom card matches the main window.
    pub appearance: panowww_store::settings::Appearance,
}

/// What the Game Setup shows about the zone of a Game on this machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZoneState {
    pub zone: Option<Zone>,
    pub status: Option<panowww_store::local::ZoneStatus>,
    /// 1-based position of the zone's display among the connected ones,
    /// for `Display {n}`.
    pub display_index: Option<u32>,
}

/// The Start card line (spec §7.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Estimate {
    pub duration_ms: u64,
    /// `≈ 4 min 20 s`
    pub duration: String,
    pub shots: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Permissions {
    pub accessibility: bool,
    pub screen_recording: bool,
    /// Windows only; always true elsewhere.
    pub vigem: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PermissionKind {
    Accessibility,
    ScreenRecording,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// What Settings shows under a Stitcher's row (spec §11.12): the program
/// is still there, it really is that Stitcher (its product name says so),
/// and its version when it can be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StitcherCheck {
    pub installed: bool,
    pub genuine: bool,
    /// The product name the program carries, whatever it is.
    pub product: Option<String>,
    pub version: Option<String>,
}

/// The Runner's phase, for the UI and the tests (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunnerPhase {
    Idle,
    Countdown,
    Running,
    Stopping,
}
