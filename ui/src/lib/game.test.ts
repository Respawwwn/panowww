import { describe, expect, test } from "vitest";
import {
  adjustCount,
  adjustedField,
  defaultInput,
  disabledReason,
  flashOf,
  gridSummary,
  inputSummary,
} from "./game";
import type { Game } from "./generated/Game";

const game: Game = {
  format: "panowww-game",
  version: 1,
  id: "3f6c1e2a-0000-4000-8000-000000000001",
  name: "Elden Ring",
  startPole: "nadir",
  shotsPerRow: 13,
  rows: 6,
  waitAfterMoveMs: 300,
  waitAfterShotMs: 300,
  input: defaultInput("keyboard"),
};

describe("defaultInput", () => {
  test("mirrors the Rust defaults per device", () => {
    expect(defaultInput("keyboard")).toEqual({
      device: "keyboard",
      turnRightKey: "ArrowRight",
      lookUpKey: "ArrowUp",
      lookDownKey: "ArrowDown",
      turnHoldMs: 100,
      pitchHoldMs: 100,
    });
    expect(defaultInput("mouse")).toMatchObject({ device: "mouse", mode: "move", turnDelta: 500 });
    expect(defaultInput("mouse")).not.toHaveProperty("dragButton");
    expect(defaultInput("gamepad")).toMatchObject({ device: "gamepad", stickDeflection: 0.8 });
  });
});

describe("summaries", () => {
  test("say the device and its numbers", () => {
    expect(inputSummary(defaultInput("keyboard"))).toBe("Keyboard · 100 / 100 ms");
    expect(inputSummary({ ...defaultInput("mouse"), mode: "drag", dragButton: "right" })).toBe(
      "Mouse · drag right · 500 / 500",
    );
    expect(gridSummary(game)).toBe("13 × 6 · 300 / 300 ms");
  });
});

describe("disabledReason", () => {
  test("gamepad on macOS beats a missing zone", () => {
    const gamepad = { ...game, input: defaultInput("gamepad") };
    expect(disabledReason(gamepad, false, "macos")).toBe("gamepadMac");
    expect(disabledReason(gamepad, true, "windows")).toBeNull();
    expect(disabledReason(game, false, "macos")).toBe("noZone");
    expect(disabledReason(game, true, "macos")).toBeNull();
  });
});

describe("adjustCount", () => {
  test("moves Shots per Row on the turn axis and Rows on the pitch axis", () => {
    expect(adjustCount(game, "turn", 3)).toEqual({ shotsPerRow: 16 });
    expect(adjustCount(game, "turn", -1)).toEqual({ shotsPerRow: 12 });
    expect(adjustCount(game, "pitch", 1)).toEqual({ rows: 7 });
    expect(adjustCount(game, "pitch", -3)).toEqual({ rows: 3 });
  });

  test("clamps to the schema bounds", () => {
    expect(adjustCount({ ...game, shotsPerRow: 4 }, "turn", -3)).toEqual({ shotsPerRow: 3 });
    expect(adjustCount({ ...game, shotsPerRow: 71 }, "turn", 3)).toEqual({ shotsPerRow: 72 });
    expect(adjustCount({ ...game, rows: 2 }, "pitch", -1)).toEqual({ rows: 2 });
    expect(adjustCount({ ...game, rows: 35 }, "pitch", 3)).toEqual({ rows: 36 });
  });
});

describe("adjustedField and flashOf", () => {
  test("holds for keyboard and gamepad, deltas for mouse", () => {
    expect(adjustedField(defaultInput("keyboard"), "turn")).toBe("turnHoldMs");
    expect(adjustedField(defaultInput("gamepad"), "pitch")).toBe("pitchHoldMs");
    expect(adjustedField(defaultInput("mouse"), "turn")).toBe("turnDelta");
    expect(adjustedField(defaultInput("mouse"), "pitch")).toBe("pitchDelta");
  });

  test("the counter reaches its field only", () => {
    expect(flashOf({ field: "rows", n: 3 }, "rows")).toBe(3);
    expect(flashOf({ field: "rows", n: 3 }, "shotsPerRow")).toBe(0);
    expect(flashOf(null, "rows")).toBe(0);
  });
});
