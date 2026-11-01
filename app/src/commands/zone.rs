//! The Capture zone of a Game on this machine, and the displays.

use panowww_core::ports::{DisplayInfo, Rect, Zone};
use panowww_store::local::zone_status;
use tauri::{AppHandle, Runtime, State};
use uuid::Uuid;

use crate::dto::{PickerSetup, RunnerPhase, ZoneState};
use crate::error::{AppError, AppErrorKind, CmdResult};
use crate::state::SharedState;

/// What the Setup shows: the zone, whether its display is still there, and
/// the display's position for `Display {n}`.
#[tauri::command]
pub fn get_zone(state: State<'_, SharedState>, game_id: Uuid) -> CmdResult<ZoneState> {
    zone_state(&state, game_id)
}

pub fn zone_state(state: &SharedState, game_id: Uuid) -> CmdResult<ZoneState> {
    let zone = state
        .local
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .zone(game_id)
        .cloned();
    let Some(zone) = zone else {
        return Ok(ZoneState {
            zone: None,
            status: None,
            display_index: None,
        });
    };
    let displays = state.backend.displays()?;
    let status = zone_status(&zone, &displays);
    let display_index = displays
        .iter()
        .position(|d| d.id == zone.display_id)
        .and_then(|i| u32::try_from(i + 1).ok());
    Ok(ZoneState {
        zone: Some(zone),
        status: Some(status),
        display_index,
    })
}

/// Stores the zone the picker drew (issue 13 calls it).
#[tauri::command]
pub fn set_zone(state: State<'_, SharedState>, game_id: Uuid, zone: Zone) -> CmdResult<ZoneState> {
    {
        let mut local = state
            .local
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        local.set_zone(game_id, zone);
        local.save(&state.paths)?;
    }
    zone_state(&state, game_id)
}

/// Opens one picker window per display and resolves once one of them
/// saved a zone, or the pick was cancelled (spec §11.4).
#[tauri::command]
pub async fn pick_zone<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SharedState>,
    game_id: Uuid,
) -> CmdResult<ZoneState> {
    let state = state.inner().clone();
    let current = state
        .local
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .zone(game_id)
        .cloned();
    let rx = state
        .picker
        .open(&app, &state.backend, game_id, current.as_ref())?;
    // The reply comes from a picker window's command, on another thread.
    tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await
        .map_err(|e| AppError::internal(format!("picker wait: {e}")))?
        .map_err(|_| AppError::internal("the picker went away"))?;
    zone_state(&state, game_id)
}

/// What a picker window shows: called by the page when it loads.
#[tauri::command]
pub fn picker_setup(state: State<'_, SharedState>, label: String) -> CmdResult<PickerSetup> {
    let current = state.picker.game_id().and_then(|game_id| {
        state
            .local
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .zone(game_id)
            .cloned()
    });
    state
        .picker
        .setup(&label, current.as_ref(), state.settings().appearance)
}

/// *Save zone*: stores the zone record keyed by the Game and ends the pick.
#[tauri::command]
pub fn picker_save<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SharedState>,
    label: String,
    rect: Rect,
) -> CmdResult<()> {
    let zone = state.picker.zone_for(&label, rect)?;
    let game_id = state
        .picker
        .game_id()
        .ok_or_else(|| AppError::new(AppErrorKind::NotFound, "No pick in progress."))?;
    {
        let mut local = state
            .local
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        local.set_zone(game_id, zone.clone());
        local.save(&state.paths)?;
    }
    state.picker.finish(&app, &state.overlay, Some(zone));
    Ok(())
}

/// *Cancel* or Esc in any picker window.
#[tauri::command]
pub fn picker_cancel<R: Runtime>(app: AppHandle<R>, state: State<'_, SharedState>) {
    state.picker.finish(&app, &state.overlay, None);
}

/// The Setup's Show zone / Hide zone toggle: the overlay frame alone, so
/// the user can position the game (spec §11.2). Ignored while a run owns
/// the overlay.
#[tauri::command]
pub fn show_zone(state: State<'_, SharedState>, game_id: Uuid, on: bool) -> CmdResult<()> {
    if state.runner.phase() != RunnerPhase::Idle {
        tracing::info!(on, "show_zone ignored: a run is in progress");
        return Ok(());
    }
    if !on {
        state.overlay.hide();
        return Ok(());
    }
    let zone = state
        .local
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .zone(game_id)
        .cloned()
        .ok_or_else(|| AppError::new(AppErrorKind::NotFound, "Set the Capture zone first."))?;
    state.overlay.show_frame(&zone);
    Ok(())
}

/// What the overlay page shows; asked when it loads, in case a state was
/// sent before it listened.
#[tauri::command]
pub fn overlay_state(state: State<'_, SharedState>) -> Option<crate::dto::OverlayState> {
    state.overlay.current()
}

#[tauri::command]
pub fn list_displays(state: State<'_, SharedState>) -> CmdResult<Vec<DisplayInfo>> {
    Ok(state.backend.displays()?)
}
