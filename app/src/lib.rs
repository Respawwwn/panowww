//! Thin Tauri shell of Panowww (spec §4.1): the commands the UI calls, the
//! single Runner owner, countdown, pre-flight, global shortcuts, windows,
//! onboarding aggregation, update check and the one `AppError` shape the
//! UI sees. Window glue only; every decision lives in `core`.

mod backend;
mod commands;
mod dto;
mod error;
mod logging;
mod runner;
mod shortcuts;
mod state;
mod update;

use std::sync::{Arc, Mutex, RwLock};

use panowww_store::AppPaths;
use panowww_store::games::GamesFolder;
use panowww_store::local::LocalGames;
use panowww_store::sessions::mark_stale_running;
use panowww_store::settings::Settings;
use panowww_store::sink::SystemWallClock;
use tauri::{Emitter, Manager};

use crate::backend::SystemBackend;
use crate::runner::Runner;
use crate::state::{AppState, SharedState};

/// The window event that carries every `RunEvent` of a run.
pub const RUN_EVENT: &str = "run";

/// Builds the Tauri app and runs it until the last window closes.
///
/// # Panics
///
/// When the Tauri runtime cannot start at all (no webview, no display):
/// there is nothing to show the error in.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            logging::init(app)?;
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Panowww starting");
            let state = build_state(app)?;
            app.manage(state);
            if let Err(error) = shortcuts::register(app.handle()) {
                tracing::warn!(%error, "global shortcuts not registered");
            }
            Ok(())
        })
        .invoke_handler(commands::handler())
        .build(context())
        .expect("the Tauri runtime failed to start")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // A running Session ends as interrupted, with its Shots kept.
                app.state::<SharedState>().runner.shutdown();
            }
        });
}

/// Loads everything from the data folder and wires the Runner to the
/// window's `run` event.
fn build_state(app: &tauri::App) -> tauri::Result<SharedState> {
    let paths = AppPaths::new(app.path().app_data_dir()?);
    paths
        .ensure()
        .map_err(|e| tauri::Error::Io(std::io::Error::other(e)))?;
    let settings = Settings::load_or_default(&paths);
    let local = LocalGames::load(&paths).unwrap_or_else(|error| {
        tracing::warn!(%error, "local-games.json unreadable, starting empty");
        LocalGames::default()
    });
    match mark_stale_running(&settings.sessions_folder, &SystemWallClock) {
        Ok(0) => {}
        Ok(n) => tracing::info!(n, "stale running Sessions marked interrupted"),
        Err(error) => tracing::warn!(%error, "could not scan the Sessions folder"),
    }
    let backend: backend::SharedBackend = Arc::new(SystemBackend);
    let handle = app.handle().clone();
    let runner = Runner::new(
        Arc::clone(&backend),
        Arc::new(move |event| {
            if let Err(error) = handle.emit(RUN_EVENT, &event) {
                tracing::warn!(%error, "run event not delivered");
            }
        }),
    );
    Ok(Arc::new(AppState {
        games: GamesFolder::new(&paths),
        paths,
        settings: RwLock::new(settings),
        local: RwLock::new(local),
        selected: Mutex::new(None),
        runner,
        backend,
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }))
}

/// The embedded `tauri.conf.json`, assets and icons. `generate_context!` may
/// appear once per crate, so the tests share this one.
fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

#[cfg(test)]
mod tests {
    use tauri::WebviewWindowBuilder;
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
    use tauri::webview::InvokeRequest;

    use super::*;

    fn app() -> tauri::App<MockRuntime> {
        mock_builder()
            .invoke_handler(commands::handler())
            .build(context())
            .expect("mock app builds")
    }

    #[test]
    fn ping_answers_with_the_workspace_version() {
        let app = app();
        let webview = WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("mock window builds");
        let response = get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "ping".to_owned(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                // The app origin differs per OS, as the ACL expects.
                url: if cfg!(windows) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: tauri::http::HeaderMap::default(),
                invoke_key: INVOKE_KEY.to_owned(),
            },
        )
        .expect("ping succeeds");
        let pong: commands::Pong = response.deserialize().unwrap();
        assert_eq!(pong.app, "Panowww");
        assert_eq!(pong.version, env!("CARGO_PKG_VERSION"));
    }
}
