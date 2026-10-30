import { describe, expect, it } from "vitest";
import { anchorOf, between, clampPoint, clampRect, isUsable, moved, withField } from "./geometry";

const display = { w: 2560, h: 1440 };

describe("picker geometry", () => {
  it("draws the rectangle between two corners in any order", () => {
    expect(between({ x: 100, y: 50 }, { x: 10, y: 80 })).toEqual({ x: 10, y: 50, w: 90, h: 30 });
    expect(between({ x: 10.4, y: 10.6 }, { x: 10.4, y: 10.6 })).toEqual({
      x: 10,
      y: 11,
      w: 0,
      h: 0,
    });
  });

  it("keeps points and rectangles on the display", () => {
    expect(clampPoint({ x: -5, y: 2000 }, display)).toEqual({ x: 0, y: 1440 });
    expect(clampRect({ x: 2500, y: -10, w: 200, h: 100 }, display)).toEqual({
      x: 2360,
      y: 0,
      w: 200,
      h: 100,
    });
    expect(clampRect({ x: 0, y: 0, w: 9000, h: 9000 }, display)).toEqual({
      x: 0,
      y: 0,
      w: 2560,
      h: 1440,
    });
  });

  it("resizes from the corner opposite the handle", () => {
    const r = { x: 100, y: 100, w: 300, h: 200 };
    expect(anchorOf(r, "nw")).toEqual({ x: 400, y: 300 });
    expect(anchorOf(r, "se")).toEqual({ x: 100, y: 100 });
    expect(between(anchorOf(r, "se"), { x: 500, y: 150 })).toEqual({
      x: 100,
      y: 100,
      w: 400,
      h: 50,
    });
  });

  it("moves without leaving the display", () => {
    expect(moved({ x: 100, y: 100, w: 300, h: 200 }, 3000, -500, display)).toEqual({
      x: 2260,
      y: 0,
      w: 300,
      h: 200,
    });
  });

  it("drops a click-sized drag", () => {
    expect(isUsable({ x: 0, y: 0, w: 7, h: 100 })).toBe(false);
    expect(isUsable({ x: 0, y: 0, w: 8, h: 8 })).toBe(true);
    expect(isUsable(null)).toBe(false);
  });

  it("applies one field at a time and ignores junk", () => {
    const r = { x: 100, y: 100, w: 300, h: 200 };
    expect(withField(r, "x", "2400", display)).toEqual({ x: 2260, y: 100, w: 300, h: 200 });
    expect(withField(r, "w", "2", display)).toEqual({ x: 100, y: 100, w: 8, h: 200 });
    expect(withField(r, "h", "abc", display)).toEqual(r);
  });
});
