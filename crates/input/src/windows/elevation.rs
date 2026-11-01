//! Whether this process runs elevated (spec §8.4). Windows has no input
//! permission; what drops injected input is UIPI, when the game's integrity
//! level is above ours. The app manifest asks for `requireAdministrator`,
//! so this is a diagnostic for the onboarding and the manual checks, not a
//! gate.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

#[expect(
    clippy::cast_possible_truncation,
    reason = "TOKEN_ELEVATION is 4 bytes; the API wants its size as u32"
)]
const ELEVATION_SIZE: u32 = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

/// Whether the process token is elevated (UAC accepted). `false` when the
/// token cannot be read.
pub fn is_elevated() -> bool {
    let mut token = HANDLE::default();
    // SAFETY: the current-process pseudo-handle is always valid; `token`
    // receives a real handle, closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) }.is_err() {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut written = 0u32;
    // SAFETY: the buffer is one TOKEN_ELEVATION and its exact size is given.
    let read = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast()),
            ELEVATION_SIZE,
            &raw mut written,
        )
    };
    // SAFETY: the handle was opened above and is not used afterwards.
    let _ = unsafe { CloseHandle(token) };
    read.is_ok() && elevation.TokenIsElevated != 0
}

#[cfg(test)]
mod tests {
    #[test]
    fn reading_the_token_does_not_fail() {
        // Either answer is right; the test process may or may not be elevated.
        let _ = super::is_elevated();
    }
}
