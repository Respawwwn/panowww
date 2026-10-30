// What the overlay writes (spec §13.4): the counter pill and the end pill.
import type { OverlayView } from "../lib/generated/OverlayView";
import { interpolate, strings } from "../lib/strings";

export function counterText(taken: number, expected: number): string {
  return `${taken} / ${expected}`;
}

/** `Done · 78 Shots` or `Stopped · 47 / 78`. */
export function endText(view: Extract<OverlayView, { kind: "end" }>): string {
  return view.stopped
    ? interpolate(strings.overlay.stopped, { k: view.taken, n: view.expected })
    : interpolate(strings.overlay.done, { n: view.taken });
}

/** The filled share of the progress bar, 0..100. */
export function progressPercent(taken: number, expected: number): number {
  if (expected <= 0) {
    return 0;
  }
  return Math.min(100, (taken / expected) * 100);
}
