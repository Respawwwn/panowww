//! Starting and stopping runs. Events stream to the window as the `run`
//! event (spec §4.4).

use std::time::Duration;

use panowww_store::sink::SessionSinkConfig;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::dto::{RunKindDto, RunnerPhase};
use crate::error::{AppError, AppErrorKind, CmdResult};
use crate::runner::{COUNTDOWN_BUTTON, RunSpec};
use crate::state::SharedState;

/// Builds the spec of a run of `game_id` from the store, or says why it
/// cannot start.
fn spec_for(
    state: &SharedState,
    game_id: Uuid,
    kind: RunKindDto,
    countdown: Duration,
) -> CmdResult<RunSpec> {
    let (game, _) = state
        .games
        .find(game_id)?
        .ok_or_else(|| AppError::not_found("The Game"))?;
    let zone = state
        .local
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .zone(game_id)
        .cloned()
        .ok_or_else(|| AppError::new(AppErrorKind::NotFound, "Set the Capture zone first."))?;
    let settings = state.settings();
    Ok(RunSpec {
        game,
        kind,
        zone,
        countdown,
        session: SessionSinkConfig::new(settings.sessions_folder, &state.version),
        preview: settings.display_preview,
    })
}

/// Pre-flight, stream, countdown, then the run, from a button (5 s).
/// Ignored while a run is in progress; `false` says so.
#[tauri::command]
pub fn start_run(
    state: State<'_, SharedState>,
    game_id: Uuid,
    kind: RunKindDto,
) -> CmdResult<bool> {
    let spec = spec_for(&state, game_id, kind, COUNTDOWN_BUTTON)?;
    Ok(state.runner.start(spec))
}

/// The Start shortcut: a Session of the selected Game, 3 s countdown,
/// ignored with no Game selected or a run in progress (spec §7.2).
pub fn start_selected_session<R: tauri::Runtime>(app: &AppHandle<R>, countdown: Duration) {
    let state = app.state::<SharedState>().inner().clone();
    let Some(game_id) = state.selected() else {
        tracing::info!("start shortcut ignored: no Game selected");
        return;
    };
    match spec_for(&state, game_id, RunKindDto::Session, countdown) {
        Ok(spec) => {
            state.runner.start(spec);
        }
        Err(error) => tracing::info!(%error, "start shortcut ignored"),
    }
}

/// Cancels a countdown or stops a run; `release_all` first.
#[tauri::command]
pub fn stop_run(state: State<'_, SharedState>) {
    state.runner.stop();
}

/// The Hide / Show preview button: persisted in Settings and applied to
/// the current run.
#[tauri::command]
pub fn set_preview(state: State<'_, SharedState>, on: bool) -> CmdResult<()> {
    {
        let mut settings = state
            .settings
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        settings.display_preview = on;
        settings.save(&state.paths)?;
    }
    state.runner.set_preview(on);
    Ok(())
}

#[tauri::command]
pub fn runner_phase(state: State<'_, SharedState>) -> RunnerPhase {
    state.runner.phase()
}
