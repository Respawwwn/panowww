//! `Camera` adapters that move the in-game camera by injecting input.
//!
//! macOS posts `CGEvent`s; Windows uses `SendInput` and ViGEm for the gamepad.
//! Adapters only perform primitives (press or release a key, move the mouse by
//! a delta, press or release a button, set a stick position); all movement
//! logic lives in `panowww-core`. Also owns the input prerequisite checks
//! (Accessibility, ViGEmBus) and the shared `KeyboardEvent.code` to `kVK_*` /
//! scan-code table.

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;
