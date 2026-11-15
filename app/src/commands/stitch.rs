//! The Stitch's surface to the UI (spec §16.5): start, cancel, phase, and
//! the preview image. Events stream to the window as the `stitch` event.

use std::path::{Path, PathBuf};

use tauri::State;

use crate::dto::{StitchCancelReason, StitchPhase};
use crate::error::{AppError, CmdResult};
use crate::runner::data_url;
use crate::state::SharedState;
use crate::stitch::{StitchSpec, preview_path};

/// Starts the Stitch of the Session in `folder`. `false` when a run is in
/// progress or a Stitch already runs: the request is ignored.
#[tauri::command]
pub fn stitch_session(state: State<'_, SharedState>, folder: String) -> bool {
    state.stitch.start(StitchSpec {
        folder: PathBuf::from(folder),
        app_version: state.version.clone(),
    })
}

/// The Cancel button: stops the Stitch, nothing is written.
#[tauri::command]
pub fn stitch_cancel(state: State<'_, SharedState>) {
    state.stitch.cancel(StitchCancelReason::Button);
}

#[tauri::command]
pub fn stitch_phase(state: State<'_, SharedState>) -> StitchPhase {
    state.stitch.phase()
}

/// `panorama-preview.jpg` of the Session as a `data:` URL, for the preview
/// window.
#[tauri::command]
pub fn panorama_preview(folder: String) -> CmdResult<String> {
    let path = preview_path(Path::new(&folder));
    let bytes = std::fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::not_found("The Panorama preview")
        } else {
            AppError::internal(format!("{}: {e}", path.display()))
        }
    })?;
    Ok(data_url("image/jpeg", &bytes))
}

/// The preview's *Fill the screen*: the app window takes the whole screen
/// (macOS: simple full screen, no Space of its own; elsewhere the OS full
/// screen), or leaves it.
#[tauri::command]
pub fn panorama_full_screen<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    on: bool,
) -> CmdResult<()> {
    window
        .set_simple_fullscreen(on)
        .map_err(|e| AppError::internal(format!("full screen {on}: {e}")))
}
