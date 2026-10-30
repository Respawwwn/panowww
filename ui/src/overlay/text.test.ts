import { describe, expect, it } from "vitest";
import { counterText, endText, progressPercent } from "./text";

describe("overlay text", () => {
  it("writes the counter and the end pills", () => {
    expect(counterText(17, 78)).toBe("17 / 78");
    expect(endText({ kind: "end", stopped: false, taken: 78, expected: 78 })).toBe(
      "Done · 78 Shots",
    );
    expect(endText({ kind: "end", stopped: true, taken: 47, expected: 78 })).toBe(
      "Stopped · 47 / 78",
    );
  });

  it("fills the bar from the Shots taken over the expected", () => {
    expect(progressPercent(0, 78)).toBe(0);
    expect(progressPercent(39, 78)).toBe(50);
    expect(progressPercent(80, 78)).toBe(100);
    expect(progressPercent(3, 0)).toBe(0);
  });
});
