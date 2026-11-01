//! Hides a window of ours from every screen capture (spec §9): the overlay
//! never lands in a Shot, and the main window and the picker never appear
//! in the Display preview. `WDA_EXCLUDEFROMCAPTURE` needs Windows 10 2004;
//! older systems fall back to a black box, which the frame drawn outside
//! the zone keeps out of the Shots anyway.

use std::ffi::c_void;

use panowww_core::ports::CaptureError;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE};

/// Excludes the top-level window `hwnd` (owned by this process) from
/// capture. `hwnd` is the raw handle a Tauri window exposes on Windows.
pub fn exclude_from_capture(hwnd: *mut c_void) -> Result<(), CaptureError> {
    // SAFETY: the handle names a window of this process; a bad handle makes
    // the call fail, nothing else.
    unsafe { SetWindowDisplayAffinity(HWND(hwnd), WDA_EXCLUDEFROMCAPTURE) }
        .map_err(|e| CaptureError::Unavailable(format!("SetWindowDisplayAffinity failed: {e}")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_null_window_is_refused_not_a_crash() {
        assert!(super::exclude_from_capture(std::ptr::null_mut()).is_err());
    }
}
