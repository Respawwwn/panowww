import { describe, expect, test } from "vitest";
import { shortcutFromKeyEvent, shortcutProblem } from "./recorder";

const press = (
  code: string,
  mods: Partial<Record<"ctrlKey" | "shiftKey" | "altKey" | "metaKey", boolean>> = {},
) => ({
  code,
  ctrlKey: false,
  shiftKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe("shortcutFromKeyEvent", () => {
  test("waits while only modifiers are down", () => {
    expect(shortcutFromKeyEvent(press("ShiftLeft", { shiftKey: true }))).toBeNull();
    expect(shortcutFromKeyEvent(press("ControlLeft", { ctrlKey: true }))).toBeNull();
    expect(shortcutFromKeyEvent(press(""))).toBeNull();
  });

  test("records the key with its modifiers in a fixed order", () => {
    expect(shortcutFromKeyEvent(press("Digit8", { ctrlKey: true, shiftKey: true }))).toEqual({
      modifiers: ["ctrl", "shift"],
      code: "Digit8",
    });
    expect(shortcutFromKeyEvent(press("KeyP", { metaKey: true, altKey: true }))).toEqual({
      modifiers: ["alt", "meta"],
      code: "KeyP",
    });
  });
});

describe("shortcutProblem", () => {
  test("accepts the defaults", () => {
    expect(shortcutProblem({ modifiers: ["ctrl", "shift"], code: "Digit8" }, "macos")).toBeNull();
    expect(shortcutProblem({ modifiers: ["ctrl", "shift"], code: "Digit9" }, "windows")).toBeNull();
  });

  test("refuses F12, bare keys, the Win key and Option-only combos", () => {
    expect(shortcutProblem({ modifiers: ["ctrl"], code: "F12" }, "macos")).toMatch(/F12/);
    expect(shortcutProblem({ modifiers: [], code: "KeyA" }, "macos")).toMatch(/modifier/i);
    expect(shortcutProblem({ modifiers: ["meta"], code: "KeyA" }, "windows")).toMatch(/Win/);
    expect(shortcutProblem({ modifiers: ["alt"], code: "KeyA" }, "macos")).toMatch(/Option/);
    expect(shortcutProblem({ modifiers: ["alt", "shift"], code: "KeyA" }, "macos")).toMatch(
      /Option/,
    );
    expect(shortcutProblem({ modifiers: ["alt", "ctrl"], code: "KeyA" }, "macos")).toBeNull();
    expect(shortcutProblem({ modifiers: ["meta", "shift"], code: "KeyA" }, "macos")).toBeNull();
  });
});
