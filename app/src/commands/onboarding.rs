//! Permissions for onboarding and the countdown error state, links, the
//! update check.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::dto::{PermissionKind, Permissions, UpdateInfo};
use crate::error::{AppError, CmdResult};
use crate::state::SharedState;
use crate::update;

#[tauri::command]
pub fn permissions(state: State<'_, SharedState>) -> Permissions {
    Permissions {
        accessibility: state.backend.accessibility_granted(),
        screen_recording: state.backend.screen_recording_granted(),
        vigem: state.backend.gamepad_available() || cfg!(not(windows)),
    }
}

/// Asks the OS; the system prompt shows the first time. Returns whether
/// it is granted now.
#[tauri::command]
pub fn request_permission(state: State<'_, SharedState>, permission: PermissionKind) -> bool {
    state.backend.request_permission(permission)
}

#[tauri::command]
pub fn open_system_settings(
    state: State<'_, SharedState>,
    permission: PermissionKind,
) -> CmdResult<()> {
    Ok(state.backend.open_permission_settings(permission)?)
}

/// Opens a web page in the default browser.
#[tauri::command]
pub fn open_url<R: tauri::Runtime>(app: AppHandle<R>, url: String) -> CmdResult<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(AppError::internal("only web links open from here"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(AppError::internal)
}

/// `None` when up to date, offline, or the switch is off.
#[tauri::command]
pub async fn check_for_updates(state: State<'_, SharedState>) -> CmdResult<Option<UpdateInfo>> {
    if !state.settings().check_for_updates {
        return Ok(None);
    }
    let version = state.version.clone();
    Ok(
        tauri::async_runtime::spawn_blocking(move || update::check(&version))
            .await
            .ok()
            .flatten(),
    )
}
