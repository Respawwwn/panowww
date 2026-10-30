//! The Screen Recording permission (spec §9): checked at launch and at
//! every countdown, requested from onboarding. The app usually needs a
//! relaunch after a grant; onboarding says so.

use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};

/// Deep link to Privacy & Security → Screen & System Audio Recording.
pub const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

/// Whether this process may capture the screen. Never prompts.
pub fn check() -> bool {
    CGPreflightScreenCaptureAccess()
}

/// Asks macOS for the permission; shows the system prompt the first time
/// only. Returns whether it is granted right now.
pub fn request() -> bool {
    CGRequestScreenCaptureAccess()
}

/// Opens System Settings on the Screen Recording list.
pub fn open_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(SETTINGS_URL)
        .spawn()
        .map(|_| ())
}
