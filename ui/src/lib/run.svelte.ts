// The run in progress and the last one that ended, from the `run` event
// stream (spec §4.4). Every screen reads from here: the Calibrate card
// for its result lines, the countdown and Session views (issue 17).
import { api } from "./api";
import type { RunEvent } from "./generated/RunEvent";
import type { RunKindDto } from "./generated/RunKindDto";
import type { RunOutcome } from "./generated/RunOutcome";
import type { StartFailure } from "./generated/StartFailure";
import type { Zone } from "./generated/Zone";

export interface ShotTaken {
  index: number;
  row: number;
  col: number;
  captureMs: number;
  thumbnail: string | null;
}

export interface Preview {
  image: string;
  displayW: number;
  displayH: number;
  zone: Zone;
}

/** What is going on right now. */
export interface CurrentRun {
  gameId: string;
  kind: RunKindDto;
  phase: "countdown" | "running";
  secondsLeft: number;
  expected: number;
  shots: ShotTaken[];
  /** Steps performed so far. */
  steps: number;
  row: number;
  col: number;
  screenUnchanged: boolean;
  samples: { first: string | null; second: string | null };
  preview: Preview | null;
  /** When the run proper started (after the countdown), epoch ms. */
  startedAt: number | null;
}

/** How the last run ended, for result lines and the end screen. */
export interface LastRun {
  gameId: string;
  kind: RunKindDto;
  steps: number;
  shots: ShotTaken[];
  screenUnchanged: boolean;
  samples: { first: string | null; second: string | null };
  startedAt: number | null;
  endedAt: number;
  ended:
    | { by: "finished"; outcome: RunOutcome; folder: string | null }
    | { by: "cancelled" }
    | { by: "cannotStart"; failure: StartFailure };
}

let current = $state<CurrentRun | null>(null);
let last = $state<LastRun | null>(null);
/** The run the UI asked for, so a `countdown` event is attributed. */
let requested: { gameId: string; kind: RunKindDto } | null = null;

function fresh(gameId: string, kind: RunKindDto): CurrentRun {
  return {
    gameId,
    kind,
    phase: "countdown",
    secondsLeft: 0,
    expected: 0,
    shots: [],
    steps: 0,
    row: 1,
    col: 1,
    screenUnchanged: false,
    samples: { first: null, second: null },
    preview: null,
    startedAt: null,
  };
}

function end(ended: LastRun["ended"]) {
  const c = current ?? fresh(requested?.gameId ?? "", requested?.kind ?? { kind: "session" });
  if (c) {
    last = {
      gameId: c.gameId,
      kind: c.kind,
      steps: c.steps,
      shots: c.shots,
      screenUnchanged: c.screenUnchanged,
      samples: c.samples,
      startedAt: c.startedAt,
      endedAt: Date.now(),
      ended,
    };
  }
  current = null;
  requested = null;
}

export function applyRunEvent(event: RunEvent): void {
  switch (event.event) {
    case "cannotStart": {
      // ts-rs flattens the newtype variant: drop the tag, keep the failure.
      const { event: _tag, ...failure } = event;
      end({ by: "cannotStart", failure: failure as StartFailure });
      return;
    }
    case "countdown":
      if (!current) {
        // Started by the shortcut: a Session of the selected Game.
        current = fresh(requested?.gameId ?? "", requested?.kind ?? { kind: "session" });
      }
      current.phase = "countdown";
      current.secondsLeft = event.secondsLeft;
      return;
    case "cancelled":
      end({ by: "cancelled" });
      return;
    case "started":
      if (!current) {
        current = fresh(requested?.gameId ?? "", requested?.kind ?? { kind: "session" });
      }
      current.phase = "running";
      current.expected = event.expected;
      current.startedAt = Date.now();
      return;
    case "shot":
      current?.shots.push({
        index: event.index,
        row: event.row,
        col: event.col,
        captureMs: event.captureMs,
        thumbnail: event.thumbnail,
      });
      return;
    case "moved":
      if (current) {
        current.steps += 1;
        current.row = event.row;
        current.col = event.col;
      }
      return;
    case "sample":
      if (current) {
        current.samples[event.slot] = event.image;
      }
      return;
    case "screenUnchanged":
      if (current) {
        current.screenUnchanged = true;
      }
      return;
    case "preview":
      if (current) {
        current.preview = {
          image: event.image,
          displayW: event.displayW,
          displayH: event.displayH,
          zone: event.zone,
        };
      }
      return;
    case "finished":
      end({ by: "finished", outcome: event.outcome, folder: event.folder });
      return;
  }
}

export const run = {
  get current(): CurrentRun | null {
    return current;
  },
  get last(): LastRun | null {
    return last;
  },
  get busy(): boolean {
    return current !== null;
  },

  /** Listens to the window's run events until the returned function is called. */
  subscribe(): Promise<() => void> {
    return api.run.onEvent(applyRunEvent);
  },

  /** Asks for a run; `false` when one is in progress. */
  async start(gameId: string, kind: RunKindDto): Promise<boolean> {
    requested = { gameId, kind };
    const started = await api.run.start(gameId, kind);
    if (!started) {
      requested = null;
    }
    return started;
  },

  /** The shortcut tells no one which Game: the sidebar's selection does. */
  expectShortcut(gameId: string | null) {
    requested = gameId ? { gameId, kind: { kind: "session" } } : null;
  },

  /** What the next events belong to, for tests. */
  expect(gameId: string, kind: RunKindDto) {
    requested = { gameId, kind };
  },

  stop(): Promise<void> {
    return api.run.stop();
  },

  /** Forgets the last result (on a Game switch). */
  clearLast() {
    last = null;
  },

  /** For tests. */
  reset() {
    current = null;
    last = null;
    requested = null;
  },
};
