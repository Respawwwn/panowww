//! `Grabber` and `Stream` adapters that capture the screen.
//!
//! macOS uses ScreenCaptureKit ([`macos`]); Windows uses Windows Graphics
//! Capture (issue 24). Adapters only return fresh frames cropped to the
//! Capture zone. Also owns display enumeration, the Screen Recording check
//! and request, and the production of the Display preview.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;

/// Display preview frames per second (spec Appendix A).
pub const PREVIEW_FPS: u32 = 2;
/// Longer side of a Display preview frame, in pixels.
pub const PREVIEW_LONG_SIDE: u32 = 640;
/// JPEG quality of a Display preview frame.
pub const PREVIEW_JPEG_QUALITY: u8 = 70;
