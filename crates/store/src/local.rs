//! `local-games.json` (spec §6): what belongs to this machine and never
//! travels with a Game, keyed by Game id. Today that is the Capture zone,
//! with the display it was drawn on so a changed display is noticed.

use std::collections::BTreeMap;
use std::path::Path;

use panowww_core::ports::{DisplayInfo, Zone};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::StoreError;
use crate::json::{self, Loaded};
use crate::paths::AppPaths;

/// Value of the `format` field.
pub const FORMAT: &str = "panowww-local-games";
/// Schema version this build writes.
pub const CURRENT_VERSION: u32 = 1;

/// The single accepted value of the `format` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LocalGamesFormat {
    #[default]
    #[serde(rename = "panowww-local-games")]
    PanowwwLocalGames,
}

/// The machine-local state of one Game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct LocalGame {
    pub zone: Zone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct LocalGames {
    pub format: LocalGamesFormat,
    pub version: u32,
    pub games: BTreeMap<Uuid, LocalGame>,
    /// The sidebar's order, as the user dragged it; Games not listed come
    /// after, by name. Machine-local like the zone: never in a Game file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<Uuid>,
}

impl Default for LocalGames {
    fn default() -> Self {
        Self {
            format: LocalGamesFormat::PanowwwLocalGames,
            version: CURRENT_VERSION,
            games: BTreeMap::new(),
            order: Vec::new(),
        }
    }
}

/// Version 0: the pre-release file, the map possibly missing, unknown keys
/// ignored.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalGamesV0 {
    games: Option<BTreeMap<Uuid, LocalGame>>,
}

/// Whether a stored zone can still be used (spec §6): never rescaled, the
/// user redraws it when its display changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ZoneStatus {
    Ready,
    /// The display is connected but its size or scale changed.
    DisplayChanged,
    /// No connected display has this id.
    DisplayGone,
}

impl LocalGames {
    /// Reads `local-games.json`. A missing file is an empty map; an older
    /// file is migrated and written back.
    pub fn load(paths: &AppPaths) -> Result<Self, StoreError> {
        let path = paths.local_games_file();
        let Some(text) = json::read_optional(&path)? else {
            return Ok(Self::default());
        };
        let loaded = read(&path, &text)?;
        if loaded.migrated {
            loaded.value.save(paths)?;
        }
        Ok(loaded.value)
    }

    pub fn save(&self, paths: &AppPaths) -> Result<(), StoreError> {
        json::write_pretty(&paths.local_games_file(), self)
    }

    pub fn zone(&self, id: Uuid) -> Option<&Zone> {
        self.games.get(&id).map(|g| &g.zone)
    }

    pub fn set_zone(&mut self, id: Uuid, zone: Zone) {
        self.games.insert(id, LocalGame { zone });
    }

    /// Forgets everything about `id`, for a deleted Game.
    pub fn remove(&mut self, id: Uuid) {
        self.games.remove(&id);
        self.order.retain(|g| *g != id);
    }

    /// The sidebar's order, from the first row down.
    pub fn set_order(&mut self, ids: Vec<Uuid>) {
        self.order = ids;
    }

    /// Where `id` sits in the order; `None` puts it after the ordered ones.
    pub fn position(&self, id: Uuid) -> Option<usize> {
        self.order.iter().position(|g| *g == id)
    }
}

/// How `zone` relates to the displays connected now.
pub fn zone_status(zone: &Zone, displays: &[DisplayInfo]) -> ZoneStatus {
    let Some(display) = displays.iter().find(|d| d.id == zone.display_id) else {
        return ZoneStatus::DisplayGone;
    };
    let same_size = display.width == zone.display_w && display.height == zone.display_h;
    let same_scale = (display.scale - zone.scale).abs() < 1e-6;
    if same_size && same_scale {
        ZoneStatus::Ready
    } else {
        ZoneStatus::DisplayChanged
    }
}

fn read(path: &Path, text: &str) -> Result<Loaded<LocalGames>, StoreError> {
    let version = json::check_envelope(path, text, FORMAT, u64::from(CURRENT_VERSION))?;
    if version == 1 {
        Ok(Loaded {
            value: json::parse(path, text)?,
            migrated: false,
        })
    } else {
        let v0: LocalGamesV0 = json::parse(path, text)?;
        Ok(Loaded {
            value: LocalGames {
                games: v0.games.unwrap_or_default(),
                ..LocalGames::default()
            },
            migrated: true,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_order_follows_the_user_and_forgets_deleted_games() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut local = LocalGames::default();
        assert_eq!(local.position(a), None);
        local.set_order(vec![b, a]);
        assert_eq!(local.position(a), Some(1));
        local.remove(b);
        assert_eq!(local.position(a), Some(0));
        assert_eq!(local.position(b), None);
    }

    use panowww_core::ports::DisplayId;

    use super::*;

    fn zone() -> Zone {
        Zone {
            display_id: DisplayId("69734272".to_owned()),
            x: 100,
            y: 50,
            w: 1600,
            h: 900,
            display_w: 2560,
            display_h: 1440,
            scale: 2.0,
        }
    }

    fn display(id: &str, width: u32, height: u32, scale: f64) -> DisplayInfo {
        DisplayInfo {
            id: DisplayId(id.to_owned()),
            width,
            height,
            scale,
            hdr: false,
        }
    }

    #[test]
    fn missing_file_is_empty_and_round_trips_after_save() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path());
        let mut local = LocalGames::load(&paths).unwrap();
        assert!(local.games.is_empty());
        assert!(
            !paths.local_games_file().exists(),
            "nothing written for an empty map"
        );
        let id = Uuid::new_v4();
        local.set_zone(id, zone());
        local.save(&paths).unwrap();
        let again = LocalGames::load(&paths).unwrap();
        assert_eq!(again.zone(id), Some(&zone()));
        assert_eq!(again, local);
        let text = std::fs::read_to_string(paths.local_games_file()).unwrap();
        assert!(text.contains(r#""format": "panowww-local-games""#));
        assert!(text.contains(r#""displayId": "69734272""#), "{text}");
    }

    #[test]
    fn remove_forgets_the_game() {
        let mut local = LocalGames::default();
        let id = Uuid::new_v4();
        local.set_zone(id, zone());
        local.remove(id);
        assert_eq!(local.zone(id), None);
    }

    #[test]
    fn version_zero_is_migrated() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path());
        let id = Uuid::new_v4();
        std::fs::write(
            paths.local_games_file(),
            format!(
                r#"{{"format":"panowww-local-games","version":0,"legacy":true,"games":{{"{id}":{{"zone":{}}}}}}}"#,
                serde_json::to_string(&zone()).unwrap()
            ),
        )
        .unwrap();
        let local = LocalGames::load(&paths).unwrap();
        assert_eq!(local.version, 1);
        assert_eq!(local.zone(id), Some(&zone()));
        assert!(
            std::fs::read_to_string(paths.local_games_file())
                .unwrap()
                .contains(r#""version": 1"#)
        );
    }

    #[test]
    fn newer_or_foreign_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path());
        std::fs::write(
            paths.local_games_file(),
            r#"{"format":"panowww-local-games","version":2}"#,
        )
        .unwrap();
        assert!(matches!(
            LocalGames::load(&paths),
            Err(StoreError::NewerVersion { version: 2, .. })
        ));
        std::fs::write(
            paths.local_games_file(),
            r#"{"format":"panowww-settings","version":1}"#,
        )
        .unwrap();
        assert!(matches!(
            LocalGames::load(&paths),
            Err(StoreError::WrongFormat { .. })
        ));
    }

    #[test]
    fn zone_status_follows_the_display_list() {
        let z = zone();
        let same = display("69734272", 2560, 1440, 2.0);
        let other = display("1", 1920, 1080, 1.0);
        assert_eq!(
            zone_status(&z, &[other.clone(), same.clone()]),
            ZoneStatus::Ready
        );
        assert_eq!(
            zone_status(&z, std::slice::from_ref(&other)),
            ZoneStatus::DisplayGone
        );
        assert_eq!(zone_status(&z, &[]), ZoneStatus::DisplayGone);
        assert_eq!(
            zone_status(&z, &[display("69734272", 1920, 1080, 2.0)]),
            ZoneStatus::DisplayChanged
        );
        assert_eq!(
            zone_status(&z, &[display("69734272", 2560, 1440, 1.0)]),
            ZoneStatus::DisplayChanged
        );
    }
}
