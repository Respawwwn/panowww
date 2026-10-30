//! The `Grabber` and `Stream` of a Mac: Shots and Samples through the
//! screenshot path at native pixels; the Display preview through the same
//! path at a reduced size on its own thread, latest-wins, torn down and up
//! with `set_preview`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;
use objc2_core_graphics::CGDirectDisplayID;
use panowww_core::ports::{
    CaptureError, DisplayId, Frame, Grabber, Jpeg, PreviewFrame, PreviewSink, Rect, Stream,
};

use super::displays::find_display;
use super::sck::{Capture, CaptureSpec, source_rect};
use crate::{PREVIEW_FPS, PREVIEW_JPEG_QUALITY, PREVIEW_LONG_SIDE};

/// Opens [`SckStream`]s. `app` holds one and calls `start` before every
/// countdown (spec §4.5).
#[derive(Debug, Default)]
pub struct SckGrabber;

impl Grabber for SckGrabber {
    fn start(
        &mut self,
        display: DisplayId,
        zone: Rect,
        preview: PreviewSink,
    ) -> Result<Box<dyn Stream>, CaptureError> {
        Ok(Box::new(SckStream::open(&display, zone, preview)?))
    }
}

/// One open capture of one display, cropped to the zone for Shots.
pub struct SckStream {
    shots: Capture,
    preview: Preview,
}

impl std::fmt::Debug for SckStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SckStream").finish_non_exhaustive()
    }
}

impl SckStream {
    /// Builds the Shot capture of `zone` (display-local points) on
    /// `display` and the preview's inputs; the preview itself starts with
    /// `set_preview(true)`.
    pub fn open(display: &DisplayId, zone: Rect, sink: PreviewSink) -> Result<Self, CaptureError> {
        let entry = find_display(display)?;
        let scale = entry.info.scale;
        let spec = CaptureSpec {
            source: Some(source_rect(zone.x, zone.y, zone.w, zone.h)),
            width: pixels(zone.w, scale),
            height: pixels(zone.h, scale),
        };
        let (shots, _) = Capture::new(entry.cg_id, spec)?;
        let display_id = display.0.as_str();
        tracing::info!(display_id, ?zone, scale, "capture opened");
        Ok(Self {
            shots,
            preview: Preview::new(entry.cg_id, entry.info.width, entry.info.height, zone, sink),
        })
    }
}

/// One screenshot of the whole display, scaled so its longer side is at
/// most `long_side` pixels, as JPEG: the dimmed picture behind the Capture
/// zone picker (spec §11.4).
pub fn display_image(display: &DisplayId, long_side: u32) -> Result<Jpeg, CaptureError> {
    let entry = find_display(display)?;
    let native_w = pixels(entry.info.width, entry.info.scale);
    let native_h = pixels(entry.info.height, entry.info.scale);
    let (width, height) = fit(
        u32::try_from(native_w).unwrap_or(u32::MAX),
        u32::try_from(native_h).unwrap_or(u32::MAX),
        long_side,
    );
    let spec = CaptureSpec {
        source: None,
        width: width as usize,
        height: height as usize,
    };
    let (capture, _) = Capture::new(entry.cg_id, spec)?;
    encode(&capture.screenshot()?)
}

impl Stream for SckStream {
    /// `SCScreenshotManager` captures on demand, so every frame is newer
    /// than `not_before` by construction.
    fn shot(&mut self, _not_before: Instant) -> Result<Frame, CaptureError> {
        let started = Instant::now();
        let frame = self.shots.screenshot()?;
        tracing::debug!(
            ms = started.elapsed().as_millis(),
            width = frame.width,
            height = frame.height,
            "shot"
        );
        Ok(frame)
    }

    fn set_preview(&mut self, on: bool) {
        if on {
            self.preview.start();
        } else {
            self.preview.stop();
        }
    }

    fn stop(mut self: Box<Self>) {
        self.preview.stop();
    }
}

/// The preview thread: a reduced screenshot of the whole display every
/// `1 / PREVIEW_FPS`, JPEG-encoded and handed to the sink, latest-wins.
struct Preview {
    cg_id: CGDirectDisplayID,
    display_w: u32,
    display_h: u32,
    zone: Rect,
    sink: Arc<PreviewSink>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Preview {
    fn new(
        cg_id: CGDirectDisplayID,
        display_w: u32,
        display_h: u32,
        zone: Rect,
        sink: PreviewSink,
    ) -> Self {
        Self {
            cg_id,
            display_w,
            display_h,
            zone,
            sink: Arc::new(sink),
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
    }

    fn start(&mut self) {
        if self.thread.is_some() {
            return;
        }
        self.running.store(true, Ordering::SeqCst);
        let (width, height) = fit(self.display_w, self.display_h, PREVIEW_LONG_SIDE);
        let spec = CaptureSpec {
            source: None,
            width: width as usize,
            height: height as usize,
        };
        let cg_id = self.cg_id;
        let (display_w, display_h, zone) = (self.display_w, self.display_h, self.zone);
        let sink = Arc::clone(&self.sink);
        let running = Arc::clone(&self.running);
        let interval = Duration::from_millis(1000 / u64::from(PREVIEW_FPS.max(1)));
        let handle = std::thread::Builder::new()
            .name("panowww-preview".to_owned())
            .spawn(move || {
                let capture = match Capture::new(cg_id, spec) {
                    Ok((capture, _)) => capture,
                    Err(error) => {
                        tracing::warn!(%error, "preview unavailable");
                        return;
                    }
                };
                while running.load(Ordering::SeqCst) {
                    let tick = Instant::now();
                    match capture.screenshot().and_then(|frame| encode(&frame)) {
                        Ok(jpeg) => sink(PreviewFrame {
                            jpeg,
                            display_w,
                            display_h,
                            zone,
                        }),
                        Err(error) => tracing::warn!(%error, "preview frame skipped"),
                    }
                    // Sleep in slices so `stop` lands within one frame.
                    while running.load(Ordering::SeqCst) && tick.elapsed() < interval {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }
            });
        match handle {
            Ok(handle) => self.thread = Some(handle),
            Err(error) => tracing::warn!(%error, "preview thread not started"),
        }
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.stop();
    }
}

fn encode(frame: &Frame) -> Result<Jpeg, CaptureError> {
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, PREVIEW_JPEG_QUALITY)
        .encode(
            &frame.rgb,
            frame.width,
            frame.height,
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| CaptureError::ShotFailed(format!("preview JPEG: {e}")))?;
    Ok(Jpeg(bytes))
}

/// `points × scale`, at least 1.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "rounded positive pixel counts"
)]
fn pixels(points: u32, scale: f64) -> usize {
    ((f64::from(points) * scale).round() as usize).max(1)
}

/// `width × height` scaled so the longer side is `long_side`, never
/// enlarged.
fn fit(width: u32, height: u32, long_side: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest == 0 || longest <= long_side {
        return (width.max(1), height.max(1));
    }
    let scale = |side: u32| {
        u32::try_from(u64::from(side) * u64::from(long_side) / u64::from(longest))
            .unwrap_or(long_side)
            .max(1)
    };
    (scale(width), scale(height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_sizes_follow_the_scale() {
        assert_eq!(pixels(1920, 2.0), 3840);
        assert_eq!(pixels(1, 1.0), 1);
        assert_eq!(pixels(0, 2.0), 1);
    }

    #[test]
    fn preview_fits_the_long_side() {
        assert_eq!(fit(2560, 1440, 640), (640, 360));
        assert_eq!(fit(1440, 2560, 640), (360, 640));
        assert_eq!(fit(320, 180, 640), (320, 180));
    }
}
