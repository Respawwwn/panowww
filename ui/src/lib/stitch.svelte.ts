// The Stitch in progress and the last one that ended, from the `stitch`
// event stream (spec §16.5). The Session end screen reads the running
// percentage and the Panorama written; the Stitch panel reads everything.
// The reducer is pure so the event sequences are tested as data.
import { api } from "./api";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { StitchEvent } from "./generated/StitchEvent";
import type { StitchProgress } from "./generated/StitchProgress";

/** The closing events: one ends every Stitch. */
export type StitchClosing = Exclude<StitchEvent, { event: "progress" }>;

export interface StitchRunning {
  /** The Session being stitched; null until `stitch_phase` has said which. */
  folder: string | null;
  progress: StitchProgress | null;
  /** When this window first heard of the Stitch, epoch ms. */
  startedAt: number;
}

export interface StitchLast {
  folder: string | null;
  closing: StitchClosing;
  endedAt: number;
}

export interface StitchState {
  running: StitchRunning | null;
  last: StitchLast | null;
  /** The Panoramas written while this window was open, by folder. */
  entries: Record<string, PanoramaEntry>;
}

export const initialStitchState: StitchState = { running: null, last: null, entries: {} };

/** The state after `event`, `now` being the clock. */
export function reduceStitch(state: StitchState, event: StitchEvent, now: number): StitchState {
  if (event.event === "progress") {
    const { event: _tag, ...progress } = event;
    const running = state.running ?? { folder: null, progress: null, startedAt: now };
    return { ...state, running: { ...running, progress: progress as StitchProgress } };
  }
  const folder = state.running?.folder ?? null;
  const entries =
    event.event === "finished" && folder !== null
      ? { ...state.entries, [folder]: event.entry }
      : state.entries;
  return {
    running: null,
    last: { folder, closing: event, endedAt: now },
    entries,
  };
}

/** A Stitch starts: the folder is known before any event arrives. */
export function startStitch(state: StitchState, folder: string, now: number): StitchState {
  return { ...state, running: { folder, progress: null, startedAt: now } };
}

/** Whole percent of the Stitch, for `session.end.stitching`. */
export function percent(progress: StitchProgress | null): number {
  return Math.round((progress?.overall ?? 0) * 100);
}

let state = $state<StitchState>(initialStitchState);

export const stitch = {
  get running(): StitchRunning | null {
    return state.running;
  },
  get last(): StitchLast | null {
    return state.last;
  },
  /** Whether the Stitch in progress is the Session in `folder`. */
  isStitching(folder: string | null): boolean {
    return folder !== null && state.running?.folder === folder;
  },
  /** The Panorama written for `folder` while this window was open. */
  entryFor(folder: string | null): PanoramaEntry | null {
    return folder === null ? null : (state.entries[folder] ?? null);
  },
  /** The last closing event of `folder`'s Stitch, if it was the last one. */
  lastFor(folder: string | null): StitchClosing | null {
    return folder !== null && state.last?.folder === folder ? state.last.closing : null;
  },
  /** Listens to the window's stitch events; asks which Session is being stitched now. */
  async subscribe(): Promise<() => void> {
    const unlisten = await api.stitch.onEvent((event) => {
      state = reduceStitch(state, event, Date.now());
    });
    const phase = await api.stitch.phase().catch(() => null);
    if (phase?.phase === "running" && !state.running?.folder) {
      state = startStitch(state, phase.folder, Date.now());
    }
    return unlisten;
  },
  /** Asks for a Stitch; `false` when a run or another Stitch is in progress. */
  async start(folder: string): Promise<boolean> {
    const started = await api.stitch.start(folder);
    if (started) {
      state = startStitch(state, folder, Date.now());
    }
    return started;
  },
  cancel(): Promise<void> {
    return api.stitch.cancel();
  },
  /** The reducer's input, for the gallery and the tests. */
  apply(event: StitchEvent): void {
    state = reduceStitch(state, event, Date.now());
  },
};
