//! The `Grabber` and `Stream` of a Windows machine: one Windows Graphics
//! Capture session per run through `windows-capture`, frames read back on
//! the capture thread only when a Shot is pending and the frame's
//! `SystemRelativeTime` is after the move; the Display preview is one frame
//! in N downscaled and JPEG-encoded on that same thread.
//!
//! WGC delivers a frame when the display composes a new one, so a Shot
//! waits for the next composition after `not_before`; a game that renders
//! nothing for [`SHOT_TIMEOUT`] fails the Shot and the Runner retries once.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;
use panowww_core::ports::{
    CaptureError, DisplayId, Frame, Grabber, Jpeg, PreviewFrame, PreviewSink, Rect, Stream,
};
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame as WcFrame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

use super::displays::find_display;
use crate::{PREVIEW_FPS, PREVIEW_JPEG_QUALITY, PREVIEW_LONG_SIDE};

/// How long a Shot waits for a frame newer than the move.
pub const SHOT_TIMEOUT: Duration = Duration::from_millis(2000);

type HandlerError = Box<dyn std::error::Error + Send + Sync>;

/// Opens [`WgcStream`]s. `app` holds one and calls `start` before every
/// countdown (spec §4.5).
#[derive(Debug, Default)]
pub struct WgcGrabber;

impl Grabber for WgcGrabber {
    fn start(
        &mut self,
        display: DisplayId,
        zone: Rect,
        preview: PreviewSink,
    ) -> Result<Box<dyn Stream>, CaptureError> {
        Ok(Box::new(WgcStream::open(&display, zone, preview)?))
    }
}

/// A pending Shot: the earliest acceptable frame time and, once the
/// capture thread served it, the cropped RGB frame.
struct Request {
    not_before_100ns: i64,
    result: Option<Result<Frame, CaptureError>>,
}

/// What the capture thread and the Runner thread share.
struct Shared {
    zone: Rect,
    display_w: u32,
    display_h: u32,
    request: Mutex<Option<Request>>,
    served: Condvar,
    frames_seen: AtomicU64,
    preview_on: AtomicBool,
    preview_last: Mutex<Option<Instant>>,
    sink: PreviewSink,
}

impl Shared {
    fn request(&self) -> MutexGuard<'_, Option<Request>> {
        self.request.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The `windows-capture` callback: runs on the capture thread.
struct Handler {
    shared: Arc<Shared>,
}

impl GraphicsCaptureApiHandler for Handler {
    type Flags = Arc<Shared>;
    type Error = HandlerError;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { shared: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut WcFrame,
        _control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        self.shared.frames_seen.fetch_add(1, Ordering::SeqCst);
        let timestamp = frame.timestamp().map_or(0, |t| t.Duration);
        self.serve_shot(frame, timestamp);
        self.serve_preview(frame);
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        tracing::info!("capture session closed by the system");
        Ok(())
    }
}

impl Handler {
    /// Crops the frame to the zone for a pending Shot, if the frame is
    /// newer than the move.
    fn serve_shot(&self, frame: &mut WcFrame, timestamp: i64) {
        let mut pending = self.shared.request();
        let Some(request) = pending.as_mut() else {
            return;
        };
        if request.result.is_some() || timestamp <= request.not_before_100ns {
            return;
        }
        request.result = Some(crop_to_zone(frame, self.shared.zone));
        drop(pending);
        self.shared.served.notify_all();
    }

    /// Every `1 / PREVIEW_FPS`, a downscaled copy of the whole frame.
    fn serve_preview(&self, frame: &mut WcFrame) {
        if !self.shared.preview_on.load(Ordering::SeqCst) {
            return;
        }
        let interval = Duration::from_millis(1000 / u64::from(PREVIEW_FPS.max(1)));
        {
            let mut last = self
                .shared
                .preview_last
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if last.is_some_and(|t| t.elapsed() < interval) {
                return;
            }
            *last = Some(Instant::now());
        }
        match preview_of(frame) {
            Ok(jpeg) => (self.shared.sink)(PreviewFrame {
                jpeg,
                display_w: self.shared.display_w,
                display_h: self.shared.display_h,
                zone: self.shared.zone,
            }),
            Err(error) => tracing::warn!(%error, "preview frame skipped"),
        }
    }
}

/// One open capture of one display, cropped to the zone for Shots.
pub struct WgcStream {
    control: Option<CaptureControl<Handler, HandlerError>>,
    shared: Arc<Shared>,
}

impl std::fmt::Debug for WgcStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WgcStream")
            .field("zone", &self.shared.zone)
            .finish_non_exhaustive()
    }
}

impl WgcStream {
    /// Starts the capture session of `display`; `zone` is display-local
    /// physical pixels. The preview starts with `set_preview(true)`.
    pub fn open(display: &DisplayId, zone: Rect, sink: PreviewSink) -> Result<Self, CaptureError> {
        let entry = find_display(display)?;
        let monitor = Monitor::enumerate()
            .map_err(|e| CaptureError::Unavailable(format!("monitors: {e}")))?
            .into_iter()
            .find(|m| m.as_raw_hmonitor() as isize == entry.hmonitor)
            .ok_or(CaptureError::DisplayGone)?;
        let shared = Arc::new(Shared {
            zone,
            display_w: entry.info.width,
            display_h: entry.info.height,
            request: Mutex::new(None),
            served: Condvar::new(),
            frames_seen: AtomicU64::new(0),
            preview_on: AtomicBool::new(false),
            preview_last: Mutex::new(None),
            sink,
        });
        let settings = Settings::new(
            monitor,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::WithoutBorder,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            Arc::clone(&shared),
        );
        let control = Handler::start_free_threaded(settings)
            .map_err(|e| CaptureError::Unavailable(format!("Windows Graphics Capture: {e}")))?;
        let display_id = display.0.as_str();
        tracing::info!(
            display_id,
            device = entry.device_name.as_str(),
            ?zone,
            "capture opened"
        );
        Ok(Self {
            control: Some(control),
            shared,
        })
    }
}

impl Stream for WgcStream {
    /// Waits for the first frame composed after `not_before`, cropped to
    /// the zone.
    fn shot(&mut self, not_before: Instant) -> Result<Frame, CaptureError> {
        let started = Instant::now();
        let not_before_100ns = qpc_100ns_at(not_before);
        let seen_before = self.shared.frames_seen.load(Ordering::SeqCst);
        let deadline = started + SHOT_TIMEOUT;
        let mut pending = self.shared.request();
        *pending = Some(Request {
            not_before_100ns,
            result: None,
        });
        loop {
            if pending.as_ref().is_some_and(|r| r.result.is_some()) {
                break;
            }
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            pending = self
                .shared
                .served
                .wait_timeout(pending, deadline - now)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        let result = pending.take().and_then(|r| r.result);
        drop(pending);
        let arrived = self.shared.frames_seen.load(Ordering::SeqCst) - seen_before;
        match result {
            Some(Ok(frame)) => {
                tracing::debug!(
                    ms = started.elapsed().as_millis(),
                    width = frame.width,
                    height = frame.height,
                    arrived,
                    "shot"
                );
                Ok(frame)
            }
            Some(Err(error)) => Err(error),
            None => Err(CaptureError::ShotFailed(format!(
                "no frame newer than the move within {} ms ({arrived} frames arrived, all older)",
                SHOT_TIMEOUT.as_millis()
            ))),
        }
    }

    fn set_preview(&mut self, on: bool) {
        self.shared.preview_on.store(on, Ordering::SeqCst);
        if !on {
            *self
                .shared
                .preview_last
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = None;
        }
    }

    fn stop(mut self: Box<Self>) {
        self.shared.preview_on.store(false, Ordering::SeqCst);
        if let Some(control) = self.control.take()
            && let Err(error) = control.stop()
        {
            tracing::warn!(%error, "capture session did not stop cleanly");
        }
    }
}

impl Drop for WgcStream {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.stop();
        }
    }
}

/// The zone of the frame as RGB8, clamped to the frame.
fn crop_to_zone(frame: &mut WcFrame, zone: Rect) -> Result<Frame, CaptureError> {
    let (frame_w, frame_h) = (frame.width(), frame.height());
    let x0 = u32::try_from(zone.x.max(0)).unwrap_or(0).min(frame_w);
    let y0 = u32::try_from(zone.y.max(0)).unwrap_or(0).min(frame_h);
    let x1 = x0.saturating_add(zone.w).min(frame_w);
    let y1 = y0.saturating_add(zone.h).min(frame_h);
    if x1 <= x0 || y1 <= y0 {
        return Err(CaptureError::ShotFailed(format!(
            "the zone {zone:?} lies outside the {frame_w}×{frame_h} frame"
        )));
    }
    let buffer = frame
        .buffer_crop(x0, y0, x1, y1)
        .map_err(|e| CaptureError::ShotFailed(format!("frame readback: {e}")))?;
    let mut packed = Vec::new();
    let bgra = buffer.as_nopadding_buffer(&mut packed);
    let (width, height) = (x1 - x0, y1 - y0);
    Ok(Frame {
        width,
        height,
        rgb: bgra_to_rgb(bgra, (width as usize) * (height as usize)),
    })
}

/// A JPEG of the whole frame with its longer side at most
/// `PREVIEW_LONG_SIDE`, box-filtered.
fn preview_of(frame: &mut WcFrame) -> Result<Jpeg, CaptureError> {
    let (frame_w, frame_h) = (frame.width(), frame.height());
    let buffer = frame
        .buffer()
        .map_err(|e| CaptureError::ShotFailed(format!("frame readback: {e}")))?;
    let mut packed = Vec::new();
    let bgra = buffer.as_nopadding_buffer(&mut packed);
    let step = frame_w.max(frame_h).div_ceil(PREVIEW_LONG_SIDE).max(1);
    let (rgb, width, height) = downscale_bgra(bgra, frame_w, frame_h, step);
    encode(&Frame { width, height, rgb })
}

/// BGRA bytes to RGB8, `pixels` pixels.
fn bgra_to_rgb(bgra: &[u8], pixels: usize) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(pixels * 3);
    for px in bgra.as_chunks::<4>().0.iter().take(pixels) {
        rgb.extend_from_slice(&[px[2], px[1], px[0]]);
    }
    rgb
}

/// Averages `step × step` BGRA blocks into one RGB pixel each.
fn downscale_bgra(bgra: &[u8], width: u32, height: u32, step: u32) -> (Vec<u8>, u32, u32) {
    let out_w = width.div_ceil(step).max(1);
    let out_h = height.div_ceil(step).max(1);
    let (row_len, block) = (width as usize, step as usize);
    let mut rgb = Vec::with_capacity((out_w as usize) * (out_h as usize) * 3);
    for out_y in 0..out_h as usize {
        for out_x in 0..out_w as usize {
            let (mut red, mut green, mut blue, mut count) = (0u32, 0u32, 0u32, 0u32);
            for y in (out_y * block)..((out_y + 1) * block).min(height as usize) {
                for x in (out_x * block)..((out_x + 1) * block).min(row_len) {
                    let at = (y * row_len + x) * 4;
                    if let Some(px) = bgra.get(at..at + 3) {
                        blue += u32::from(px[0]);
                        green += u32::from(px[1]);
                        red += u32::from(px[2]);
                        count += 1;
                    }
                }
            }
            let count = count.max(1);
            rgb.extend_from_slice(&[
                channel(red / count),
                channel(green / count),
                channel(blue / count),
            ]);
        }
    }
    (rgb, out_w, out_h)
}

/// An averaged channel back to a byte.
fn channel(value: u32) -> u8 {
    u8::try_from(value.min(255)).unwrap_or(u8::MAX)
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

/// The QPC time, in the 100 ns units of WGC's `SystemRelativeTime`, of an
/// `Instant` of this process.
fn qpc_100ns_at(instant: Instant) -> i64 {
    let (mut counter, mut frequency) = (0i64, 0i64);
    // SAFETY: two out-parameters; both calls always succeed on Windows XP+.
    unsafe {
        let _ = QueryPerformanceCounter(&raw mut counter);
        let _ = QueryPerformanceFrequency(&raw mut frequency);
    }
    if frequency <= 0 {
        return 0;
    }
    let now_100ns = i128::from(counter) * 10_000_000 / i128::from(frequency);
    let age_100ns = i128::try_from(instant.elapsed().as_nanos() / 100).unwrap_or(0);
    i64::try_from(now_100ns - age_100ns).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_becomes_rgb() {
        assert_eq!(
            bgra_to_rgb(&[10, 20, 30, 255, 1, 2, 3, 0], 2),
            [30, 20, 10, 3, 2, 1]
        );
        assert_eq!(bgra_to_rgb(&[10, 20, 30, 255, 1, 2, 3, 0], 1), [30, 20, 10]);
    }

    #[test]
    fn downscaling_averages_blocks_and_rounds_the_size_up() {
        // 3 × 2 image, step 2: two columns (2 + 1 wide), one row.
        let bgra = [
            0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 50, 0, //
            0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 50, 0,
        ];
        let (rgb, w, h) = downscale_bgra(&bgra, 3, 2, 2);
        assert_eq!((w, h), (2, 1));
        assert_eq!(rgb, [150, 0, 0, 50, 0, 0]);
    }

    #[test]
    fn the_preview_step_fits_the_long_side() {
        assert_eq!(1920u32.div_ceil(PREVIEW_LONG_SIDE), 3);
        assert_eq!(3840u32.div_ceil(PREVIEW_LONG_SIDE), 6);
        assert_eq!(640u32.div_ceil(PREVIEW_LONG_SIDE), 1);
    }

    #[test]
    fn qpc_times_are_monotonic_and_in_the_past_for_old_instants() {
        let earlier = Instant::now();
        std::thread::sleep(Duration::from_millis(5));
        let a = qpc_100ns_at(earlier);
        let b = qpc_100ns_at(Instant::now());
        assert!(b - a >= 40_000, "5 ms is 50 000 units, got {}", b - a);
    }
}
