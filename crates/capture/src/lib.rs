//! `Grabber` and `Stream` adapters that capture the screen.
//!
//! macOS uses ScreenCaptureKit; Windows uses Windows Graphics Capture.
//! Adapters only return fresh frames cropped to the Capture zone. Also owns
//! display enumeration, the Screen Recording check and request, and the
//! production of the Display preview.

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;
