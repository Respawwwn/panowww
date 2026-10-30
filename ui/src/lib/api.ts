// The one place the UI talks to Rust. Every command gets a typed wrapper
// here; the types come from `generated/`, written by ts-rs (`just generate`).
import { invoke } from "@tauri-apps/api/core";
import type { Pong } from "./generated/Pong";

export const api = {
  /** Proves the IPC works and returns the app version. */
  ping: () => invoke<Pong>("ping"),
};
