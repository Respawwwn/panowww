/// Every `#[tauri::command]` of the app, so tauri-build generates its
/// `allow-<command>` permission for `capabilities/default.json`.
const COMMANDS: &[&str] = &[
    "ping",
    "list_games",
    "reorder_games",
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
    "set_shortcuts_enabled",
    "permissions",
    "request_permission",
    "open_system_settings",
    "open_url",
    "check_for_updates",
];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    let mut attributes = tauri_build::Attributes::new().app_manifest(manifest);
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // tauri-build embeds its Windows manifest as a resource of the app
        // binary only; the test binaries then miss the Common-Controls v6
        // dependency and fail to start. The linker embeds ours in every
        // binary of the crate instead.
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        embed_windows_manifest();
    }
    tauri_build::try_build(attributes)
        .expect("tauri-build fails only on a broken tauri.conf.json or capability");
}

fn embed_windows_manifest() {
    let manifest = std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"),
    )
    .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}
