//! The ScreenCaptureKit calls: shareable content, the content filter that
//! excludes this process, one screenshot into an RGB8 frame.

use std::ptr::NonNull;
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGDataProvider, CGDirectDisplayID, CGImage};
use objc2_foundation::{NSArray, NSError};
use objc2_screen_capture_kit::{
    SCContentFilter, SCDisplay, SCRunningApplication, SCScreenshotManager, SCShareableContent,
    SCStreamConfiguration, SCWindow,
};
use panowww_core::ports::{CaptureError, Frame};

/// `kCVPixelFormatType_32BGRA`.
const FOURCC_BGRA: u32 = u32::from_be_bytes(*b"BGRA");
/// How long to wait for ScreenCaptureKit before calling the capture lost.
const TIMEOUT: Duration = Duration::from_secs(5);

/// A display filter excluding Panowww and the configuration of one capture:
/// the Shot path at native pixels, or the preview at a reduced size.
pub struct Capture {
    filter: Retained<SCContentFilter>,
    config: Retained<SCStreamConfiguration>,
}

// SAFETY: SCContentFilter and SCStreamConfiguration are immutable once
// built, and ScreenCaptureKit accepts them from any thread; the Runner
// thread takes the Shot path, the preview thread its own copy.
unsafe impl Send for Capture {}

/// What a capture is cropped to and how large the output is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureSpec {
    /// Display-local points; `None` captures the whole display.
    pub source: Option<CGRect>,
    /// Output size in pixels.
    pub width: usize,
    pub height: usize,
}

fn error_text(error: *mut NSError, fallback: &str) -> String {
    // SAFETY: ScreenCaptureKit hands a valid NSError or NULL to its
    // completion handlers.
    unsafe { error.as_ref() }.map_or_else(
        || fallback.to_owned(),
        |e| e.localizedDescription().to_string(),
    )
}

/// The displays, windows and applications ScreenCaptureKit may capture.
/// Fails without the Screen Recording permission.
fn shareable_content() -> Result<Retained<SCShareableContent>, CaptureError> {
    let (tx, rx) = mpsc::channel::<Result<Retained<SCShareableContent>, String>>();
    let block = RcBlock::new(
        move |content: *mut SCShareableContent, error: *mut NSError| {
            let result = match NonNull::new(content) {
                None => Err(error_text(error, "no shareable content")),
                // SAFETY: a non-NULL pointer from the completion handler is a live
                // SCShareableContent; retaining it is the documented way to keep it.
                Some(p) => unsafe { Retained::retain(p.as_ptr()) }
                    .ok_or_else(|| "retain failed".to_owned()),
            };
            let _ = tx.send(result);
        },
    );
    // SAFETY: the block outlives the call; ScreenCaptureKit copies it.
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
    match rx.recv_timeout(TIMEOUT) {
        Ok(Ok(content)) => Ok(content),
        Ok(Err(message)) => Err(permission_or_unavailable(&message)),
        Err(_) => Err(CaptureError::Unavailable(
            "ScreenCaptureKit did not answer".to_owned(),
        )),
    }
}

/// A missing permission comes back as a generic error from
/// ScreenCaptureKit; the preflight tells the two apart.
fn permission_or_unavailable(message: &str) -> CaptureError {
    if super::permission::check() {
        CaptureError::Unavailable(message.to_owned())
    } else {
        CaptureError::PermissionRevoked
    }
}

impl Capture {
    /// A capture of the display `cg_id`, excluding every window of this
    /// process. Returns the capture and the display's point-to-pixel scale.
    pub fn new(cg_id: CGDirectDisplayID, spec: CaptureSpec) -> Result<(Self, f64), CaptureError> {
        let content = shareable_content()?;
        // SAFETY: plain accessors of a live SCShareableContent.
        let (displays, applications) = unsafe { (content.displays(), content.applications()) };
        let display: Retained<SCDisplay> = displays
            .iter()
            // SAFETY: plain accessor of a live SCDisplay.
            .find(|d| unsafe { d.displayID() } == cg_id)
            .ok_or(CaptureError::DisplayGone)?;
        let pid = i32::try_from(std::process::id()).unwrap_or(-1);
        let ours: Vec<Retained<SCRunningApplication>> = applications
            .iter()
            // SAFETY: plain accessor of a live SCRunningApplication.
            .filter(|a| unsafe { a.processID() } == pid)
            .collect();
        let ours_array: Retained<NSArray<SCRunningApplication>> =
            NSArray::from_retained_slice(&ours);
        let no_windows: Retained<NSArray<SCWindow>> = NSArray::new();
        // SAFETY: the initialiser takes a live display and arrays it copies.
        let filter = unsafe {
            SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
                SCContentFilter::alloc(),
                &display,
                &ours_array,
                &no_windows,
            )
        };
        // SAFETY: plain accessor of a live filter.
        let scale = f64::from(unsafe { filter.pointPixelScale() });
        // SAFETY: a fresh configuration; every setter takes a plain value.
        let config = unsafe {
            let config = SCStreamConfiguration::new();
            if let Some(rect) = spec.source {
                config.setSourceRect(rect);
            }
            config.setWidth(spec.width);
            config.setHeight(spec.height);
            config.setShowsCursor(false);
            config.setPixelFormat(FOURCC_BGRA);
            config
        };
        Ok((
            Self { filter, config },
            if scale > 0.0 { scale } else { 1.0 },
        ))
    }

    /// One screenshot, taken now, as RGB8.
    pub fn screenshot(&self) -> Result<Frame, CaptureError> {
        let (tx, rx) = mpsc::channel::<Result<CFRetained<CGImage>, String>>();
        let block = RcBlock::new(move |image: *mut CGImage, error: *mut NSError| {
            let result = match NonNull::new(image) {
                None => Err(error_text(error, "captureImage returned NULL")),
                // SAFETY: a non-NULL CGImage from the completion handler is
                // live; retaining keeps it past the handler.
                Some(p) => Ok(unsafe { CFRetained::retain(p) }),
            };
            let _ = tx.send(result);
        });
        // SAFETY: the filter and configuration are live; the block is copied.
        unsafe {
            SCScreenshotManager::captureImageWithFilter_configuration_completionHandler(
                &self.filter,
                &self.config,
                Some(&block),
            );
        }
        let image = match rx.recv_timeout(TIMEOUT) {
            Ok(Ok(image)) => image,
            Ok(Err(message)) => return Err(CaptureError::ShotFailed(message)),
            Err(_) => {
                return Err(CaptureError::ShotFailed(
                    "ScreenCaptureKit did not answer".to_owned(),
                ));
            }
        };
        frame_from_image(&image)
    }
}

/// Copies a 32-bit `CGImage` into an RGB8 frame, whatever its channel order.
fn frame_from_image(image: &CGImage) -> Result<Frame, CaptureError> {
    let width = CGImage::width(Some(image));
    let height = CGImage::height(Some(image));
    let bytes_per_row = CGImage::bytes_per_row(Some(image));
    let bits_per_pixel = CGImage::bits_per_pixel(Some(image));
    let info = CGImage::bitmap_info(Some(image)).0;
    if bits_per_pixel != 32 {
        return Err(CaptureError::ShotFailed(format!(
            "unexpected {bits_per_pixel} bits per pixel"
        )));
    }
    let provider = CGImage::data_provider(Some(image))
        .ok_or_else(|| CaptureError::ShotFailed("the image has no data provider".to_owned()))?;
    let data = CGDataProvider::data(Some(&provider))
        .ok_or_else(|| CaptureError::ShotFailed("the image data could not be copied".to_owned()))?;
    let raw = data.to_vec();
    let order = channel_order(info);
    let mut rgb = Vec::with_capacity(width * height * 3);
    for row in 0..height {
        let line = raw
            .get(row * bytes_per_row..row * bytes_per_row + width * 4)
            .ok_or_else(|| {
                CaptureError::ShotFailed("the image data is shorter than its size".to_owned())
            })?;
        for pixel in line.as_chunks::<4>().0 {
            let [r, g, b] = order.rgb(*pixel);
            rgb.extend_from_slice(&[r, g, b]);
        }
    }
    Ok(Frame {
        width: u32::try_from(width).unwrap_or(u32::MAX),
        height: u32::try_from(height).unwrap_or(u32::MAX),
        rgb,
    })
}

/// Where red, green and blue sit in a 32-bit pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChannelOrder {
    /// Little-endian, alpha first: B G R A in memory (what SCK gives).
    Bgra,
    /// Big-endian, alpha last: R G B A.
    Rgba,
    /// Big-endian, alpha first: A R G B.
    Argb,
    /// Little-endian, alpha last: A B G R.
    Abgr,
}

impl ChannelOrder {
    fn rgb(self, p: [u8; 4]) -> [u8; 3] {
        match self {
            Self::Bgra => [p[2], p[1], p[0]],
            Self::Rgba => [p[0], p[1], p[2]],
            Self::Argb => [p[1], p[2], p[3]],
            Self::Abgr => [p[3], p[2], p[1]],
        }
    }
}

/// `kCGBitmapByteOrder32Little` and the alpha-first `CGImageAlphaInfo`s.
fn channel_order(bitmap_info: u32) -> ChannelOrder {
    const BYTE_ORDER_32_LITTLE: u32 = 0x2000;
    const ALPHA_MASK: u32 = 0x1F;
    let little = bitmap_info & BYTE_ORDER_32_LITTLE != 0;
    // PremultipliedFirst = 2, First = 4, NoneSkipFirst = 6.
    let alpha_first = matches!(bitmap_info & ALPHA_MASK, 2 | 4 | 6);
    match (little, alpha_first) {
        (true, true) => ChannelOrder::Bgra,
        (true, false) => ChannelOrder::Abgr,
        (false, true) => ChannelOrder::Argb,
        (false, false) => ChannelOrder::Rgba,
    }
}

/// The display-local rectangle of a zone, in points.
pub fn source_rect(x: i32, y: i32, w: u32, h: u32) -> CGRect {
    CGRect::new(
        CGPoint::new(f64::from(x), f64::from(y)),
        CGSize::new(f64::from(w), f64::from(h)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_orders_follow_the_bitmap_info() {
        assert_eq!(channel_order(0x2000 | 2), ChannelOrder::Bgra, "SCK's BGRA");
        assert_eq!(channel_order(0x2000 | 6), ChannelOrder::Bgra);
        assert_eq!(channel_order(1), ChannelOrder::Rgba);
        assert_eq!(channel_order(2), ChannelOrder::Argb);
        assert_eq!(channel_order(0x2000 | 1), ChannelOrder::Abgr);
    }

    #[test]
    fn every_order_picks_the_same_colour() {
        let bgra = [10, 20, 30, 255];
        assert_eq!(ChannelOrder::Bgra.rgb(bgra), [30, 20, 10]);
        assert_eq!(ChannelOrder::Rgba.rgb([30, 20, 10, 255]), [30, 20, 10]);
        assert_eq!(ChannelOrder::Argb.rgb([255, 30, 20, 10]), [30, 20, 10]);
        assert_eq!(ChannelOrder::Abgr.rgb([255, 10, 20, 30]), [30, 20, 10]);
    }
}
