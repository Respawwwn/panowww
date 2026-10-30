//! Thin Tauri shell of Panowww (spec §4.1): the commands the UI calls, the
//! single Runner owner, countdown, pre-flight, global shortcuts, windows,
//! onboarding aggregation, update check and the one `AppError` shape the
//! UI sees. Window glue only; every decision lives in `core`.

mod backend;
mod commands;
mod dto;
mod error;
mod logging;
mod overlay;
mod picker;
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
use crate::overlay::WindowOverlay;
use crate::runner::Runner;
use crate::state::{AppState, SharedState};

/// The app window's label in `tauri.conf.json`.
pub const MAIN_WINDOW: &str = "main";
/// The window event that carries every `RunEvent` of a run.
pub const RUN_EVENT: &str = "run";
/// The window event sent when a file of the Games folder changed on disk.
pub const GAMES_CHANGED_EVENT: &str = "games-changed";

/// Builds the Tauri app and runs it until the last window closes.
///
/// # Panics
///
/// When the Tauri runtime cannot start at all (no webview, no display):
/// there is nothing to show the error in.
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());
    builder
        .setup(|app| {
            logging::init(app)?;
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Panowww starting");
            let state = build_state(app)?;
            app.manage(state);
            if let Err(error) = overlay::init(app.handle()) {
                tracing::warn!(%error, "overlay window not prepared");
            }
            if let Err(error) = shortcuts::register(app.handle()) {
                tracing::warn!(%error, "global shortcuts not registered");
            }
            Ok(())
        })
        .invoke_handler(commands::handler())
        .build(context())
        .expect("the Tauri runtime failed to start")
        .run(|app, event| match event {
            tauri::RunEvent::ExitRequested { .. } => {
                // A running Session ends as interrupted, with its Shots kept.
                app.state::<SharedState>().runner.shutdown();
            }
            // The hidden overlay window would keep the app alive.
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } if label == MAIN_WINDOW => {
                app.state::<SharedState>().runner.shutdown();
                app.exit(0);
            }
            // A picker window closed by other means cancels the pick.
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } if label.starts_with(picker::LABEL_PREFIX) => {
                let state = app.state::<SharedState>();
                state.picker.window_gone(app, &state.overlay, &label);
            }
            _ => {}
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
    let overlay: overlay::SharedOverlay = Arc::new(WindowOverlay::new(
        app.handle().clone(),
        Arc::clone(&backend),
    ));
    let handle = app.handle().clone();
    let runner = Runner::new(
        Arc::clone(&backend),
        Arc::new(move |event| {
            if let Err(error) = handle.emit(RUN_EVENT, &event) {
                tracing::warn!(%error, "run event not delivered");
            }
        }),
        Arc::clone(&overlay),
    );
    // A Game edited by hand, or a damaged file repaired, is re-read: the
    // window re-lists the folder on `games-changed` (spec §5).
    let folder_handle = app.handle().clone();
    let watcher = panowww_store::watcher::watch(&paths.games_dir(), move |event| {
        tracing::debug!(?event, "Games folder changed");
        if let Err(error) = folder_handle.emit(GAMES_CHANGED_EVENT, ()) {
            tracing::warn!(%error, "games-changed not delivered");
        }
    })
    .map_err(|error| {
        tracing::warn!(%error, "Games folder not watched");
        error
    })
    .ok();
    Ok(Arc::new(AppState {
        games: GamesFolder::new(&paths),
        paths,
        settings: RwLock::new(settings),
        local: RwLock::new(local),
        selected: Mutex::new(None),
        runner,
        backend,
        overlay,
        picker: picker::Picker::default(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        watcher: Mutex::new(watcher),
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
