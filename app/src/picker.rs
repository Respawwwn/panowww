//! The Capture zone picker windows (spec §11.4): one borderless window per
//! display, covering it, drawn by `ui/picker.html`. This module owns the
//! pick in progress: opening the windows, answering each page's setup
//! request, taking the zone one of them saves or the cancel, closing them
//! all. `pick_zone` waits on the reply channel.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use panowww_core::ports::{DisplayInfo, Rect, Zone};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::backend::SharedBackend;
use crate::dto::PickerSetup;
use crate::error::{AppError, AppErrorKind};
use crate::overlay::SharedOverlay;
use crate::runner::data_url;

/// Labels of the picker windows: `picker-1`, `picker-2`, …
pub const LABEL_PREFIX: &str = "picker-";
/// Longer side of the screenshot behind the picker, in pixels.
pub const IMAGE_LONG_SIDE: u32 = 2560;

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
        let mut windows: Vec<PickerWindow> = Vec::with_capacity(displays.len());
        for (i, display) in displays.into_iter().enumerate() {
            let index = u32::try_from(i + 1).unwrap_or(u32::MAX);
            let label = format!("{LABEL_PREFIX}{index}");
            let (x, y) = backend.display_origin(&display.id).unwrap_or((0.0, 0.0));
            let built =
                WebviewWindowBuilder::new(app, &label, WebviewUrl::App("picker.html".into()))
                    .title("Capture zone")
                    .decorations(false)
                    .resizable(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .position(x, y)
                    .inner_size(f64::from(display.width), f64::from(display.height))
                    .focused(i == focus)
                    .initialization_script(format!("window.__PANOWWW_PICKER__ = {label:?};"))
                    .build();
            if let Err(error) = built {
                for w in &windows {
                    close_window(app, &w.label);
                }
                return Err(AppError::new(
                    AppErrorKind::Internal,
                    format!("The picker window could not open: {error}"),
                ));
            }
            windows.push(PickerWindow {
                label,
                display,
                index,
            });
        }
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
        backend: &SharedBackend,
        label: &str,
        current: Option<&Zone>,
    ) -> Result<PickerSetup, AppError> {
        let window = self.window(label)?;
        let image = match backend.display_image(&window.display.id, IMAGE_LONG_SIDE) {
            Ok(jpeg) => Some(data_url("image/jpeg", &jpeg.0)),
            Err(error) => {
                tracing::warn!(%error, label, "picker screenshot failed");
                None
            }
        };
        let zone = current
            .filter(|z| z.display_id == window.display.id)
            .map(Zone::rect);
        Ok(PickerSetup {
            display: window.display,
            display_index: window.index,
            image,
            zone,
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

fn close_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if let Some(window) = app.get_webview_window(label)
        && let Err(error) = window.destroy()
    {
        tracing::warn!(%error, label, "picker window not closed");
    }
}
