// The current Settings, shared by every screen: the key caps read the
// shortcuts from here. Starts with the defaults of the Rust side so the UI
// renders before the first IPC answer (and in the gallery).
import { api } from "./api";
import type { Settings } from "./generated/Settings";
import type { Shortcut } from "./generated/Shortcut";
import type { ShortcutRef } from "./strings";

export const defaultSettings: Settings = {
  format: "panowww-settings",
  version: 1,
  sessionsFolder: "~/Pictures/Panowww",
  shortcuts: {
    start: { modifiers: ["ctrl", "shift"], code: "Digit8" },
    stop: { modifiers: ["ctrl", "shift"], code: "Digit9" },
  },
  newGameDefaults: { startPole: "nadir", waitAfterMoveMs: 300, waitAfterShotMs: 300 },
  checkForUpdates: true,
  displayPreview: true,
  appearance: "auto",
  onboarded: false,
};

let current = $state<Settings>(structuredClone(defaultSettings));

export const settings = {
  get value(): Settings {
    return current;
  },
  set value(next: Settings) {
    current = next;
  },
  shortcut(ref: ShortcutRef): Shortcut {
    return ref === "start" ? current.shortcuts.start : current.shortcuts.stop;
  },
  async load(): Promise<Settings> {
    current = await api.settings.get();
    return current;
  },
  async save(next: Settings): Promise<void> {
    current = next;
    await api.settings.save(next);
  },
};
