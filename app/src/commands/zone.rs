//! The Capture zone of a Game on this machine, and the displays.

use panowww_core::ports::{DisplayInfo, Zone};
use panowww_store::local::zone_status;
use tauri::State;
use uuid::Uuid;

use crate::dto::ZoneState;
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

/// Opens the picker windows (issue 13). Until then: not available.
#[tauri::command]
pub fn pick_zone(_state: State<'_, SharedState>, _game_id: Uuid) -> CmdResult<ZoneState> {
    Err(AppError::new(
        AppErrorKind::Internal,
        "The Capture zone picker arrives with issue 13.",
    ))
}

/// Draws or hides the overlay frame without a run (issue 12). Until then
/// a no-op.
#[tauri::command]
pub fn show_zone(_state: State<'_, SharedState>, game_id: Uuid, on: bool) {
    tracing::info!(%game_id, on, "show_zone: overlay arrives with issue 12");
}

#[tauri::command]
pub fn list_displays(state: State<'_, SharedState>) -> CmdResult<Vec<DisplayInfo>> {
    Ok(state.backend.displays()?)
}
