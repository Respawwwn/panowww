//! The Capture zone picker windows (spec §11.4): one transparent,
//! borderless window per display, covering it above the menu bar, so the
//! page draws over the live, dimmed display (`ui/picker.html`). On macOS
//! each is a `tauri-nspanel` panel that can take the keyboard. This module
//! owns the pick in progress: opening the windows, answering each page's
//! setup request, taking the zone one of them saves or the cancel, closing
//! them all. `pick_zone` waits on the reply channel.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use panowww_core::ports::{DisplayInfo, Rect, Zone};
use panowww_store::settings::Appearance;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::backend::SharedBackend;
use crate::dto::PickerSetup;
use crate::error::{AppError, AppErrorKind};
use crate::overlay::SharedOverlay;

/// Labels of the picker windows: `picker-1`, `picker-2`, …
pub const LABEL_PREFIX: &str = "picker-";

/// One picker window and the display it covers.
#[derive(Debug, Clone)]
struct PickerWindow {
    label: String,
    display: DisplayInfo,
    index: u32,
}

struct Pending {
    game_id: Uuid,
    windows: Vec<PickerWindow>,
    reply: Sender<Option<Zone>>,
}

/// The pick in progress, at most one.
#[derive(Default)]
pub struct Picker {
    pending: Mutex<Option<Pending>>,
}

impl std::fmt::Debug for Picker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Picker").finish_non_exhaustive()
    }
}

impl Picker {
    /// Opens one window per display and hands back the channel the result
    /// arrives on: `Some(zone)` once saved, `None` on cancel.
    pub fn open<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        backend: &SharedBackend,
        game_id: Uuid,
        current: Option<&Zone>,
    ) -> Result<Receiver<Option<Zone>>, AppError> {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_some() {
            return Err(AppError::new(
                AppErrorKind::Busy,
                "The Capture zone picker is already open.",
            ));
        }
        let displays = backend.displays()?;
        if displays.is_empty() {
            return Err(AppError::new(AppErrorKind::Internal, "No display found."));
        }
        // The window of the display that holds the zone gets the focus,
        // else the first one.
        let focus = current
            .and_then(|z| displays.iter().position(|d| d.id == z.display_id))
            .unwrap_or(0);
        // Window creation and the panel conversion are AppKit work: on the
        // main thread, with the result handed back.
        let (done_tx, done_rx) = channel::<Result<Vec<PickerWindow>, AppError>>();
        let handle = app.clone();
        let origins: Vec<(DisplayInfo, (f64, f64))> = displays
            .into_iter()
            .map(|d| {
                let origin = backend.display_origin(&d.id).unwrap_or((0.0, 0.0));
                (d, origin)
            })
            .collect();
        app.run_on_main_thread(move || {
            let _ = done_tx.send(build_windows(&handle, origins, focus));
        })
        .map_err(|e| AppError::internal(format!("picker windows: {e}")))?;
        let windows = done_rx
            .recv()
            .map_err(|_| AppError::internal("the picker windows were not built"))??;
        let (reply, rx) = channel();
        *pending = Some(Pending {
            game_id,
            windows,
            reply,
        });
        Ok(rx)
    }

    /// What the window `label` shows.
    pub fn setup(
        &self,
        label: &str,
        current: Option<&Zone>,
        appearance: Appearance,
    ) -> Result<PickerSetup, AppError> {
        let window = self.window(label)?;
        let zone = current
            .filter(|z| z.display_id == window.display.id)
            .map(Zone::rect);
        Ok(PickerSetup {
            display: window.display,
            display_index: window.index,
            zone,
            appearance,
        })
    }

    /// The zone record of `rect` drawn in the window `label` (spec §6).
    pub fn zone_for(&self, label: &str, rect: Rect) -> Result<Zone, AppError> {
        let window = self.window(label)?;
        let d = &window.display;
        let inside = rect.w > 0
            && rect.h > 0
            && rect.x >= 0
            && rect.y >= 0
            && u64::try_from(rect.x).unwrap_or(0) + u64::from(rect.w) <= u64::from(d.width)
            && u64::try_from(rect.y).unwrap_or(0) + u64::from(rect.h) <= u64::from(d.height);
        if !inside {
            return Err(AppError::new(
                AppErrorKind::Validation,
                "The zone must lie inside the display.",
            ));
        }
        Ok(Zone {
            display_id: d.id.clone(),
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            display_w: d.width,
            display_h: d.height,
            scale: d.scale,
        })
    }

    /// The Game of the pick in progress.
    pub fn game_id(&self) -> Option<Uuid> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|p| p.game_id)
    }

    /// Ends the pick: every window closes, the overlay frame goes, and
    /// `pick_zone` gets `result`. A no-op when nothing is pending.
    pub fn finish<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        overlay: &SharedOverlay,
        result: Option<Zone>,
    ) {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let Some(pending) = pending else {
            return;
        };
        overlay.hide();
        for w in &pending.windows {
            close_window(app, &w.label);
        }
        let _ = pending.reply.send(result);
    }

    /// A picker window closed by the OS or the user cancels the pick.
    pub fn window_gone<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        overlay: &SharedOverlay,
        label: &str,
    ) {
        let ours = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|p| p.windows.iter().any(|w| w.label == label));
        if ours {
            tracing::info!(label, "picker window closed: pick cancelled");
            self.finish(app, overlay, None);
        }
    }

    fn window(&self, label: &str) -> Result<PickerWindow, AppError> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|p| p.windows.iter().find(|w| w.label == label))
            .cloned()
            .ok_or_else(|| AppError::new(AppErrorKind::NotFound, "No pick in progress."))
    }
}

/// One window per display, built on the main thread. On failure the ones
/// already open are closed again.
fn build_windows<R: Runtime>(
    app: &AppHandle<R>,
    displays: Vec<(DisplayInfo, (f64, f64))>,
    focus: usize,
) -> Result<Vec<PickerWindow>, AppError> {
    let mut windows: Vec<PickerWindow> = Vec::with_capacity(displays.len());
    for (i, (display, (x, y))) in displays.into_iter().enumerate() {
        let index = u32::try_from(i + 1).unwrap_or(u32::MAX);
        let label = format!("{LABEL_PREFIX}{index}");
        let built = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("picker.html".into()))
            .title("Capture zone")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .position(x, y)
            .inner_size(f64::from(display.width), f64::from(display.height))
            .focused(i == focus)
            .initialization_script(format!("window.__PANOWWW_PICKER__ = {label:?};"))
            .build();
        let window = match built {
            Ok(window) => window,
            Err(error) => {
                for w in &windows {
                    close_now(app, &w.label);
                }
                return Err(AppError::new(
                    AppErrorKind::Internal,
                    format!("The picker window could not open: {error}"),
                ));
            }
        };
        #[cfg(target_os = "macos")]
        if let Err(error) = macos::raise(app, &window, i == focus) {
            tracing::warn!(%error, label, "picker window stays a plain window");
        }
        // The keyboard (Esc, the fields) goes to the focused window's page.
        if i == focus
            && let Err(error) = window.set_focus()
        {
            tracing::warn!(%error, label, "picker window not focused");
        }
        windows.push(PickerWindow {
            label,
            display,
            index,
        });
    }
    Ok(windows)
}

/// Closes a picker window from any thread: the panel is handed back to
/// Tauri and destroyed on the main thread.
fn close_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let handle = app.clone();
    let label = label.to_owned();
    if let Err(error) = app.run_on_main_thread(move || close_now(&handle, &label)) {
        tracing::warn!(%error, "picker window not closed");
    }
}

/// Main thread only.
fn close_now<R: Runtime>(app: &AppHandle<R>, label: &str) {
    #[cfg(target_os = "macos")]
    macos::release(app, label);
    if let Some(window) = app.get_webview_window(label)
        && let Err(error) = window.destroy()
    {
        tracing::warn!(%error, label, "picker window not closed");
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use tauri::{AppHandle, Runtime, WebviewWindow};
    use tauri_nspanel::objc2_app_kit::NSWindowCollectionBehavior;
    use tauri_nspanel::{ManagerExt, PanelLevel, WebviewWindowExt, tauri_panel};

    tauri_panel! {
        panel!(PickerPanel {
            config: {
                can_become_key_window: true,
                can_become_main_window: false,
                is_floating_panel: true,
                hides_on_deactivate: false
            }
        })
    }

    /// Makes the window a panel above the menu bar and the Dock, on every
    /// Space, keyboard allowed (Esc, the fields); the focused one comes
    /// front as key.
    pub fn raise<R: Runtime>(
        app: &AppHandle<R>,
        window: &WebviewWindow<R>,
        focus: bool,
    ) -> tauri::Result<()> {
        let panel = window.to_panel::<PickerPanel<R>>()?;
        panel.set_collection_behavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
        panel.set_level(PanelLevel::Status.value());
        panel.set_has_shadow(false);
        panel.set_opaque(false);
        panel.set_hides_on_deactivate(false);
        if focus {
            panel.show_and_make_key();
        } else {
            panel.order_front_regardless();
        }
        let _ = app;
        Ok(())
    }

    /// Gives the panel back to Tauri before the window is destroyed.
    pub fn release<R: Runtime>(app: &AppHandle<R>, label: &str) {
        if let Ok(panel) = app.get_webview_panel(label) {
            let _ = panel.to_window();
        }
    }
}
