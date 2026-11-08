//! `settings.json` (spec §6): the Sessions folder, the two global
//! shortcuts, the New Game defaults, the update check, the Display
//! preview switch, the appearance and the Stitchers' paths.

use std::path::{Path, PathBuf};

use panowww_core::game::NewGameDefaults;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::StoreError;
use crate::json::{self, Loaded};
use crate::paths::{AppPaths, default_sessions_folder, expand_home};
use crate::stitchers::Stitcher;

/// Value of the `format` field.
pub const FORMAT: &str = "panowww-settings";
/// Schema version this build writes.
pub const CURRENT_VERSION: u32 = 1;

/// The single accepted value of the `format` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SettingsFormat {
    #[default]
    #[serde(rename = "panowww-settings")]
    PanowwwSettings,
}

/// A global shortcut, stored as modifiers plus the physical key's
/// `KeyboardEvent.code`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub code: String,
}

impl Shortcut {
    pub fn new(modifiers: impl IntoIterator<Item = Modifier>, code: &str) -> Self {
        Self {
            modifiers: modifiers.into_iter().collect(),
            code: code.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Meta,
}

/// The Start and Stop shortcuts; defaults `Ctrl+Shift+8` / `Ctrl+Shift+9`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Shortcuts {
    pub start: Shortcut,
    pub stop: Shortcut,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            start: Shortcut::new([Modifier::Ctrl, Modifier::Shift], "Digit8"),
            stop: Shortcut::new([Modifier::Ctrl, Modifier::Shift], "Digit9"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
// Four independent switches, each one key of settings.json; a sub-struct
// would change the file for nothing.
#[expect(clippy::struct_excessive_bools, reason = "one key per switch")]
pub struct Settings {
    pub format: SettingsFormat,
    pub version: u32,
    pub sessions_folder: PathBuf,
    pub shortcuts: Shortcuts,
    pub new_game_defaults: NewGameDefaults,
    pub check_for_updates: bool,
    pub display_preview: bool,
    /// Light, dark, or the OS appearance. Added after the first version 1
    /// files: missing means `Auto`.
    #[serde(default)]
    pub appearance: Appearance,
    /// Onboarding was closed once with *Later* or *Continue* (spec §11.11:
    /// shown at first launch). Missing in older files means not yet.
    #[serde(default)]
    pub onboarded: bool,
    /// The update check also offers pre-releases (spec §14). Off by default.
    #[serde(default)]
    pub include_prereleases: bool,
    /// Where the Stitchers are (spec §11.12); missing in older files means
    /// none is set.
    #[serde(default)]
    pub stitchers: Stitchers,
}

/// The program behind each *Open in…* button of a completed Session;
/// `None` hides the button. macOS keeps the `.app`, Windows the `.exe`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Stitchers {
    pub ptgui: Option<PathBuf>,
    pub autopano_giga: Option<PathBuf>,
}

impl Stitchers {
    #[must_use]
    pub fn path(&self, stitcher: Stitcher) -> Option<&Path> {
        match stitcher {
            Stitcher::Ptgui => self.ptgui.as_deref(),
            Stitcher::AutopanoGiga => self.autopano_giga.as_deref(),
        }
    }

    /// The Stitchers that are set, in display order.
    pub fn set(&self) -> impl Iterator<Item = Stitcher> + '_ {
        [Stitcher::Ptgui, Stitcher::AutopanoGiga]
            .into_iter()
            .filter(|s| self.path(*s).is_some())
    }
}

/// The window's appearance (spec §12: follows the OS by default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Appearance {
    #[default]
    Auto,
    Light,
    Dark,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: SettingsFormat::PanowwwSettings,
            version: CURRENT_VERSION,
            sessions_folder: default_sessions_folder(),
            shortcuts: Shortcuts::default(),
            new_game_defaults: NewGameDefaults::default(),
            check_for_updates: true,
            display_preview: true,
            appearance: Appearance::Auto,
            onboarded: false,
            include_prereleases: false,
            stitchers: Stitchers::default(),
        }
    }
}

/// Version 0: the pre-release file, every key optional, unknown keys
/// ignored. Missing keys take the defaults.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsV0 {
    sessions_folder: Option<PathBuf>,
    shortcuts: Option<Shortcuts>,
    new_game_defaults: Option<NewGameDefaults>,
    check_for_updates: Option<bool>,
    display_preview: Option<bool>,
}

impl Settings {
    /// Reads `settings.json`. A missing file yields the defaults; an older
    /// file is migrated. Both are written back so the file on disk is
    /// always current. A damaged file is an error: see
    /// [`Settings::load_or_default`].
    pub fn load(paths: &AppPaths) -> Result<Self, StoreError> {
        let path = paths.settings_file();
        let mut loaded = match json::read_optional(&path)? {
            Some(text) => read(&path, &text)?,
            None => Loaded {
                value: Self::default(),
                migrated: true,
            },
        };
        // A file holding the UI's `~/…` placeholder is rewritten absolute.
        if let Some(folder) = expand_home(&loaded.value.sessions_folder) {
            loaded.value.sessions_folder = folder;
            loaded.migrated = true;
        }
        if loaded.migrated {
            loaded.value.save(paths)?;
        }
        Ok(loaded.value)
    }

    /// [`Settings::load`], falling back to the defaults when the file is
    /// damaged, with the reason in the log. The damaged file stays on disk
    /// until the next save.
    pub fn load_or_default(paths: &AppPaths) -> Self {
        Self::load(paths).unwrap_or_else(|error| {
            tracing::warn!(%error, "settings.json unreadable, using the defaults");
            Self::default()
        })
    }

    pub fn save(&self, paths: &AppPaths) -> Result<(), StoreError> {
        json::write_pretty(&paths.settings_file(), self)
    }
}

/// Parses the text of a settings file of any supported version.
fn read(path: &Path, text: &str) -> Result<Loaded<Settings>, StoreError> {
    let version = json::check_envelope(path, text, FORMAT, u64::from(CURRENT_VERSION))?;
    if version == 1 {
        Ok(Loaded {
            value: json::parse(path, text)?,
            migrated: false,
        })
    } else {
        let v0: SettingsV0 = json::parse(path, text)?;
        let defaults = Settings::default();
        Ok(Loaded {
            value: Settings {
                format: SettingsFormat::PanowwwSettings,
                version: CURRENT_VERSION,
                sessions_folder: v0.sessions_folder.unwrap_or(defaults.sessions_folder),
                shortcuts: v0.shortcuts.unwrap_or(defaults.shortcuts),
                new_game_defaults: v0.new_game_defaults.unwrap_or(defaults.new_game_defaults),
                check_for_updates: v0.check_for_updates.unwrap_or(defaults.check_for_updates),
                display_preview: v0.display_preview.unwrap_or(defaults.display_preview),
                appearance: defaults.appearance,
                onboarded: defaults.onboarded,
                include_prereleases: defaults.include_prereleases,
                stitchers: defaults.stitchers,
            },
            migrated: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> (tempfile::TempDir, AppPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path());
        (dir, paths)
    }

    #[test]
    fn defaults_match_the_spec() {
        let s = Settings::default();
        assert_eq!(s.shortcuts.start.code, "Digit8");
        assert_eq!(s.shortcuts.stop.code, "Digit9");
        assert_eq!(
            s.shortcuts.start.modifiers,
            [Modifier::Ctrl, Modifier::Shift]
        );
        assert_eq!(s.new_game_defaults, NewGameDefaults::default());
        assert!(s.check_for_updates && s.display_preview);
        assert!(s.sessions_folder.ends_with("Pictures/Panowww"));
    }

    #[test]
    fn a_missing_file_yields_and_writes_the_defaults() {
        let (_dir, paths) = paths();
        let settings = Settings::load(&paths).unwrap();
        assert_eq!(settings, Settings::default());
        let text = std::fs::read_to_string(paths.settings_file()).unwrap();
        assert!(text.contains(r#""format": "panowww-settings""#));
        assert!(text.contains(r#""version": 1"#));
        assert!(text.contains(r#""sessionsFolder""#), "{text}");
    }

    #[test]
    fn save_then_load_round_trips() {
        let (_dir, paths) = paths();
        let mut settings = Settings {
            display_preview: false,
            ..Settings::default()
        };
        settings.shortcuts.start = Shortcut::new([Modifier::Alt], "KeyP");
        settings.sessions_folder = PathBuf::from("/Volumes/Shots");
        settings.save(&paths).unwrap();
        assert_eq!(Settings::load(&paths).unwrap(), settings);
    }

    #[test]
    fn a_version_one_file_without_the_newer_keys_loads_with_their_defaults() {
        let (_dir, paths) = paths();
        let mut text = serde_json::to_string(&Settings::default()).unwrap();
        for key in [
            r#","appearance":"auto""#,
            r#","onboarded":false"#,
            r#","includePrereleases":false"#,
            r#","stitchers":{"ptgui":null,"autopanoGiga":null}"#,
        ] {
            assert!(text.contains(key), "{text}");
            text = text.replace(key, "");
        }
        std::fs::write(paths.settings_file(), text).unwrap();
        let settings = Settings::load(&paths).unwrap();
        assert_eq!(settings.appearance, Appearance::Auto);
        assert!(!settings.onboarded);
        assert!(!settings.include_prereleases);
        assert_eq!(settings.stitchers, Stitchers::default());
    }

    #[test]
    fn a_stitcher_path_round_trips_and_lists_as_set() {
        let (_dir, paths) = paths();
        let mut settings = Settings::default();
        settings.stitchers.autopano_giga = Some(PathBuf::from("C:/Kolor/AutopanoGiga_x64.exe"));
        settings.save(&paths).unwrap();
        let loaded = Settings::load(&paths).unwrap();
        assert_eq!(loaded.stitchers, settings.stitchers);
        assert_eq!(loaded.stitchers.path(Stitcher::Ptgui), None);
        assert_eq!(
            loaded.stitchers.set().collect::<Vec<_>>(),
            [Stitcher::AutopanoGiga]
        );
    }

    #[test]
    fn a_version_zero_file_is_migrated_and_rewritten() {
        let (_dir, paths) = paths();
        std::fs::write(
            paths.settings_file(),
            r#"{"format":"panowww-settings","version":0,"displayPreview":false,"obsoleteKey":1}"#,
        )
        .unwrap();
        let settings = Settings::load(&paths).unwrap();
        assert!(!settings.display_preview);
        assert_eq!(
            settings.shortcuts,
            Shortcuts::default(),
            "missing keys take the defaults"
        );
        let text = std::fs::read_to_string(paths.settings_file()).unwrap();
        assert!(text.contains(r#""version": 1"#));
        assert!(!text.contains("obsoleteKey"));
    }

    #[test]
    fn damaged_newer_and_foreign_files_are_errors_and_load_or_default_recovers() {
        let (_dir, paths) = paths();
        for (text, check) in [
            ("{nope", "Json"),
            (r#"{"format":"panowww-game","version":1}"#, "WrongFormat"),
            (
                r#"{"format":"panowww-settings","version":2}"#,
                "NewerVersion",
            ),
            (
                r#"{"format":"panowww-settings","version":1,"unknown":true}"#,
                "Json",
            ),
        ] {
            std::fs::write(paths.settings_file(), text).unwrap();
            let error = Settings::load(&paths).unwrap_err();
            assert!(format!("{error:?}").starts_with(check), "{text}: {error:?}");
            assert_eq!(Settings::load_or_default(&paths), Settings::default());
            assert_eq!(
                std::fs::read_to_string(paths.settings_file()).unwrap(),
                text,
                "left in place"
            );
        }
    }
}
