// Rendering a Settings shortcut as key caps: `Ctrl+Shift+8` becomes the
// caps ["Ctrl", "Shift", "8"], joined by "+" in the UI (spec §12).
import type { Modifier } from "./generated/Modifier";
import type { Shortcut } from "./generated/Shortcut";

export type Platform = "macos" | "windows";

/** The platform the UI runs on, from the user agent; macOS when unsure. */
export function detectPlatform(): Platform {
  if (typeof navigator !== "undefined" && /windows/i.test(navigator.userAgent)) {
    return "windows";
  }
  return "macos";
}

const MODIFIER_ORDER: Modifier[] = ["ctrl", "alt", "shift", "meta"];

/** The cap of one modifier on `platform`. */
export function modifierLabel(modifier: Modifier, platform: Platform): string {
  switch (modifier) {
    case "ctrl":
      return "Ctrl";
    case "shift":
      return "Shift";
    case "alt":
      return platform === "macos" ? "Option" : "Alt";
    case "meta":
      return platform === "macos" ? "Cmd" : "Win";
  }
}

/** The cap of a physical key named by its `KeyboardEvent.code`. */
export function keyLabel(code: string): string {
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter?.[1]) {
    return letter[1];
  }
  const digit = /^Digit(\d)$/.exec(code);
  if (digit?.[1]) {
    return digit[1];
  }
  const numpad = /^Numpad(\d)$/.exec(code);
  if (numpad?.[1]) {
    return `Num ${numpad[1]}`;
  }
  if (/^F\d{1,2}$/.test(code)) {
    return code;
  }
  const named: Record<string, string> = {
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    Space: "Space",
    Enter: "Enter",
    NumpadEnter: "Num Enter",
    Tab: "Tab",
    Backspace: "⌫",
    Delete: "Del",
    Escape: "Esc",
    Home: "Home",
    End: "End",
    PageUp: "PgUp",
    PageDown: "PgDn",
    Insert: "Ins",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Backquote: "`",
    Comma: ",",
    Period: ".",
    Slash: "/",
    NumpadAdd: "Num +",
    NumpadSubtract: "Num -",
    NumpadMultiply: "Num *",
    NumpadDivide: "Num /",
    NumpadDecimal: "Num .",
  };
  return named[code] ?? code;
}

/** The caps of `shortcut`, modifiers first in a fixed order, then the key. */
export function shortcutCaps(shortcut: Shortcut, platform: Platform): string[] {
  const modifiers = MODIFIER_ORDER.filter((m) => shortcut.modifiers.includes(m)).map((m) =>
    modifierLabel(m, platform),
  );
  return [...modifiers, keyLabel(shortcut.code)];
}

/** `Ctrl+Shift+8`, for logs and tooltips; the UI draws caps instead. */
export function shortcutText(shortcut: Shortcut, platform: Platform): string {
  return shortcutCaps(shortcut, platform).join("+");
}
