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
    "picker_drawing",
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

/// The `<trustInfo>` that makes Windows ask for elevation at launch
/// (spec §8.4, §14): UIPI drops input into games that run elevated.
const REQUIRE_ADMINISTRATOR: &str = r#"  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#;

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    let mut attributes = tauri_build::Attributes::new().app_manifest(manifest);
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        attributes = attributes.windows_attributes(windows_attributes());
    }
    tauri_build::try_build(attributes)
        .expect("tauri-build fails only on a broken tauri.conf.json or capability");
}

/// Two Windows manifests from one file. The app binary gets the plain
/// manifest plus the elevation request, as a resource compiled by
/// tauri-build (its resource reaches the app binary only). Every other
/// binary of the crate, the unit tests first, gets the plain manifest
/// embedded by the linker: without its Common-Controls v6 dependency they
/// fail to start (`TaskDialogIndirect`), and with the elevation request
/// `just test` would ask for UAC. The linker arguments are one manifest
/// input per binary, never two: merging two inputs needs `mt.exe`, which
/// is not on the PATH of a plain shell.
fn windows_attributes() -> tauri_build::WindowsAttributes {
    let plain = std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"),
    )
    .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", plain.display());
    let text = std::fs::read_to_string(&plain).expect("app/windows-app-manifest.xml is readable");
    let elevated = text.replacen("</assembly>", REQUIRE_ADMINISTRATOR, 1);
    assert!(
        elevated.contains("requireAdministrator"),
        "the manifest must end with </assembly>"
    );
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", plain.display());
    // Later arguments win: the app binary keeps only the resource manifest.
    // The linker then notes the ignored input (LNK4075); harmless.
    println!("cargo:rustc-link-arg-bins=/MANIFEST:NO");
    tauri_build::WindowsAttributes::new().app_manifest(elevated)
}
