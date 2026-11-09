//! The Sessions tables: listing, size, zip, reveal, delete, thumbnails,
//! and opening a Session in a Stitcher.

use std::path::Path;

use panowww_store::session::SessionRecord;
use panowww_store::sessions::{self, SessionSummary};
use panowww_store::stitchers::{self, Stitcher};
use panowww_store::thumbnails;
use tauri::State;
use uuid::Uuid;

use crate::error::{AppError, CmdResult};
use crate::runner::data_url;
use crate::state::SharedState;

/// All Sessions, or those of one Game, newest first.
#[tauri::command]
pub fn list_sessions(
    state: State<'_, SharedState>,
    game_id: Option<Uuid>,
) -> CmdResult<Vec<SessionSummary>> {
    let folder = state.settings().sessions_folder;
    Ok(match game_id {
        Some(id) => sessions::list_for_game(&folder, id)?,
        None => sessions::list_all(&folder)?,
    })
}

/// Bytes of the Session folder, computed on demand.
#[tauri::command]
pub fn session_size(folder: String) -> CmdResult<u64> {
    Ok(sessions::folder_size(Path::new(&folder))?)
}

/// Writes `<folder>.zip` next to the folder; returns its path.
#[tauri::command]
pub fn zip_session(folder: String) -> CmdResult<String> {
    Ok(sessions::zip_session(Path::new(&folder))?
        .display()
        .to_string())
}

#[tauri::command]
pub fn delete_session(state: State<'_, SharedState>, folder: String) -> CmdResult<()> {
    let folder = Path::new(&folder);
    if let Ok(record) = SessionRecord::read(folder) {
        thumbnails::remove_thumbnails(&state.paths, &record)?;
    }
    sessions::delete_session(folder)?;
    Ok(())
}

/// The history thumbnail of one Shot as a `data:` URL, generated on demand.
#[tauri::command]
pub fn session_thumbnail(
    state: State<'_, SharedState>,
    folder: String,
    file: String,
) -> CmdResult<String> {
    let folder = Path::new(&folder);
    let record = SessionRecord::read(folder)?;
    let path = thumbnails::history_thumbnail(&state.paths, folder, &record, &file)?;
    let bytes =
        std::fs::read(&path).map_err(|e| AppError::internal(format!("{}: {e}", path.display())))?;
    Ok(data_url("image/jpeg", &bytes))
}

/// Shows a file or folder selected in Finder / Explorer.
#[tauri::command]
pub fn reveal_path(path: String) -> CmdResult<()> {
    Ok(sessions::reveal(Path::new(&path))?)
}

/// Session end → *Open in PTGui* / *Open in Autopano Giga* (spec §11.7):
/// the program set in Settings opens the Session's Shots. Blocking while
/// PTGui creates its project (a few seconds), so off the main thread.
#[tauri::command]
pub async fn open_in_stitcher(
    state: State<'_, SharedState>,
    folder: String,
    stitcher: Stitcher,
) -> CmdResult<()> {
    let program = state
        .settings()
        .stitchers
        .path(stitcher)
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::not_found(format!("{} in Settings", stitcher.name())))?;
    tauri::async_runtime::spawn_blocking(move || {
        stitchers::open(stitcher, &program, Path::new(&folder))
    })
    .await
    .map_err(AppError::internal)??;
    Ok(())
}
