//! `Camera` adapters that move the in-game camera by injecting input.
//!
//! macOS posts `CGEvent`s ([`macos::CgCamera`]); Windows calls `SendInput`
//! and drives a virtual Xbox 360 pad through `ViGEmBus`
//! ([`windows::SendInputCamera`]). Adapters only perform primitives (press
//! or release a key, move the mouse by a delta, press or release a button,
//! set a stick position); all movement logic lives in `panowww-core`
//! (ADR 0001). Also owns the input prerequisite checks (Accessibility,
//! `ViGEmBus`, elevation) and the shared `KeyboardEvent.code` to `kVK_*` /
//! scan-code table ([`keys`]).

pub mod keys;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;
