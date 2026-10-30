//! The Accessibility permission (spec §8.4): checked at launch and at every
//! countdown, requested from onboarding. Unsigned builds lose the grant on
//! update; the user then removes Panowww from the list and adds it again.

use objc2_application_services::AXIsProcessTrusted;
use objc2_core_graphics::{CGPreflightPostEventAccess, CGRequestPostEventAccess};

/// Deep link to Privacy & Security → Accessibility.
pub const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/// Whether this process may post events: both the Accessibility trust and
/// the post-event preflight, since a stale grant can tick one and not the
/// other.
pub fn check() -> bool {
    // SAFETY: takes no arguments and only reads TCC state.
    let trusted = unsafe { AXIsProcessTrusted() };
    let may_post = CGPreflightPostEventAccess();
    trusted && may_post
}

/// Asks macOS for the permission; shows the system prompt the first time.
/// Returns whether it is granted right now.
pub fn request() -> bool {
    CGRequestPostEventAccess()
}

/// Opens System Settings on the Accessibility list.
pub fn open_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(SETTINGS_URL)
        .spawn()
        .map(|_| ())
}
