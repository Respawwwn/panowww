//! The Tauri commands: the only surface the UI calls. Every type crossing
//! this boundary derives `TS` and is exported to `ui/src/lib/generated/`.
//! Each command is listed in `build.rs` and allowed in
//! `capabilities/default.json`.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands take their arguments by value"
)]

pub mod games;
pub mod onboarding;
pub mod run;
pub mod sessions;
pub mod settings;
pub mod zone;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The answer to `ping`: proves the IPC and shares the app version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Pong {
    pub app: String,
    pub version: String,
}

#[tauri::command]
pub fn ping() -> Pong {
    Pong {
        app: "Panowww".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

/// Every command, registered on the builder.
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        ping,
        games::list_games,
        games::create_game,
        games::save_game,
        games::duplicate_game,
        games::delete_game,
        games::share_game,
        games::import_game,
        games::import_replace,
        games::import_keep_both,
        games::delete_game_file,
        games::estimate_session,
        games::adjust_step,
        games::select_game,
        zone::get_zone,
        zone::set_zone,
        zone::pick_zone,
        zone::show_zone,
        zone::overlay_state,
        zone::list_displays,
        run::start_run,
        run::stop_run,
        run::set_preview,
        run::runner_phase,
        sessions::list_sessions,
        sessions::session_size,
        sessions::zip_session,
        sessions::delete_session,
        sessions::session_thumbnail,
        sessions::reveal_path,
        settings::get_settings,
        settings::save_settings,
        settings::choose_sessions_folder,
        settings::reveal_logs,
        settings::apply_shortcuts,
        onboarding::permissions,
        onboarding::request_permission,
        onboarding::open_system_settings,
        onboarding::open_url,
        onboarding::check_for_updates,
    ]
}
