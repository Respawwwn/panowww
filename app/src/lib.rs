//! Thin Tauri shell of Panowww (spec §4.1): the commands the UI calls, the
//! single Runner owner, countdown, pre-flight, overlay, global shortcuts,
//! windows, onboarding aggregation, update check and the one `AppError`
//! shape the UI sees. Window glue only; every decision lives in `core`.

mod commands;
mod logging;

/// Builds the Tauri app and runs it until the last window closes.
///
/// # Panics
///
/// When the Tauri runtime cannot start at all (no webview, no display):
/// there is nothing to show the error in.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            logging::init(app)?;
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Panowww starting");
            Ok(())
        })
        .invoke_handler(commands::handler())
        .run(context())
        .expect("the Tauri runtime failed to start");
}

/// The embedded `tauri.conf.json`, assets and icons. `generate_context!` may
/// appear once per crate, so the tests share this one.
fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}
