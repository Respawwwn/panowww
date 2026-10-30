import { describe, expect, test } from "vitest";
import type { Shortcut } from "./generated/Shortcut";
import { keyLabel, shortcutCaps, shortcutText } from "./shortcut";

const start: Shortcut = { modifiers: ["ctrl", "shift"], code: "Digit8" };

describe("shortcutCaps", () => {
  test("renders the default Start shortcut", () => {
    expect(shortcutCaps(start, "macos")).toEqual(["Ctrl", "Shift", "8"]);
    expect(shortcutText(start, "windows")).toBe("Ctrl+Shift+8");
  });

  test("orders modifiers and names them per platform", () => {
    const combo: Shortcut = { modifiers: ["meta", "shift", "alt"], code: "KeyP" };
    expect(shortcutCaps(combo, "macos")).toEqual(["Option", "Shift", "Cmd", "P"]);
    expect(shortcutCaps(combo, "windows")).toEqual(["Alt", "Shift", "Win", "P"]);
  });
});

describe("keyLabel", () => {
  test("maps codes to caps", () => {
    expect(keyLabel("KeyW")).toBe("W");
    expect(keyLabel("Digit0")).toBe("0");
    expect(keyLabel("Numpad8")).toBe("Num 8");
    expect(keyLabel("ArrowUp")).toBe("↑");
    expect(keyLabel("F12")).toBe("F12");
    expect(keyLabel("Space")).toBe("Space");
    expect(keyLabel("BracketLeft")).toBe("[");
    expect(keyLabel("Unknown")).toBe("Unknown");
  });
});
