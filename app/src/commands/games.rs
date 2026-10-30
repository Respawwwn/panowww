//! Games: list, create, save, duplicate, delete, share and import through
//! the system dialogs, the Start card estimate, the calibration adjust.

use panowww_core::calibrate::{self, Magnitude, Observation};
use panowww_core::game::Game;
use panowww_core::plan::{RunKind, estimate, format_duration, plan};
use panowww_store::games::{GameEntry, Import};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::dto::{AxisDto, Estimate};
use crate::error::{AppError, CmdResult};
use crate::runner::COUNTDOWN_BUTTON;
use crate::state::SharedState;

#[tauri::command]
pub fn list_games(state: State<'_, SharedState>) -> CmdResult<Vec<GameEntry>> {
    Ok(state.games.list()?)
}

#[tauri::command]
pub fn create_game(state: State<'_, SharedState>, name: String) -> CmdResult<Game> {
    let defaults = state.settings().new_game_defaults;
    Ok(state.games.create(&name, defaults)?)
}

/// Autosave on every valid change; renames the file when the name changed.
#[tauri::command]
pub fn save_game(state: State<'_, SharedState>, game: Game) -> CmdResult<String> {
    Ok(state.games.save(&game)?.display().to_string())
}

#[tauri::command]
pub fn duplicate_game(state: State<'_, SharedState>, id: Uuid) -> CmdResult<Game> {
    Ok(state.games.duplicate(id)?)
}

#[tauri::command]
pub fn delete_game(state: State<'_, SharedState>, id: Uuid) -> CmdResult<()> {
    state.games.delete(id)?;
    let mut local = state
        .local
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    local.remove(id);
    local.save(&state.paths)?;
    if state.selected() == Some(id) {
        *state
            .selected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
    Ok(())
}

/// For damaged files, which have no id.
#[tauri::command]
pub fn delete_game_file(state: State<'_, SharedState>, path: String) -> CmdResult<()> {
    Ok(state.games.delete_file(std::path::Path::new(&path))?)
}

/// *Share*: a save dialog, then a copy of the file. `false` when cancelled.
#[tauri::command]
pub async fn share_game<R: tauri::Runtime>(app: AppHandle<R>, id: Uuid) -> CmdResult<bool> {
    let state = tauri::Manager::state::<SharedState>(&app).inner().clone();
    let (game, _) = state
        .games
        .find(id)?
        .ok_or_else(|| AppError::not_found("The Game"))?;
    let file_name = panowww_core::game::name::file_name(&game.name, &[""; 0]);
    let dialog = app
        .dialog()
        .file()
        .set_file_name(file_name)
        .add_filter("Panowww Game", &["json"]);
    let chosen = tauri::async_runtime::spawn_blocking(move || dialog.blocking_save_file())
        .await
        .map_err(AppError::internal)?;
    let Some(path) = chosen.and_then(|p| p.into_path().ok()) else {
        return Ok(false);
    };
    state.games.share_to(id, &path)?;
    Ok(true)
}

/// *Import…*: a file picker, then the store's import. `None` when
/// cancelled; `KnownId` leaves the decision to the user.
#[tauri::command]
pub async fn import_game<R: tauri::Runtime>(app: AppHandle<R>) -> CmdResult<Option<Import>> {
    let state = tauri::Manager::state::<SharedState>(&app).inner().clone();
    let dialog = app.dialog().file().add_filter("Panowww Game", &["json"]);
    let chosen = tauri::async_runtime::spawn_blocking(move || dialog.blocking_pick_file())
        .await
        .map_err(AppError::internal)?;
    let Some(path) = chosen.and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    Ok(Some(state.games.import(&path)?))
}

#[tauri::command]
pub fn import_replace(state: State<'_, SharedState>, incoming: Game) -> CmdResult<Game> {
    Ok(state.games.import_replace(incoming)?)
}

#[tauri::command]
pub fn import_keep_both(state: State<'_, SharedState>, incoming: Game) -> CmdResult<Game> {
    Ok(state.games.import_keep_both(incoming)?)
}

/// The Start card line: countdown from the button plus the plan (§7.6).
#[tauri::command]
pub fn estimate_session(game: Game) -> Estimate {
    let plan = plan(&game, RunKind::Session);
    let total = estimate(&plan, COUNTDOWN_BUTTON);
    Estimate {
        duration_ms: u64::try_from(total.as_millis()).unwrap_or(u64::MAX),
        duration: format_duration(total),
        shots: u32::try_from(plan.expected_shots()).unwrap_or(u32::MAX),
    }
}

#[tauri::command]
pub fn adjust_step(
    state: State<'_, SharedState>,
    id: Uuid,
    axis: AxisDto,
    observation: String,
    magnitude: String,
) -> CmdResult<Game> {
    let (mut game, _) = state
        .games
        .find(id)?
        .ok_or_else(|| AppError::not_found("The Game"))?;
    let observation = match observation.as_str() {
        "overshot" => Observation::Overshot,
        "undershot" => Observation::Undershot,
        other => return Err(AppError::internal(format!("unknown observation {other}"))),
    };
    let magnitude = match magnitude.as_str() {
        "fine" => Magnitude::Fine,
        "coarse" => Magnitude::Coarse,
        other => return Err(AppError::internal(format!("unknown magnitude {other}"))),
    };
    calibrate::adjust_step(&mut game, axis.into(), observation, magnitude);
    state.games.save(&game)?;
    Ok(game)
}

/// The Game the detail area shows; what the Start shortcut runs.
#[tauri::command]
pub fn select_game(state: State<'_, SharedState>, id: Option<Uuid>) {
    *state
        .selected
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = id;
}
