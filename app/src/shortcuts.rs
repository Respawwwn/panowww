//! The two global shortcuts (spec §10): Start always runs a Session of
//! the selected Game with the 3 s countdown; Stop cancels or stops.
//! Registered at launch and again when Settings change; a combo that
//! fails to register is reported to Settings.

use std::str::FromStr;

use panowww_store::settings::{Modifier, Shortcut as Stored, Shortcuts};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::commands::run::start_selected_session;
use crate::error::{AppError, AppErrorKind};
use crate::runner::COUNTDOWN_SHORTCUT;
use crate::state::SharedState;

/// A stored shortcut as the plugin wants it.
pub fn to_plugin(stored: &Stored) -> Result<Shortcut, AppError> {
    let code = Code::from_str(&stored.code).map_err(|_| {
        AppError::new(
            AppErrorKind::Internal,
            format!("{} is not a key.", stored.code),
        )
    })?;
    let mut modifiers = Modifiers::empty();
    for m in &stored.modifiers {
        modifiers |= match m {
            Modifier::Ctrl => Modifiers::CONTROL,
            Modifier::Shift => Modifiers::SHIFT,
            Modifier::Alt => Modifiers::ALT,
            Modifier::Meta => Modifiers::SUPER,
        };
    }
    Ok(Shortcut::new(Some(modifiers), code))
}

/// Replaces every registered shortcut with the ones in Settings.
pub fn register<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let settings = app.state::<SharedState>().settings();
    register_shortcuts(app, &settings.shortcuts)
}

/// Replaces every registered shortcut with `shortcuts`, before they are
/// saved: a combo the OS refuses leaves the settings as they were.
pub fn register_shortcuts<R: tauri::Runtime>(
    app: &AppHandle<R>,
    shortcuts: &Shortcuts,
) -> Result<(), AppError> {
    if shortcuts.start == shortcuts.stop {
        return Err(AppError::new(
            AppErrorKind::Validation,
            "Start and Stop need two different shortcuts.",
        ));
    }
    let plugin = app.global_shortcut();
    plugin
        .unregister_all()
        .map_err(|e| AppError::internal(format!("unregistering shortcuts: {e}")))?;

    let start = to_plugin(&shortcuts.start)?;
    plugin
        .on_shortcut(start, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("start shortcut");
                start_selected_session(app, COUNTDOWN_SHORTCUT);
            }
        })
        .map_err(|e| registration_error(&shortcuts.start, &e))?;

    let stop = to_plugin(&shortcuts.stop)?;
    plugin
        .on_shortcut(stop, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("stop shortcut");
                app.state::<SharedState>().runner.stop();
            }
        })
        .map_err(|e| registration_error(&shortcuts.stop, &e))?;
    Ok(())
}

/// Settings open: nothing fires while a combo is recorded. Closed: the
/// saved ones are back.
pub fn set_enabled<R: tauri::Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<(), AppError> {
    if enabled {
        register(app)
    } else {
        app.global_shortcut()
            .unregister_all()
            .map_err(|e| AppError::internal(format!("unregistering shortcuts: {e}")))
    }
}

fn registration_error(shortcut: &Stored, error: &impl std::fmt::Display) -> AppError {
    let combo = shortcut
        .modifiers
        .iter()
        .map(|m| format!("{m:?}"))
        .chain(std::iter::once(shortcut.code.clone()))
        .collect::<Vec<_>>()
        .join("+");
    AppError::new(
        AppErrorKind::Internal,
        format!("{combo} could not be registered: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_shortcuts_convert() {
        let start = Stored::new([Modifier::Ctrl, Modifier::Shift], "Digit8");
        let shortcut = to_plugin(&start).unwrap();
        assert_eq!(shortcut.key, Code::Digit8);
        assert!(
            shortcut.mods.contains(Modifiers::CONTROL) && shortcut.mods.contains(Modifiers::SHIFT)
        );
        assert!(to_plugin(&Stored::new([], "NotAKey")).is_err());
    }
}
