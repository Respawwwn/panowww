//! Stitchers (GLOSSARY): opening a completed Session in PTGui or Autopano
//! Giga. Interface: [`Stitcher`], [`open`], [`project_path`].
//!
//! PTGui gets a project, `<Session>.pts` in the Session folder, created
//! once with `-createproject` (every Shot in it, no control points; PTGui
//! works out the field of view itself) and opened on every click. Autopano
//! Giga has no command line: it gets every Shot as an argument and loads
//! them in one group. Shots are passed as relative names with the Session
//! folder as the working directory: a 36 × 36 grid stays far under the
//! Windows command-line limit of 32 767 characters, and both programs
//! accept them. Checked with PTGui 13.10 Pro and Autopano Giga 4.4 on
//! both OSes.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::StoreError;
use crate::session::SessionRecord;

/// A program that assembles a Session's Shots into a panorama.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Stitcher {
    Ptgui,
    AutopanoGiga,
}

impl Stitcher {
    /// The name shown to the user.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ptgui => "PTGui",
            Self::AutopanoGiga => "Autopano Giga",
        }
    }
}

/// `<folder>/<folder name>.pts`: the PTGui project of a Session.
#[must_use]
pub fn project_path(folder: &Path) -> PathBuf {
    let name = folder.file_name().map_or_else(
        || "Session".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    folder.join(format!("{name}.pts"))
}

/// Opens the Session in `program`, the Stitcher's path from Settings.
pub fn open(stitcher: Stitcher, program: &Path, folder: &Path) -> Result<(), StoreError> {
    if !program.exists() {
        return Err(StoreError::StitcherMissing {
            stitcher: stitcher.name(),
            path: program.to_path_buf(),
        });
    }
    let record = SessionRecord::read(folder)?;
    let shots: Vec<&str> = record.shots.iter().map(|s| s.file.as_str()).collect();
    if shots.is_empty() {
        return Err(StoreError::Stitcher {
            stitcher: stitcher.name(),
            message: "this Session has no Shot".to_owned(),
        });
    }
    match stitcher {
        Stitcher::Ptgui => {
            let project = project_path(folder);
            if !project.exists() {
                create_project(program, folder, &shots, &project)?;
            }
            launch(program, folder, [project.as_os_str()])
        }
        Stitcher::AutopanoGiga => launch(program, folder, shots.iter().map(AsRef::as_ref)),
    }
}

/// `PTGui -createproject <Shots> -output <project>`: no window, a few
/// seconds; PTGui 13 or later.
fn create_project(
    program: &Path,
    folder: &Path,
    shots: &[&str],
    project: &Path,
) -> Result<(), StoreError> {
    let status = Command::new(executable(program))
        .arg("-createproject")
        .args(shots)
        .arg("-output")
        .arg(project)
        .current_dir(folder)
        .status()
        .map_err(|e| StoreError::io("launch", program, e))?;
    if status.success() && project.exists() {
        Ok(())
    } else {
        Err(StoreError::Stitcher {
            stitcher: Stitcher::Ptgui.name(),
            message: format!(
                "the project was not created ({status}). PTGui 13 or later is needed."
            ),
        })
    }
}

/// Starts the program with `documents`, relative to `folder`, and returns
/// at once. A macOS `.app` goes through `open -a`, which hands the
/// documents to the running instance when there is one.
fn launch<'a>(
    program: &Path,
    folder: &Path,
    documents: impl IntoIterator<Item = &'a std::ffi::OsStr>,
) -> Result<(), StoreError> {
    let mut command = if is_app_bundle(program) {
        let mut c = Command::new("open");
        c.arg("-a").arg(program);
        c
    } else {
        Command::new(program)
    };
    command
        .args(documents)
        .current_dir(folder)
        .spawn()
        .map(drop)
        .map_err(|e| StoreError::io("launch", program, e))
}

/// The binary to run with arguments: inside `Contents/MacOS` for a macOS
/// `.app`, the program itself otherwise.
fn executable(program: &Path) -> PathBuf {
    if !is_app_bundle(program) {
        return program.to_path_buf();
    }
    let binaries = program.join("Contents").join("MacOS");
    std::fs::read_dir(&binaries)
        .ok()
        .and_then(|entries| {
            let mut files: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .collect();
            files.sort();
            files.into_iter().next()
        })
        .unwrap_or(binaries)
}

fn is_app_bundle(program: &Path) -> bool {
    program.extension().is_some_and(|e| e == "app") && program.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_project_is_named_after_the_session_folder() {
        let folder = Path::new("/Pictures/Panowww/Game/2026-11-04_18-47-27");
        assert_eq!(project_path(folder), folder.join("2026-11-04_18-47-27.pts"));
    }

    #[test]
    fn a_plain_program_is_its_own_executable() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("PTGui.exe");
        std::fs::write(&exe, b"").unwrap();
        assert_eq!(executable(&exe), exe);
        assert!(!is_app_bundle(&exe));
    }

    #[test]
    fn an_app_bundle_runs_the_binary_inside_it() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("PTGui.app");
        let binaries = app.join("Contents").join("MacOS");
        std::fs::create_dir_all(&binaries).unwrap();
        std::fs::write(binaries.join("PTGui"), b"").unwrap();
        assert!(is_app_bundle(&app));
        assert_eq!(executable(&app), binaries.join("PTGui"));
    }

    #[test]
    fn a_program_that_moved_is_reported_before_anything_runs() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("PTGui.exe");
        let error = open(Stitcher::Ptgui, &gone, dir.path()).unwrap_err();
        assert!(
            matches!(error, StoreError::StitcherMissing { stitcher: "PTGui", ref path } if *path == gone),
            "{error}"
        );
        assert!(error.to_string().contains("Settings"));
    }

    #[test]
    fn a_folder_without_a_session_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("AutopanoGiga_x64.exe");
        std::fs::write(&exe, b"").unwrap();
        let error = open(Stitcher::AutopanoGiga, &exe, dir.path()).unwrap_err();
        assert!(
            matches!(error, StoreError::Io { action: "read", .. }),
            "{error}"
        );
    }
}
