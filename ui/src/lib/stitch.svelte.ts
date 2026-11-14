// The Stitch in progress and the last one that ended, from the `stitch`
// event stream (spec §16.5). The Session end screen reads the running
// percentage and the Panorama written; the Stitch panel reads everything.
// The reducer is pure so the event sequences are tested as data.
import { api } from "./api";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { Stage } from "./generated/Stage";
import type { StitchEvent } from "./generated/StitchEvent";
import type { StitchProgress } from "./generated/StitchProgress";

export type StageName = Stage["stage"];
/** The stages in order (spec §16.6). */
export const STAGES: readonly StageName[] = [
  "read",
  "overlaps",
  "align",
  "render",
  "blend",
  "save",
];
/** Share of the whole per stage, measured on the spike (spec §16.8). */
export const STAGE_SHARE: Readonly<Record<StageName, number>> = {
  read: 0.07,
  overlaps: 0.2,
  align: 0.3,
  render: 0.25,
  blend: 0.15,
  save: 0.03,
};

/** When a stage was first and last seen, epoch ms: its time in the panel. */
export interface StageSpan {
  startedAt: number;
  endedAt: number | null;
}

/** The closing events: one ends every Stitch. */
export type StitchClosing = Exclude<StitchEvent, { event: "progress" }>;

export interface StitchRunning {
  /** The Session being stitched; null until `stitch_phase` has said which. */
  folder: string | null;
  progress: StitchProgress | null;
  /** When this window first heard of the Stitch, epoch ms. */
  startedAt: number;
  /** The stages seen so far, timed from this window's clock. */
  stages: Partial<Record<StageName, StageSpan>>;
}

export interface StitchLast {
  folder: string | null;
  closing: StitchClosing;
  startedAt: number;
  endedAt: number;
  /** The stage times of that Stitch, for the finished and failed panels. */
  stages: Partial<Record<StageName, StageSpan>>;
  /** The last progress seen: which stage failed, how far it got. */
  progress: StitchProgress | null;
}

export interface StitchState {
  running: StitchRunning | null;
  last: StitchLast | null;
  /** The Panoramas written while this window was open, by folder. */
  entries: Record<string, PanoramaEntry>;
}

export const initialStitchState: StitchState = { running: null, last: null, entries: {} };

/** `stages` with the stage of `progress` open and every earlier one closed. */
function timeStages(
  stages: Partial<Record<StageName, StageSpan>>,
  progress: StitchProgress,
  now: number,
): Partial<Record<StageName, StageSpan>> {
  const next = { ...stages };
  const current = progress.stage.stage;
  for (const name of STAGES) {
    const span = next[name];
    if (name === current) {
      if (!span) {
        next[name] = { startedAt: now, endedAt: null };
      }
      break;
    }
    if (span && span.endedAt === null) {
      next[name] = { ...span, endedAt: now };
    }
  }
  return next;
}

/** Every open stage closed at `now`: the Stitch ended. */
function closeStages(
  stages: Partial<Record<StageName, StageSpan>>,
  now: number,
): Partial<Record<StageName, StageSpan>> {
  const next = { ...stages };
  for (const name of STAGES) {
    const span = next[name];
    if (span && span.endedAt === null) {
      next[name] = { ...span, endedAt: now };
    }
  }
  return next;
}

/** The state after `event`, `now` being the clock. */
export function reduceStitch(state: StitchState, event: StitchEvent, now: number): StitchState {
  if (event.event === "progress") {
    const { event: _tag, ...rest } = event;
    const progress = rest as StitchProgress;
    const running = state.running ?? { folder: null, progress: null, startedAt: now, stages: {} };
    return {
      ...state,
      running: { ...running, progress, stages: timeStages(running.stages, progress, now) },
    };
  }
  const folder = state.running?.folder ?? null;
  const entries =
    event.event === "finished" && folder !== null
      ? { ...state.entries, [folder]: event.entry }
      : state.entries;
  return {
    running: null,
    last: {
      folder,
      closing: event,
      startedAt: state.running?.startedAt ?? now,
      endedAt: now,
      stages: closeStages(state.running?.stages ?? {}, now),
      progress: state.running?.progress ?? null,
    },
    entries,
  };
}

/** A Stitch starts: the folder is known before any event arrives. */
export function startStitch(state: StitchState, folder: string, now: number): StitchState {
  return { ...state, running: { folder, progress: null, startedAt: now, stages: {} } };
}

/**
 * Milliseconds left, from the stage shares refined by the elapsed time:
 * once a Stitch is a few percent in, its own pace predicts the rest better
 * than the spike's ratios; before that there is nothing to say.
 */
export function remainingMs(elapsedMs: number, overall: number): number | null {
  if (overall < 0.03 || elapsedMs <= 0) {
    return null;
  }
  const total = elapsedMs / Math.min(overall, 1);
  return Math.max(0, total - elapsedMs);
}

/** Milliseconds a stage took, or has taken so far at `now`. */
export function stageMs(span: StageSpan | undefined, now: number): number | null {
  if (!span) {
    return null;
  }
  return (span.endedAt ?? now) - span.startedAt;
}

/** How each stage is drawn in the panel's list. */
export type StageState = "done" | "now" | "todo" | "bad";

/**
 * The state of `name` given the stage in progress (or the one that failed
 * when `failed`), or none when the Stitch has not started.
 */
export function stageState(
  name: StageName,
  current: StageName | null,
  failed: boolean,
): StageState {
  if (current === null) {
    return "todo";
  }
  const i = STAGES.indexOf(name);
  const c = STAGES.indexOf(current);
  if (i < c) {
    return "done";
  }
  if (i === c) {
    return failed ? "bad" : "now";
  }
  return "todo";
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
  /** The last Stitch of `folder`, if it was the last one to end. */
  lastFor(folder: string | null): StitchLast | null {
    return folder !== null && state.last?.folder === folder ? state.last : null;
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
