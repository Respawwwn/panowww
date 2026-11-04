//! The Sessions on disk (spec §6, §11.9): listing per Game and global,
//! newest first; stale `running` Sessions marked `interrupted` at launch;
//! size on demand; zip next to the folder; reveal; delete.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::StoreError;
use crate::session::{SessionRecord, SessionStatus, ShotEntry, TIMESTAMP};
use crate::sink::WallClock;

/// One Session folder and its record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionSummary {
    pub folder: PathBuf,
    pub record: SessionRecord,
}

/// Every readable Session under `sessions_folder`, newest first. Folders
/// without a readable `session.json` are skipped and logged.
pub fn list_all(sessions_folder: &Path) -> Result<Vec<SessionSummary>, StoreError> {
    let mut sessions = Vec::new();
    for folder in session_folders(sessions_folder)? {
        match SessionRecord::read(&folder) {
            Ok(record) => sessions.push(SessionSummary { folder, record }),
            Err(error) => tracing::warn!(%error, "Session folder skipped"),
        }
    }
    sessions.sort_by(|a, b| {
        b.record
            .started_at()
            .cmp(&a.record.started_at())
            .then_with(|| b.folder.cmp(&a.folder))
    });
    Ok(sessions)
}

/// The Sessions of one Game, newest first.
pub fn list_for_game(
    sessions_folder: &Path,
    game_id: Uuid,
) -> Result<Vec<SessionSummary>, StoreError> {
    Ok(list_all(sessions_folder)?
        .into_iter()
        .filter(|s| s.record.game.id == game_id)
        .collect())
}

/// At launch: a Session still `running` was cut short by a crash or a
/// force quit. Marks it `interrupted` with the Shots found on disk.
/// Returns how many were marked.
pub fn mark_stale_running(
    sessions_folder: &Path,
    clock: &dyn WallClock,
) -> Result<usize, StoreError> {
    let mut marked = 0;
    for session in list_all(sessions_folder)? {
        if session.record.status != SessionStatus::Running {
            continue;
        }
        let mut record = session.record;
        let ended_at = clock.now().strftime(TIMESTAMP).to_string();
        record.status = SessionStatus::Interrupted;
        record.ended_at = Some(ended_at.clone());
        record
            .shots
            .retain(|s| session.folder.join(&s.file).exists());
        // A crash leaves the PNG files the record never listed (it is
        // written at the end): they are the Shots taken.
        for (file, row, col) in shot_files(&session.folder)? {
            if !record.shots.iter().any(|s| s.file == file) {
                record.shots.push(ShotEntry {
                    file,
                    row,
                    col,
                    taken_at: ended_at.clone(),
                    capture_ms: 0,
                });
            }
        }
        record.shots.sort_by_key(|s| (s.row, s.col));
        record.shots_taken = u32::try_from(record.shots.len()).unwrap_or(u32::MAX);
        record.write(&session.folder)?;
        marked += 1;
    }
    Ok(marked)
}

/// The `rNN_cNN.png` files of a Session folder, with their Row and column.
fn shot_files(folder: &Path) -> Result<Vec<(String, u32, u32)>, StoreError> {
    let mut shots = Vec::new();
    for entry in std::fs::read_dir(folder).map_err(|e| StoreError::io("read", folder, e))? {
        let entry = entry.map_err(|e| StoreError::io("read", folder, e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some((row, col)) = parse_shot_name(&name) {
            shots.push((name, row, col));
        }
    }
    Ok(shots)
}

/// `r03_c12.png` → `(3, 12)`.
fn parse_shot_name(name: &str) -> Option<(u32, u32)> {
    let stem = name.strip_suffix(".png")?;
    let (row, col) = stem.split_once("_c")?;
    let row = row.strip_prefix('r')?.parse().ok()?;
    let col = col.parse().ok()?;
    Some((row, col))
}

/// The sum of the files directly in `folder`, in bytes.
pub fn folder_size(folder: &Path) -> Result<u64, StoreError> {
    let mut total = 0;
    for entry in std::fs::read_dir(folder).map_err(|e| StoreError::io("read", folder, e))? {
        let entry = entry.map_err(|e| StoreError::io("read", folder, e))?;
        let metadata = entry
            .metadata()
            .map_err(|e| StoreError::io("read", entry.path(), e))?;
        if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
}

/// `<parent>/<folder name>.zip`
pub fn zip_path(folder: &Path) -> PathBuf {
    let name = folder.file_name().unwrap_or_default().to_string_lossy();
    folder.with_file_name(format!("{name}.zip"))
}

/// Writes the Session folder's files into [`zip_path`], stored without
/// further compression (PNG already is). Replaces an existing zip.
pub fn zip_session(folder: &Path) -> Result<PathBuf, StoreError> {
    let target = zip_path(folder);
    let temp = target.with_extension("zip.tmp");
    let file = std::fs::File::create(&temp).map_err(|e| StoreError::io("create", &temp, e))?;
    let mut zip = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|e| StoreError::io("read", folder, e))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    let zip_error = |e: zip::result::ZipError| StoreError::Zip {
        path: target.clone(),
        message: e.to_string(),
    };
    for path in files {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        zip.start_file(name, options).map_err(zip_error)?;
        let mut source =
            std::fs::File::open(&path).map_err(|e| StoreError::io("read", &path, e))?;
        std::io::copy(&mut source, &mut zip).map_err(|e| StoreError::io("write", &temp, e))?;
    }
    zip.finish().map_err(zip_error)?;
    std::fs::rename(&temp, &target).map_err(|e| StoreError::io("rename", &temp, e))?;
    Ok(target)
}

/// Removes the Session folder and its zip, if any.
pub fn delete_session(folder: &Path) -> Result<(), StoreError> {
    std::fs::remove_dir_all(folder).map_err(|e| StoreError::io("delete", folder, e))?;
    let zip = zip_path(folder);
    match std::fs::remove_file(&zip) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(StoreError::io("delete", &zip, e)),
    }
}

/// Shows `path` selected in Finder or Explorer.
pub fn reveal(path: &Path) -> Result<(), StoreError> {
    let mut command = if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg("-R").arg(path);
        c
    } else if cfg!(target_os = "windows") {
        // Explorer wants backslashes, and `/select,"<path>"` as one raw
        // argument: with a space in the path, std would quote the whole
        // thing and Explorer falls back to Documents.
        let native: PathBuf = path.components().collect();
        let mut c = std::process::Command::new("explorer");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.raw_arg(format!("/select,\"{}\"", native.display()));
        }
        #[cfg(not(windows))]
        c.arg(format!("/select,{}", native.display()));
        c
    } else {
        return Err(StoreError::io(
            "reveal",
            path,
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "no file manager on this OS",
            ),
        ));
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|e| StoreError::io("reveal", path, e))
}

/// `<sessions_folder>/<Game>/<timestamp>` folders holding a `session.json`.
fn session_folders(sessions_folder: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut folders = Vec::new();
    let game_dirs = match std::fs::read_dir(sessions_folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(folders),
        Err(e) => return Err(StoreError::io("read", sessions_folder, e)),
    };
    for game_dir in game_dirs
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
    {
        let sessions =
            std::fs::read_dir(&game_dir).map_err(|e| StoreError::io("read", &game_dir, e))?;
        for folder in sessions.filter_map(Result::ok).map(|e| e.path()) {
            if SessionRecord::path(&folder).is_file() {
                folders.push(folder);
            }
        }
    }
    folders.sort();
    Ok(folders)
}
