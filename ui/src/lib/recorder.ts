// The shortcut recorder (spec §10, §11.12): the next key press becomes the
// combo, with the rules that keep it usable: a real key plus at least one
// modifier, never F12, never the Win key, and on macOS never Option alone
// or Option+Shift alone (they type characters).
import type { Modifier } from "./generated/Modifier";
import type { Shortcut } from "./generated/Shortcut";
import type { Platform } from "./shortcut";
import { strings } from "./strings";

const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "CapsLock",
  "Fn",
]);

/** The combo a key event describes, or null while only modifiers are down. */
export function shortcutFromKeyEvent(event: {
  code: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}): Shortcut | null {
  if (!event.code || MODIFIER_CODES.has(event.code)) {
    return null;
  }
  const modifiers: Modifier[] = [];
  if (event.ctrlKey) {
    modifiers.push("ctrl");
  }
  if (event.altKey) {
    modifiers.push("alt");
  }
  if (event.shiftKey) {
    modifiers.push("shift");
  }
  if (event.metaKey) {
    modifiers.push("meta");
  }
  return { modifiers, code: event.code };
}

/** Why `shortcut` is refused, in plain English, or null when it is fine. */
export function shortcutProblem(shortcut: Shortcut, platform: Platform): string | null {
  const r = strings.settings.recorder;
  if (shortcut.code === "F12") {
    return r.f12;
  }
  if (shortcut.modifiers.length === 0) {
    return r.noModifier;
  }
  if (platform === "windows" && shortcut.modifiers.includes("meta")) {
    return r.winKey;
  }
  if (platform === "macos") {
    const set = new Set(shortcut.modifiers);
    const optionOnly = set.has("alt") && !set.has("ctrl") && !set.has("meta");
    if (optionOnly) {
      return r.optionOnly;
    }
  }
  return null;
}
