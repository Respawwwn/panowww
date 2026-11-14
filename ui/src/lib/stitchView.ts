// What the Stitch panel draws (spec §16.3), computed from the stitch store
// and the Session: the panel's mode, each Shot's cell state, the grid
// caption, the measured values as text. Pure, so the states are tested as
// data and the screen only lays them out.

import { shotLabel } from "./format";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { ShotRef } from "./generated/ShotRef";
import type { StartPole } from "./generated/StartPole";
import type { StitchProgress } from "./generated/StitchProgress";
import type { StitchLast, StitchRunning } from "./stitch.svelte";
import { interpolate, strings } from "./strings";

export type PanelMode = "idle" | "running" | "finished" | "failed";

/** Which panel to show, from the store's view of this Session's Stitch. */
export function panelMode(
  running: StitchRunning | null,
  last: StitchLast | null,
  entry: PanoramaEntry | null,
): PanelMode {
  if (running) {
    return "running";
  }
  if (last?.closing.event === "failed") {
    return "failed";
  }
  if (last?.closing.event === "finished" || entry) {
    return "finished";
  }
  return "idle";
}

export type CellState = "done" | "now" | "dim" | "warn" | "bad";

export interface CellStates {
  mode: PanelMode;
  progress: StitchProgress | null;
  /** Placed from the grid, after a finished Stitch. */
  unaligned: readonly ShotRef[];
  /** Without control points, after a failed Stitch. */
  unlinked: readonly ShotRef[];
}

/** How the cell of Shot (`row`, `col`) is drawn. */
export function cellState(row: number, col: number, states: CellStates): CellState {
  const named = (list: readonly ShotRef[]) => list.some((s) => s.row === row && s.col === col);
  switch (states.mode) {
    case "running": {
      const stage = states.progress?.stage;
      if (!stage || stage.stage === "read" || stage.stage === "overlaps") {
        return "dim";
      }
      if (stage.stage === "align") {
        if (row < stage.row) {
          return "done";
        }
        return row === stage.row ? "now" : "dim";
      }
      return "done";
    }
    case "finished":
      return named(states.unaligned) ? "warn" : "done";
    case "failed":
      return named(states.unlinked) ? "bad" : "done";
    case "idle":
      return "done";
  }
}

/** The caption beside *Shots*. */
export function gridCaption(
  states: CellStates,
  rows: number,
  shotsPerRow: number,
  linked: { linked: number; total: number } | null,
): string {
  const s = strings.stitch.shots;
  const n = rows * shotsPerRow;
  switch (states.mode) {
    case "running": {
      const stage = states.progress?.stage;
      if (!stage || stage.stage === "read" || stage.stage === "overlaps") {
        return s.reading;
      }
      if (stage.stage === "align") {
        return interpolate(s.aligning, {
          row: stage.row,
          rows: stage.rows,
          k: Math.min(n, (stage.row - 1) * shotsPerRow),
          n,
        });
      }
      return interpolate(s.aligned, { n });
    }
    case "failed":
      return linked ? interpolate(s.linked, { k: linked.linked, n: linked.total }) : "";
    case "finished":
    case "idle":
      return interpolate(s.count, { n });
  }
}

/** A degree value with one decimal, the sign dropped. */
export function deg(value: number): string {
  return Math.abs(value).toFixed(1);
}

/** The six rows of the *Measured from the Shots* card. */
export function measuredRows(entry: PanoramaEntry, startPole: StartPole): [string, string][] {
  const m = strings.stitch.measured;
  const pole = startPole === "nadir" ? strings.setup.grid.nadir : strings.setup.grid.zenith;
  const offset = entry.startPoleOffsetDeg;
  const poleText =
    Math.abs(offset) < 0.05
      ? interpolate(m.poleOn, { pole })
      : interpolate(offset > 0 ? m.poleShort : m.polePast, { deg: deg(offset), pole });
  const closing = entry.closingTurnDeg;
  const closingText =
    Math.abs(closing) < 0.05
      ? m.closes
      : interpolate(closing > 0 ? m.closingLong : m.closingShort, { deg: deg(closing) });
  return [
    [m.fov, interpolate(m.fovValue, { deg: deg(entry.fieldOfViewDeg) })],
    [m.turnStep, `${deg(entry.turnStepDeg)}°`],
    [m.pitchStep, `${deg(entry.pitchStepDeg)}°`],
    [m.startPole, poleText],
    [m.closing, closingText],
    [m.drift, interpolate(m.driftValue, { deg: (Math.ceil(entry.driftDeg * 10) / 10).toFixed(1) })],
  ];
}

/** `r07_c03, r07_c09 and r07_c12`: Shots named for the amber and red boxes. */
export function nameList(names: readonly string[], and: string): string {
  if (names.length <= 1) {
    return names[0] ?? "";
  }
  return `${names.slice(0, -1).join(", ")} ${and} ${names[names.length - 1]}`;
}

/** `rNN_cNN` of a Shot file name or ref. */
export function refLabel(ref: ShotRef): string {
  return shotLabel(ref.row, ref.col);
}

export function fileStem(file: string): string {
  return file.replace(/\.png$/, "");
}

/**
 * The Panorama's size before the Stitch, for `stitch.willWrite`: the
 * width is 2π × focal length, and at the usual 90° field of view the
 * focal length is half the Shot's width, so about π × that width, rounded
 * down to a multiple of 64 as the stitch crate does; the height is half.
 */
export function expectedSize(shotWidth: number): { w: number; h: number } {
  const w = Math.max(64, Math.floor((Math.PI * shotWidth) / 64) * 64);
  return { w, h: w / 2 };
}
