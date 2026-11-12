import { describe, expect, test } from "vitest";
import { overlapPercent, verdict, verdictText } from "./overlap";

describe("slider → Overlap → verdict", () => {
  test("percent is one minus offset over size, either direction", () => {
    expect(overlapPercent(0, 1000)).toBe(100);
    expect(overlapPercent(700, 1000)).toBeCloseTo(30);
    expect(overlapPercent(-700, 1000)).toBeCloseTo(30);
    expect(overlapPercent(1500, 1000)).toBe(0);
    expect(overlapPercent(10, 0)).toBe(0);
  });

  test("the target bounds are inclusive", () => {
    expect(verdict(25)).toBe("ok");
    expect(verdict(35)).toBe("ok");
    expect(verdict(24.9)).toBe("low");
    expect(verdict(35.1)).toBe("high");
  });

  test("the verdict names the Step button to press per axis", () => {
    expect(verdictText("ok", "turn")).toContain("target");
    expect(verdictText("low", "turn")).toContain("turn Step is too big → Overshot (−)");
    expect(verdictText("low", "pitch")).toContain("pitch Step is too big → Overshot (−)");
    expect(verdictText("high", "turn")).toContain("turn Step is too small → Undershot (+)");
    expect(verdictText("high", "pitch")).toContain("pitch Step is too small → Undershot (+)");
  });

  test("a slider position maps end to end", () => {
    const size = 1920;
    const offset = 1344; // the second Sample slid 70 % of the width
    const p = overlapPercent(offset, size);
    expect(Math.round(p)).toBe(30);
    expect(verdict(p)).toBe("ok");
  });
});
