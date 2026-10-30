//! `session.json` (spec §6): what one Session folder records about its
//! run. Written as `running` at the first Shot, rewritten at the end.

use std::path::{Path, PathBuf};

use panowww_core::game::Game;
use panowww_core::ports::Zone;
use panowww_core::runner::RunError;
use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::StoreError;
use crate::json;

/// Value of the `format` field.
pub const FORMAT: &str = "panowww-session";
/// Schema version this build writes.
pub const CURRENT_VERSION: u32 = 1;
/// File name inside the Session folder.
pub const FILE: &str = "session.json";
/// Folder name of a Session: local time, sortable, legal on both file
/// systems (`2026-10-30_15-04-05`).
pub const FOLDER_TIMESTAMP: &str = "%Y-%m-%d_%H-%M-%S";
/// `startedAt` / `endedAt` / `takenAt`: RFC 3339 with the local offset.
pub const TIMESTAMP: &str = "%Y-%m-%dT%H:%M:%S%:z";

/// The single accepted value of the `format` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SessionFormat {
    #[default]
    #[serde(rename = "panowww-session")]
    PanowwwSession,
}

/// The end states of §7.3, plus `running` while the Session goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SessionStatus {
    Running,
    Completed,
    Interrupted,
    Failed,
}

/// One Shot of the folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ShotEntry {
    /// `rNN_cNN.png`
    pub file: String,
    pub row: u32,
    pub col: u32,
    pub taken_at: String,
    pub capture_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct SessionRecord {
    pub format: SessionFormat,
    pub version: u32,
    pub id: Uuid,
    pub app_version: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ended_at: Option<String>,
    pub status: SessionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<RunError>,
    /// The Game as it was when the Session started.
    pub game: Game,
    pub zone: Zone,
    pub shots_expected: u32,
    pub shots_taken: u32,
    pub shots: Vec<ShotEntry>,
}

impl SessionRecord {
    /// Reads `<folder>/session.json`.
    pub fn read(folder: &Path) -> Result<Self, StoreError> {
        let path = Self::path(folder);
        let text = std::fs::read_to_string(&path).map_err(|e| StoreError::io("read", &path, e))?;
        match json::check_envelope(&path, &text, FORMAT, u64::from(CURRENT_VERSION))? {
            1 => json::parse(&path, &text),
            other => Err(StoreError::Json {
                path,
                source: serde_json::Error::custom(format!("unknown format version {other}")),
            }),
        }
    }

    /// Writes `<folder>/session.json` atomically.
    pub fn write(&self, folder: &Path) -> Result<(), StoreError> {
        json::write_pretty(&Self::path(folder), self)
    }

    pub fn path(folder: &Path) -> PathBuf {
        folder.join(FILE)
    }

    /// `startedAt` as an instant, for sorting. `None` for a hand-edited
    /// value that does not parse.
    pub fn started_at(&self) -> Option<jiff::Timestamp> {
        self.started_at.parse().ok()
    }
}

/// `rNN_cNN.png`: two digits, 1-based, row then column.
pub fn shot_file_name(row: u32, col: u32) -> String {
    format!("r{row:02}_c{col:02}.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_file_names_are_two_digit_row_then_column() {
        assert_eq!(shot_file_name(1, 1), "r01_c01.png");
        assert_eq!(shot_file_name(6, 13), "r06_c13.png");
        assert_eq!(shot_file_name(36, 72), "r36_c72.png");
    }

    #[test]
    fn timestamps_sort_by_instant_whatever_the_offset() {
        let a: jiff::Timestamp = "2026-10-30T15:04:05+01:00".parse().unwrap();
        let b: jiff::Timestamp = "2026-10-30T14:04:06Z".parse().unwrap();
        assert!(b > a);
    }
}
