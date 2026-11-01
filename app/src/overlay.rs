//! The overlay (spec §10): one transparent, click-through, non-activating
//! window over the zone's display, drawn by `ui/overlay.html`. This module
//! owns it: the `Overlay` port the Runner and the Setup call, its
//! implementation over the `overlay` webview (a `tauri-nspanel` panel on
//! macOS, so the game keeps the focus, full-screen Spaces included) and the
//! one-time setup. Every AppKit call runs on the main thread: calling it
//! from the Runner thread traps.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use panowww_core::ports::Zone;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::backend::SharedBackend;
use crate::dto::{OverlayState, OverlayView, OverlayZone};

/// The window's label in `tauri.conf.json`.
pub const OVERLAY_WINDOW: &str = "overlay";
/// The window event carrying each `OverlayState` to the overlay page.
pub const OVERLAY_EVENT: &str = "overlay";
/// How long the end pill stays (spec §10: ≈ 2 s).
pub const END_PILL: Duration = Duration::from_secs(2);

/// What a run and the Setup may draw over the game (spec §4.2).
pub trait Overlay: Send + Sync {
    /// The frame alone: Show zone, and the start of a run.
    fn show_frame(&self, zone: &Zone);
    /// The frame with the digit centred.
    fn show_countdown(&self, zone: &Zone, seconds_left: u32);
    /// The progress bar and the `taken / expected` pill of the run in
    /// progress, inside the frame shown last.
    fn progress(&self, taken: u32, expected: u32);
    /// `Done · n Shots` or `Stopped · k / n`, hidden after `END_PILL`.
    fn end_pill(&self, stopped: bool, taken: u32, expected: u32);
    fn hide(&self);
    /// What is on screen, for the page when it loads; `None` when hidden.
    fn current(&self) -> Option<OverlayState>;
}

pub type SharedOverlay = Arc<dyn Overlay>;

/// The `overlay` webview window, moved over the zone's display before
/// each show.
pub struct WindowOverlay<R: Runtime> {
    app: AppHandle<R>,
    backend: SharedBackend,
    /// The zone of the frame on screen; `progress` and the pill reuse it.
    zone: Arc<Mutex<Option<Zone>>>,
    /// What the page shows; served to it when it (re)loads.
    last: Arc<Mutex<Option<OverlayState>>>,
    /// Bumped by every call, so a pending end-pill hide gives up when
    /// something newer was drawn.
    generation: Arc<AtomicU64>,
}

impl<R: Runtime> WindowOverlay<R> {
    pub fn new(app: AppHandle<R>, backend: SharedBackend) -> Self {
        Self {
            app,
            backend,
            zone: Arc::new(Mutex::new(None)),
            last: Arc::new(Mutex::new(None)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    fn bump(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Moves the window over the zone's display, sends the state and
    /// orders the window front without activating anything.
    fn present(&self, zone: &Zone, view: OverlayView) {
        self.bump();
        *self
            .zone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(zone.clone());
        let state = OverlayState {
            zone: OverlayZone::from(zone),
            view,
        };
        *self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(state.clone());
        let origin = self
            .backend
            .display_origin(&zone.display_id)
            .unwrap_or((0.0, 0.0));
        let size = (f64::from(zone.display_w), f64::from(zone.display_h));
        let app = self.app.clone();
        let result = self.app.run_on_main_thread(move || {
            let Some(window) = app.get_webview_window(OVERLAY_WINDOW) else {
                tracing::warn!("no overlay window");
                return;
            };
            place(&window, origin, size);
            if let Err(error) = app.emit_to(OVERLAY_WINDOW, OVERLAY_EVENT, &state) {
                tracing::warn!(%error, "overlay state not delivered");
            }
            order_front(&app, &window);
        });
        if let Err(error) = result {
            tracing::warn!(%error, "overlay not shown");
        }
    }
}

impl<R: Runtime> Overlay for WindowOverlay<R> {
    fn show_frame(&self, zone: &Zone) {
        self.present(zone, OverlayView::Frame);
    }

    fn show_countdown(&self, zone: &Zone, seconds_left: u32) {
        self.present(zone, OverlayView::Countdown { seconds_left });
    }

    fn progress(&self, taken: u32, expected: u32) {
        let zone = self
            .zone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(zone) = zone {
            self.present(&zone, OverlayView::Progress { taken, expected });
        }
    }

    fn end_pill(&self, stopped: bool, taken: u32, expected: u32) {
        let zone = self
            .zone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let Some(zone) = zone else {
            return;
        };
        self.present(
            &zone,
            OverlayView::End {
                stopped,
                taken,
                expected,
            },
        );
        let shown = self.generation.load(Ordering::SeqCst);
        let generation = Arc::clone(&self.generation);
        let current = Arc::clone(&self.zone);
        let last = Arc::clone(&self.last);
        let app = self.app.clone();
        std::thread::Builder::new()
            .name("panowww-end-pill".to_owned())
            .spawn(move || {
                std::thread::sleep(END_PILL);
                // Something newer on screen keeps the overlay.
                if generation
                    .compare_exchange(shown, shown + 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    *current
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
                    *last
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
                    hide_window(&app);
                }
            })
            .expect("spawning the end-pill thread");
    }

    fn hide(&self) {
        self.bump();
        *self
            .zone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        hide_window(&self.app);
    }

    fn current(&self) -> Option<OverlayState> {
        self.last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// Covers the zone's display: `origin` and `size` are display coordinates
/// as the backend reports them, points on macOS and physical pixels on
/// Windows (spec §4.2), so each OS gets the matching window API.
fn place<R: Runtime>(window: &WebviewWindow<R>, origin: (f64, f64), size: (f64, f64)) {
    #[cfg(target_os = "windows")]
    let (position, dimensions) = (
        tauri::Position::Physical(tauri::PhysicalPosition::new(px(origin.0), px(origin.1))),
        tauri::Size::Physical(tauri::PhysicalSize::new(
            px(size.0).unsigned_abs(),
            px(size.1).unsigned_abs(),
        )),
    );
    #[cfg(not(target_os = "windows"))]
    let (position, dimensions) = (
        tauri::Position::Logical(tauri::LogicalPosition::new(origin.0, origin.1)),
        tauri::Size::Logical(tauri::LogicalSize::new(size.0, size.1)),
    );
    if let Err(error) = window.set_position(position) {
        tracing::warn!(%error, "overlay not moved");
    }
    if let Err(error) = window.set_size(dimensions) {
        tracing::warn!(%error, "overlay not sized");
    }
}

/// Rounds a physical coordinate to whole pixels.
#[cfg(target_os = "windows")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "display coordinates are far inside i32 once rounded"
)]
fn px(value: f64) -> i32 {
    value.round() as i32
}

/// Makes the hidden `overlay` window click-through and, on macOS, a
/// non-activating panel that joins every Space and stays above the game.
/// Called once from Tauri's `setup`, on the main thread.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(OVERLAY_WINDOW)
        .ok_or(tauri::Error::WindowNotFound)?;
    window.set_ignore_cursor_events(true)?;
    crate::backend::hide_from_capture(&window);
    #[cfg(target_os = "macos")]
    macos::make_panel(&window)?;
    Ok(())
}

fn hide_window<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    let result = app.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        {
            use tauri_nspanel::ManagerExt;
            if let Ok(panel) = handle.get_webview_panel(OVERLAY_WINDOW) {
                panel.hide();
                return;
            }
        }
        if let Some(window) = handle.get_webview_window(OVERLAY_WINDOW)
            && let Err(error) = window.hide()
        {
            tracing::warn!(%error, "overlay not hidden");
        }
    });
    if let Err(error) = result {
        tracing::warn!(%error, "overlay not hidden");
    }
}

/// Main thread only.
#[cfg(target_os = "macos")]
fn order_front<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    use tauri_nspanel::ManagerExt;
    match app.get_webview_panel(OVERLAY_WINDOW) {
        Ok(panel) => panel.order_front_regardless(),
        Err(_) => {
            // The panel conversion failed at setup: a plain window, which
            // may take the focus. Logged there.
            if let Err(error) = window.show() {
                tracing::warn!(%error, "overlay not shown");
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn order_front<R: Runtime>(_app: &AppHandle<R>, window: &WebviewWindow<R>) {
    if let Err(error) = window.show() {
        tracing::warn!(%error, "overlay not shown");
    }
    // tao focuses a `focus: false` window on show; keep it above anyway.
    let _ = window.set_always_on_top(true);
}

#[cfg(target_os = "macos")]
mod macos {
    use tauri::{Runtime, WebviewWindow};
    use tauri_nspanel::objc2_app_kit::{NSWindowCollectionBehavior, NSWindowStyleMask};
    use tauri_nspanel::{PanelLevel, WebviewWindowExt, tauri_panel};

    tauri_panel! {
        panel!(OverlayPanel {
            config: {
                can_become_key_window: false,
                can_become_main_window: false,
                is_floating_panel: true,
                hides_on_deactivate: false
            }
        })
    }

    /// Spec §10 and the spike: non-activating, every Space, above full-screen
    /// games, no mouse, no shadow.
    pub fn make_panel<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
        let panel = window.to_panel::<OverlayPanel<R>>()?;
        panel
            .add_style_mask(NSWindowStyleMask::NonactivatingPanel)
            .map_err(|error| {
                tauri::Error::Io(std::io::Error::other(format!(
                    "overlay style mask: {error:?}"
                )))
            })?;
        panel.set_collection_behavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        panel.set_level(PanelLevel::ScreenSaver.value());
        panel.set_ignores_mouse_events(true);
        panel.set_has_shadow(false);
        panel.set_opaque(false);
        panel.set_hides_on_deactivate(false);
        Ok(())
    }
}
