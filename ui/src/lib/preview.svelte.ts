// The Panorama preview window (spec §16.4): which Panorama it shows,
// whether it fills the screen, and whether the 360° viewer gave up
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
  /** *Fill the screen*: the app window takes the screen, the viewer the window. */
  fullScreen: boolean;
  /** The viewer could not start: the flat preview image is shown. */
  fallback: boolean;
}

export type PreviewAction =
  | { type: "open"; subject: PreviewSubject }
  | { type: "close" }
  | { type: "toggleFullScreen" }
  | { type: "viewerFailed" };

export const initialPreviewState: PreviewState = {
  subject: null,
  fullScreen: false,
  fallback: false,
};

/** The state after `action`. Opening resets the full screen and the fallback. */
export function reducePreview(state: PreviewState, action: PreviewAction): PreviewState {
  switch (action.type) {
    case "open":
      return { subject: action.subject, fullScreen: false, fallback: false };
    case "close":
      return { ...state, subject: null, fullScreen: false };
    case "toggleFullScreen":
      return state.subject ? { ...state, fullScreen: !state.fullScreen } : state;
    case "viewerFailed":
      return state.subject ? { ...state, fallback: true } : state;
  }
}

let state = $state<PreviewState>(initialPreviewState);

export const preview = {
  get subject(): PreviewSubject | null {
    return state.subject;
  },
  get fullScreen(): boolean {
    return state.fullScreen;
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
  toggleFullScreen() {
    state = reducePreview(state, { type: "toggleFullScreen" });
  },
  viewerFailed() {
    state = reducePreview(state, { type: "viewerFailed" });
  },
};
