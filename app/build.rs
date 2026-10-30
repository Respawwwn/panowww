/// Every `#[tauri::command]` of the app, so tauri-build generates its
/// `allow-<command>` permission for `capabilities/default.json`.
const COMMANDS: &[&str] = &[
    "ping",
    "list_games",
    "create_game",
    "save_game",
    "duplicate_game",
    "delete_game",
    "share_game",
    "import_game",
    "import_replace",
    "import_keep_both",
    "delete_game_file",
    "estimate_session",
    "adjust_step",
    "select_game",
    "get_zone",
    "set_zone",
    "pick_zone",
    "show_zone",
    "overlay_state",
    "picker_setup",
    "picker_save",
    "picker_cancel",
    "picker_frame",
    "list_displays",
    "start_run",
    "stop_run",
    "set_preview",
    "runner_phase",
    "list_sessions",
    "session_size",
    "zip_session",
    "delete_session",
    "session_thumbnail",
    "reveal_path",
    "get_settings",
    "save_settings",
    "choose_sessions_folder",
    "reveal_logs",
    "apply_shortcuts",
    "permissions",
    "request_permission",
    "open_system_settings",
    "open_url",
    "check_for_updates",
];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri-build fails only on a broken tauri.conf.json or capability");
}
