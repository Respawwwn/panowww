//! The connected displays: a stable id, the size in points, the scale.
//! `CGDirectDisplayID` is not stable across reboots or replugs, so the id
//! is built from the vendor, model and serial numbers, with the position
//! as a tie-breaker for identical displays without a serial.

use objc2_core_foundation::CGRect;
use objc2_core_graphics::{
    CGDirectDisplayID, CGDisplayBounds, CGDisplayModelNumber, CGDisplayPixelsHigh,
    CGDisplayPixelsWide, CGDisplaySerialNumber, CGDisplayVendorNumber, CGGetActiveDisplayList,
};
use panowww_core::ports::{CaptureError, DisplayId, DisplayInfo};

/// Room for more displays than anyone connects.
const MAX_DISPLAYS: u32 = 16;

/// One connected display, with what ScreenCaptureKit needs to find it.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayEntry {
    pub info: DisplayInfo,
    pub cg_id: CGDirectDisplayID,
    /// Global points, origin at the top-left of the main display.
    pub bounds: CGRect,
}

/// Every active display, main display first.
pub fn list_displays() -> Result<Vec<DisplayEntry>, CaptureError> {
    let mut ids = [0 as CGDirectDisplayID; MAX_DISPLAYS as usize];
    let mut count: u32 = 0;
    // SAFETY: `ids` holds MAX_DISPLAYS entries and `count` is written with
    // how many were filled, as the function contract states.
    let error = unsafe { CGGetActiveDisplayList(MAX_DISPLAYS, ids.as_mut_ptr(), &raw mut count) };
    if error != objc2_core_graphics::CGError::Success {
        return Err(CaptureError::Unavailable(format!(
            "CGGetActiveDisplayList failed: {error:?}"
        )));
    }
    let mut entries: Vec<DisplayEntry> = Vec::new();
    for &cg_id in ids.iter().take(count as usize) {
        let bounds = CGDisplayBounds(cg_id);
        let pixels_wide = CGDisplayPixelsWide(cg_id);
        let width = points(bounds.size.width);
        let height = points(bounds.size.height);
        let scale = if width > 0 {
            f64::from(u32::try_from(pixels_wide).unwrap_or(width)) / f64::from(width)
        } else {
            1.0
        };
        let base = format!(
            "{:04x}-{:04x}-{:08x}",
            CGDisplayVendorNumber(cg_id),
            CGDisplayModelNumber(cg_id),
            CGDisplaySerialNumber(cg_id)
        );
        let id = if entries.iter().any(|e| e.info.id.0 == base) {
            format!("{base}@{},{}", bounds.origin.x, bounds.origin.y)
        } else {
            base
        };
        entries.push(DisplayEntry {
            info: DisplayInfo {
                id: DisplayId(id),
                width,
                height,
                scale,
                hdr: false,
            },
            cg_id,
            bounds,
        });
        let _ = CGDisplayPixelsHigh(cg_id);
    }
    Ok(entries)
}

/// The display with this id, if connected.
pub fn find_display(id: &DisplayId) -> Result<DisplayEntry, CaptureError> {
    list_displays()?
        .into_iter()
        .find(|e| e.info.id == *id)
        .ok_or(CaptureError::DisplayGone)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "display sizes are small positive integers"
)]
fn points(value: f64) -> u32 {
    value.round().max(0.0) as u32
}
