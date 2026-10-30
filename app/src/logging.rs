//! `tracing` setup: a daily-rotating file in Tauri's logs folder (Settings →
//! *Reveal logs*), plus the terminal in debug builds.

use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, fmt};

/// File name prefix; the appender adds the date.
const LOG_FILE: &str = "panowww.log";

/// Keeps the background writer alive for the whole app run.
pub struct LogGuard(#[allow(dead_code, reason = "held for its Drop")] WorkerGuard);

/// Installs the global subscriber and parks its guard in the app state.
pub fn init(app: &tauri::App) -> tauri::Result<()> {
    let logs = app.path().app_log_dir()?;
    std::fs::create_dir_all(&logs)?;
    let (file, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&logs, LOG_FILE));
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,panowww=debug"));
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(file);
    let registry = tracing_subscriber::registry().with(filter).with(file_layer);
    if cfg!(debug_assertions) {
        registry
            .with(fmt::layer().with_writer(std::io::stderr))
            .init();
    } else {
        registry.init();
    }
    app.manage(LogGuard(guard));
    Ok(())
}
