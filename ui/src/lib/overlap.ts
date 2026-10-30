// Check overlap maths in the window (spec §7.4, §11.8), mirroring
// core::overlap so the chip and the verdict follow the slider live.
import { strings } from "./strings";

/** The stitcher's target, in percent. */
export const OVERLAP_TARGET = { min: 25, max: 35 } as const;

export type Verdict = "ok" | "low" | "high";

/** `1 − |offset| / size`, in percent, clamped to 0–100. */
export function overlapPercent(offset: number, size: number): number {
  if (size <= 0) {
    return 0;
  }
  const share = 1 - Math.abs(offset) / size;
  return Math.min(1, Math.max(0, share)) * 100;
}

/** The verdict for an Overlap in percent; the bounds are inside the target. */
export function verdict(percent: number): Verdict {
  if (percent < OVERLAP_TARGET.min) {
    return "low";
  }
  if (percent > OVERLAP_TARGET.max) {
    return "high";
  }
  return "ok";
}

/** The verdict box text of §13.3 for the axis, with `{p}` left to fill. */
export function verdictText(v: Verdict, axis: "turn" | "pitch"): string {
  switch (v) {
    case "ok":
      return strings.overlap.verdict.ok;
    case "low":
      return strings.overlap.verdict.low[axis];
    case "high":
      return strings.overlap.verdict.high[axis];
  }
}
