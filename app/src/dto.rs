//! The payloads crossing the command boundary that no library crate
//! defines. Every type is exported to `ui/src/lib/generated/`.

use panowww_core::plan::{Axis, RunKind};
use panowww_core::ports::Zone;
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
