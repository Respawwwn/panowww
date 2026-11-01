//! Settings: read, write, the Sessions folder picker, the logs, the
//! shortcuts.

use panowww_store::settings::Settings;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{AppError, CmdResult};
use crate::shortcuts;
use crate::state::SharedState;

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> Settings {
    state.settings()
}

/// Writes the settings. Changed shortcuts are registered first: a combo
/// the OS refuses, or Start equal to Stop, is reported and nothing is
/// saved.
#[tauri::command]
pub fn save_settings<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, SharedState>,
    settings: Settings,
) -> CmdResult<()> {
    let previous = state.settings();
    if previous.shortcuts != settings.shortcuts
        && let Err(error) = shortcuts::register_shortcuts(&app, &settings.shortcuts)
    {
        // Back to the ones that worked.
        let _ = shortcuts::register_shortcuts(&app, &previous.shortcuts);
        return Err(error);
    }
    let mut current = state
        .settings
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *current = settings;
    current.save(&state.paths)?;
    Ok(())
}

/// Settings window open or closed: the global shortcuts pause while a
/// combo may be recorded, so recording Start does not start a Session.
#[tauri::command]
pub fn set_shortcuts_enabled<R: tauri::Runtime>(app: AppHandle<R>, enabled: bool) -> CmdResult<()> {
    shortcuts::set_enabled(&app, enabled)
}

/// Folder picker for the Sessions folder; `None` when cancelled. The
/// choice is not saved here: the UI saves the settings as a whole.
#[tauri::command]
pub async fn choose_sessions_folder<R: tauri::Runtime>(
    app: AppHandle<R>,
) -> CmdResult<Option<String>> {
    let dialog = app.dialog().file();
    let chosen = tauri::async_runtime::spawn_blocking(move || dialog.blocking_pick_folder())
        .await
        .map_err(AppError::internal)?;
    Ok(chosen
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string()))
}

/// Settings → *Reveal logs*: the app's logs folder in Finder / Explorer.
#[tauri::command]
pub fn reveal_logs<R: tauri::Runtime>(app: AppHandle<R>) -> CmdResult<()> {
    let logs = app.path().app_log_dir()?;
    Ok(panowww_store::sessions::reveal(&logs)?)
}

/// Re-registers the global shortcuts from Settings; the error names the
/// combo that failed.
#[tauri::command]
pub fn apply_shortcuts<R: tauri::Runtime>(app: AppHandle<R>) -> CmdResult<()> {
    shortcuts::register(&app)
}
