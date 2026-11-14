// The Panorama preview window (spec §16.4): which Panorama it shows,
// whether it fills the app window, and whether the 360° viewer gave up
// (no WebGL 2) and the flat image is shown instead. The reducer is pure so
// the states are tested as data; the store wraps it for the screens.
import type { Game } from "./generated/Game";
import type { PanoramaEntry } from "./generated/PanoramaEntry";

export interface PreviewSubject {
  folder: string;
  entry: PanoramaEntry;
  /** For the title bar: the Game's name and when the Session was shot. */
  gameName: string;
  startedAt: number | null;
  startPole: Game["startPole"];
}

export interface PreviewState {
  subject: PreviewSubject | null;
  /** *Fill the window*: the viewer takes the whole app window. */
  fill: boolean;
  /** The viewer could not start: the flat preview image is shown. */
  fallback: boolean;
}

export type PreviewAction =
  | { type: "open"; subject: PreviewSubject }
  | { type: "close" }
  | { type: "toggleFill" }
  | { type: "viewerFailed" };

export const initialPreviewState: PreviewState = { subject: null, fill: false, fallback: false };

/** The state after `action`. Opening resets the fill and the fallback. */
export function reducePreview(state: PreviewState, action: PreviewAction): PreviewState {
  switch (action.type) {
    case "open":
      return { subject: action.subject, fill: false, fallback: false };
    case "close":
      return { ...state, subject: null, fill: false };
    case "toggleFill":
      return state.subject ? { ...state, fill: !state.fill } : state;
    case "viewerFailed":
      return state.subject ? { ...state, fallback: true } : state;
  }
}

let state = $state<PreviewState>(initialPreviewState);

export const preview = {
  get subject(): PreviewSubject | null {
    return state.subject;
  },
  get fill(): boolean {
    return state.fill;
  },
  get fallback(): boolean {
    return state.fallback;
  },
  show(subject: PreviewSubject) {
    state = reducePreview(state, { type: "open", subject });
  },
  close() {
    state = reducePreview(state, { type: "close" });
  },
  toggleFill() {
    state = reducePreview(state, { type: "toggleFill" });
  },
  viewerFailed() {
    state = reducePreview(state, { type: "viewerFailed" });
  },
};
