import { describe, expect, test } from "vitest";
import { interpolate, splitKeyCaps, splitPlaceholders, strings } from "./strings";

describe("interpolate", () => {
  test("fills every placeholder", () => {
    expect(
      interpolate(strings.setup.start.estimateDetail, {
        shots: 78,
        rows: 6,
        shotsPerRow: 13,
        pole: "nadir",
        move: 400,
        shot: 250,
      }),
    ).toBe("78 Shots · 6 Rows × 13 · from nadir · wait 400 / 250 ms");
  });

  test("leaves a missing value visible", () => {
    expect(interpolate("Display {n} · {w} × {h}", { n: 1 })).toBe("Display 1 · {w} × {h}");
  });

  test("ignores shortcut markers", () => {
    expect(interpolate(strings.countdown.stop)).toBe("Stop at any time with [stop]");
  });
});

describe("splitKeyCaps", () => {
  test("splits around the shortcut markers", () => {
    expect(splitKeyCaps(strings.session.running.hint)).toEqual([
      {
        kind: "text",
        text: "The game keeps the focus. Do not touch the mouse or the keyboard. Stop with ",
      },
      { kind: "keys", shortcut: "stop" },
    ]);
  });

  test("keeps a marker at the end and plain text as is", () => {
    expect(splitKeyCaps(strings.setup.start.shortcutStart)).toEqual([
      { kind: "text", text: "Start from the game  " },
      { kind: "keys", shortcut: "start" },
    ]);
    expect(splitKeyCaps("No caps [here]")).toEqual([{ kind: "text", text: "No caps [here]" }]);
  });
});

describe("splitPlaceholders", () => {
  test("cuts around the named slots and leaves the others as text", () => {
    expect(
      splitPlaceholders(strings.session.end.next.stitchAndOpen, ["stitchHere", "stitchers"]),
    ).toEqual([
      { kind: "text", text: "Next: " },
      { kind: "slot", name: "stitchHere" },
      { kind: "text", text: ", or open it in " },
      { kind: "slot", name: "stitchers" },
      { kind: "text", text: "." },
    ]);
    expect(splitPlaceholders("Display {n} · {w}", ["w"])).toEqual([
      { kind: "text", text: "Display {n} · " },
      { kind: "slot", name: "w" },
    ]);
    expect(splitPlaceholders(strings.session.end.next.drop, ["stitchHere"])).toEqual([
      { kind: "text", text: strings.session.end.next.drop },
    ]);
  });
});

describe("strings", () => {
  const leaves: string[] = [];
  const walk = (node: unknown) => {
    if (typeof node === "string") {
      leaves.push(node);
    } else if (node && typeof node === "object") {
      for (const value of Object.values(node)) {
        walk(value);
      }
    }
  };
  walk(strings);

  test("never shout", () => {
    for (const text of leaves) {
      expect(text, text).not.toMatch(/!/);
    }
  });

  test("never write a shortcut out", () => {
    for (const text of leaves) {
      expect(text, text).not.toMatch(/Ctrl\+Shift/);
    }
  });
});
