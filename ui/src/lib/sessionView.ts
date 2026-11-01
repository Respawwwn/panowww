// What the Session end screen shows (spec §11.7), built either from the
// run that just ended (live thumbnails in memory) or from a Session folder
// on disk (history, thumbnails generated on demand).
import type { Game } from "./generated/Game";
import type { RunError } from "./generated/RunError";
import type { SessionRecord } from "./generated/SessionRecord";
import type { SessionStatus } from "./generated/SessionStatus";
import type { LastRun } from "./run.svelte";

export interface SessionShot {
  row: number;
  col: number;
  /** `rNN_cNN.png`, for the history thumbnail. */
  file: string;
  /** A data: URL when already in memory. */
  src: string | null;
}

export interface SessionView {
  status: SessionStatus;
  game: Game;
  folder: string | null;
  startedAt: number | null;
  endedAt: number | null;
  shotsExpected: number;
  /** Shots on disk; the grid lists `shots`, which an old record may lack. */
  shotsTaken: number;
  shots: SessionShot[];
  error: RunError | null;
  /** The display the zone was on, for the failed reason. */
  displayId: string | null;
}

function file(row: number, col: number): string {
  return `r${String(row).padStart(2, "0")}_c${String(col).padStart(2, "0")}.png`;
}

/** The run that just ended, as the flow hands it over. */
export function viewFromLast(last: LastRun, game: Game, folder: string | null): SessionView | null {
  if (last.ended.by !== "finished" || last.kind.kind !== "session") {
    return null;
  }
  const outcome = last.ended.outcome;
  return {
    status: outcome.status,
    game,
    folder,
    startedAt: last.startedAt,
    endedAt: last.endedAt,
    shotsExpected: game.rows * game.shotsPerRow,
    shotsTaken: last.shots.length,
    shots: last.shots.map((s) => ({
      row: s.row,
      col: s.col,
      file: file(s.row, s.col),
      src: s.thumbnail,
    })),
    error: outcome.status === "failed" ? outcome.error : null,
    displayId: null,
  };
}

/** A Session folder's record, opened from a table. */
export function viewFromRecord(record: SessionRecord, folder: string): SessionView {
  const at = (s: string | undefined): number | null => {
    if (!s) {
      return null;
    }
    const ms = new Date(s).getTime();
    return Number.isNaN(ms) ? null : ms;
  };
  return {
    status: record.status,
    game: record.game,
    folder,
    startedAt: at(record.startedAt),
    endedAt: at(record.endedAt),
    shotsExpected: record.shotsExpected,
    shotsTaken: Math.max(record.shotsTaken, record.shots.length),
    shots: record.shots.map((s) => ({ row: s.row, col: s.col, file: s.file, src: null })),
    error: record.error ?? null,
    displayId: record.zone.displayId,
  };
}
