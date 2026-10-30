//! macOS: ScreenCaptureKit (spec §9). `SCScreenshotManager` per Shot and
//! Sample through a display content filter that excludes Panowww's own
//! windows, output at native pixels; the Display preview is the same
//! screenshot path at a reduced output size, two frames per second.
//!
//! Raw `objc2-screen-capture-kit` bindings, no Swift: the `screencapturekit`
//! crate needs the macOS 26 SDK to build (spike finding).

pub mod displays;
pub mod permission;
mod sck;
mod stream;

pub use displays::{DisplayEntry, list_displays};
pub use stream::{SckGrabber, SckStream};
