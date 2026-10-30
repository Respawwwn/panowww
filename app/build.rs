/// Every `#[tauri::command]` of the app, so tauri-build generates its
/// `allow-<command>` permission for `capabilities/default.json`.
const COMMANDS: &[&str] = &["ping"];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri-build fails only on a broken tauri.conf.json or capability");
}
