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
    let settings = state.settings();
    tracing::debug!(appearance = ?settings.appearance, "settings read by the UI");
    settings
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
    let mut settings = settings;
    if let Some(folder) = panowww_store::paths::expand_home(&settings.sessions_folder) {
        settings.sessions_folder = folder;
    }
    let previous = state.settings();
    if previous.shortcuts != settings.shortcuts
        && let Err(error) = shortcuts::register_shortcuts(&app, &settings.shortcuts)
    {
        // Back to the ones that worked.
        let _ = shortcuts::register_shortcuts(&app, &previous.shortcuts);
        return Err(error);
    }
    let appearance = settings.appearance;
    let mut current = state
        .settings
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *current = settings;
    current.save(&state.paths)?;
    drop(current);
    apply_window_theme(&app, appearance);
    Ok(())
}

/// The native window chrome follows the appearance (spec §12). On Windows
/// the caption bar would otherwise keep the OS theme over a light page;
/// macOS hides its title bar behind the page and is left as validated.
pub fn apply_window_theme<R: tauri::Runtime>(
    app: &AppHandle<R>,
    appearance: panowww_store::settings::Appearance,
) {
    if !cfg!(target_os = "windows") {
        return;
    }
    if let Some(window) = app.get_webview_window(crate::MAIN_WINDOW)
        && let Err(error) = window.set_theme(window_theme(appearance))
    {
        tracing::warn!(%error, "window theme not applied");
    }
}

/// The native theme of an appearance: none for Auto (the OS decides).
pub fn window_theme(appearance: panowww_store::settings::Appearance) -> Option<tauri::Theme> {
    match appearance {
        panowww_store::settings::Appearance::Auto => None,
        panowww_store::settings::Appearance::Light => Some(tauri::Theme::Light),
        panowww_store::settings::Appearance::Dark => Some(tauri::Theme::Dark),
    }
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
