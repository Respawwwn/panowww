//! The Panowww binary: hands over to the library's `run`.
// No console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    panowww_app::run();
}
