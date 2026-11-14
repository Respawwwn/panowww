// What the Session end screen offers (spec §11.7, §16.2): the actions row
// and the *Next* line depend on the Session's state, the Experimental
// switch, the Stitchers set and whether a Panorama exists. Pure, so the
// four states are tested as a table.
import type { SessionStatus } from "./generated/SessionStatus";
import type { Stitcher } from "./generated/Stitcher";

export type NextVariant = "stitchAndOpen" | "stitchOrDrop" | "openIn" | "drop";

/** Which `session.end.next.*` line a completed Session shows. */
export function nextVariant(stitchOn: boolean, stitchers: readonly Stitcher[]): NextVariant {
  const any = stitchers.length > 0;
  if (stitchOn) {
    return any ? "stitchAndOpen" : "stitchOrDrop";
  }
  return any ? "openIn" : "drop";
}

export type SessionAction =
  | { id: "stitch"; primary: true }
  | { id: "stitching"; primary: true }
  | { id: "previewPanorama"; primary: true }
  /** Opens the Stitch panel in its finished state; Stitch again lives there. */
  | { id: "stitchPanel"; primary: false }
  | { id: "openIn"; stitcher: Stitcher; primary: boolean }
  | { id: "zip"; primary: boolean }
  | { id: "reveal"; primary: false };

export interface SessionEndState {
  status: SessionStatus;
  /** `experimental.stitch` in Settings. */
  stitchOn: boolean;
  /** The Stitchers set in Settings, in display order. */
  stitchers: readonly Stitcher[];
  /** A Panorama is in the folder. */
  hasPanorama: boolean;
  /** A Stitch of this Session is in progress. */
  stitching: boolean;
}

/**
 * The actions row, left to right. The Stitch (or Preview Panorama) is
 * primary when the switch is on; otherwise the first Stitcher, or Zip
 * Session when none is set, as before the Stitch existed. Only a completed
 * Session gets the Stitch and the *Open in…* buttons. Once a Panorama
 * exists, the Stitch button opens the panel instead of stitching: the
 * panel owns *Stitch again* and its question.
 */
export function sessionActions(state: SessionEndState): SessionAction[] {
  const completed = state.status === "completed";
  const actions: SessionAction[] = [];
  const stitchable = completed && state.stitchOn;
  if (stitchable) {
    if (state.stitching) {
      actions.push({ id: "stitching", primary: true });
    } else if (state.hasPanorama) {
      actions.push({ id: "previewPanorama", primary: true });
      actions.push({ id: "stitchPanel", primary: false });
    } else {
      actions.push({ id: "stitch", primary: true });
    }
  }
  if (completed) {
    for (const [i, stitcher] of state.stitchers.entries()) {
      actions.push({ id: "openIn", stitcher, primary: !stitchable && i === 0 });
    }
  }
  const stitchers = completed ? state.stitchers.length : 0;
  actions.push({ id: "zip", primary: !stitchable && stitchers === 0 });
  actions.push({ id: "reveal", primary: false });
  return actions;
}
