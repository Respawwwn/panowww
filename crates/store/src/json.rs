//! Atomic, versioned JSON files: written through a temporary file and a
//! rename, read through a `format` / `version` envelope check so a file
//! from a newer Panowww is refused and an older one is handed to a
//! migration.

use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::StoreError;

/// Writes `text` to `path`, replacing any previous content only once the
/// whole text is on disk.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), StoreError> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let temp = path.with_file_name(format!(".{file_name}.tmp"));
    std::fs::write(&temp, text).map_err(|e| StoreError::io("write", &temp, e))?;
    std::fs::rename(&temp, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        StoreError::io("rename", path, e)
    })
}

/// Serialises `value` as pretty JSON with a trailing newline and writes it
/// atomically.
pub fn write_pretty<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), StoreError> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| StoreError::Json {
        path: path.to_owned(),
        source: e,
    })?;
    text.push('\n');
    write_atomic(path, &text)
}

/// The file's text, or `None` when it does not exist.
pub fn read_optional(path: &Path) -> Result<Option<String>, StoreError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(StoreError::io("read", path, e)),
    }
}

/// A versioned file read and brought up to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded<T> {
    pub value: T,
    /// The file was older and should be rewritten.
    pub migrated: bool,
}

/// Checks the envelope of `text`: `format` must be `expected_format`, and
/// `version` an integer at most `current`. Returns the version so the
/// caller picks its parser.
pub fn check_envelope(
    path: &Path,
    text: &str,
    expected_format: &'static str,
    current: u64,
) -> Result<u64, StoreError> {
    let value: Value = serde_json::from_str(text).map_err(|e| StoreError::Json {
        path: path.to_owned(),
        source: e,
    })?;
    let object = value.as_object().ok_or(StoreError::WrongFormat {
        path: path.to_owned(),
        expected: expected_format,
    })?;
    if object.get("format").and_then(Value::as_str) != Some(expected_format) {
        return Err(StoreError::WrongFormat {
            path: path.to_owned(),
            expected: expected_format,
        });
    }
    let version = object.get("version").and_then(Value::as_u64).unwrap_or(0);
    if version > current {
        return Err(StoreError::NewerVersion {
            path: path.to_owned(),
            version,
        });
    }
    Ok(version)
}

/// Parses `text` as `T`, naming the file in the error.
pub fn parse<T: DeserializeOwned>(path: &Path, text: &str) -> Result<T, StoreError> {
    serde_json::from_str(text).map_err(|e| StoreError::Json {
        path: path.to_owned(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_replaces_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.json");
        write_atomic(&path, "1").unwrap();
        write_atomic(&path, "2").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "2");
        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn write_atomic_names_the_folder_when_it_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("a.json");
        let error = write_atomic(&path, "1").unwrap_err();
        assert!(
            matches!(
                error,
                StoreError::Io {
                    action: "write",
                    ..
                }
            ),
            "{error}"
        );
    }

    #[test]
    fn read_optional_distinguishes_missing_from_broken() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_optional(&dir.path().join("none.json")).unwrap(), None);
        let path = dir.path().join("a.json");
        std::fs::write(&path, "x").unwrap();
        assert_eq!(read_optional(&path).unwrap(), Some("x".to_owned()));
    }

    #[test]
    fn envelope_checks_format_and_version() {
        let path = Path::new("x.json");
        assert_eq!(
            check_envelope(path, r#"{"format":"f","version":1}"#, "f", 1).unwrap(),
            1
        );
        assert_eq!(
            check_envelope(path, r#"{"format":"f"}"#, "f", 1).unwrap(),
            0,
            "no version reads as 0"
        );
        assert!(matches!(
            check_envelope(path, r#"{"format":"g","version":1}"#, "f", 1),
            Err(StoreError::WrongFormat { expected: "f", .. })
        ));
        assert!(matches!(
            check_envelope(path, r#"{"format":"f","version":2}"#, "f", 1),
            Err(StoreError::NewerVersion { version: 2, .. })
        ));
        assert!(matches!(
            check_envelope(path, "[]", "f", 1),
            Err(StoreError::WrongFormat { .. })
        ));
        assert!(matches!(
            check_envelope(path, "{", "f", 1),
            Err(StoreError::Json { .. })
        ));
    }
}
