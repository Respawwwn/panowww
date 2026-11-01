//! The Accessibility permission (spec §8.4): checked at launch and at every
//! countdown, requested from onboarding. Unsigned builds lose the grant on
//! update; the user then removes Panowww from the list and adds it again.
//!
//! Only `AXIsProcessTrusted` is consulted: `CGPreflightPostEventAccess`
//! keeps the answer it gave when the process started, so a grant made while
//! Panowww runs would stay red until a relaunch. Posting events needs the
//! Accessibility grant alone; Panowww never listens to input, so it never
//! needs Input Monitoring.

use objc2_application_services::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFRetained, CFString};

/// Deep link to Privacy & Security → Accessibility.
pub const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/// Whether this process is trusted for Accessibility right now; follows
/// the switch in System Settings live.
pub fn check() -> bool {
    // SAFETY: takes no arguments and only reads TCC state.
    unsafe { AXIsProcessTrusted() }
}

/// Asks macOS for the permission; shows the Accessibility prompt the first
/// time (and adds Panowww to the list, unticked). Returns whether it is
/// granted right now.
pub fn request() -> bool {
    // SAFETY: the key is a CoreFoundation constant and the dictionary lives
    // for the call.
    unsafe {
        let options: CFRetained<CFDictionary<CFString, CFBoolean>> =
            CFDictionary::from_slices(&[kAXTrustedCheckOptionPrompt], &[CFBoolean::new(true)]);
        AXIsProcessTrustedWithOptions(Some(options.as_opaque()))
    }
}

/// Opens System Settings on the Accessibility list.
pub fn open_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(SETTINGS_URL)
        .spawn()
        .map(|_| ())
}
