import { describe, expect, test } from "vitest";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { StitchLast, StitchRunning } from "./stitch.svelte";
import {
  type CellStates,
  cellState,
  expectedSize,
  gridCaption,
  measuredOfEntry,
  measuredRows,
  nameList,
  panelMode,
} from "./stitchView";

const entry: PanoramaEntry = {
  file: "panorama.jpg",
  preview: "panorama-preview.jpg",
  width: 5184,
  height: 2592,
  bytes: 5_100_000,
  stitchedAt: "2026-11-14T16:00:00+01:00",
  durationMs: 9800,
  appVersion: "1.1.0",
  fieldOfViewDeg: 92.43,
  turnStepDeg: 25.64,
  pitchStepDeg: 29.4,
  startPoleOffsetDeg: 1.8,
  closingTurnDeg: 0.4,
  driftDeg: 0.26,
  unaligned: [],
  droppedSteps: [],
  antiGhost: true,
};

const running = (stage: StitchRunning["progress"]): StitchRunning => ({
  folder: "/S/a",
  progress: stage,
  startedAt: 0,
  stages: {},
  measured: null,
});

const last = (closing: StitchLast["closing"]): StitchLast => ({
  folder: "/S/a",
  closing,
  startedAt: 0,
  endedAt: 1,
  stages: {},
  progress: null,
  measured: null,
});

describe("the panel's mode", () => {
  test("follows the store, and a Panorama on disk counts as finished", () => {
    expect(panelMode(null, null, null)).toBe("idle");
    expect(panelMode(running(null), null, null)).toBe("running");
    expect(panelMode(null, last({ event: "finished", entry }), null)).toBe("finished");
    expect(panelMode(null, null, entry)).toBe("finished");
    expect(panelMode(null, last({ event: "failed", error: { kind: "notCompleted" } }), entry)).toBe(
      "failed",
    );
    expect(panelMode(null, last({ event: "cancelled", reason: "button" }), null)).toBe("idle");
  });
});

describe("the cells", () => {
  const base: CellStates = { mode: "running", progress: null, unaligned: [], unlinked: [] };
  const at = (progress: CellStates["progress"]) => ({ ...base, progress });

  test("are dimmed while reading, framed on the Row being aligned, plain after", () => {
    expect(cellState(3, 2, at(null))).toBe("dim");
    expect(cellState(3, 2, at({ stage: { stage: "read" }, stageFraction: 0, overall: 0 }))).toBe(
      "dim",
    );
    const align = at({
      stage: { stage: "align", row: 3, rows: 7 },
      stageFraction: 0.3,
      overall: 0.4,
    });
    expect(cellState(2, 5, align)).toBe("done");
    expect(cellState(3, 5, align)).toBe("now");
    expect(cellState(4, 5, align)).toBe("dim");
    expect(cellState(7, 1, at({ stage: { stage: "blend" }, stageFraction: 0, overall: 0.8 }))).toBe(
      "done",
    );
  });

  test("turn amber when placed from the grid and red when unlinked", () => {
    const finished: CellStates = { ...base, mode: "finished", unaligned: [{ row: 7, col: 3 }] };
    expect(cellState(7, 3, finished)).toBe("warn");
    expect(cellState(7, 4, finished)).toBe("done");
    const failed: CellStates = { ...base, mode: "failed", unlinked: [{ row: 1, col: 1 }] };
    expect(cellState(1, 1, failed)).toBe("bad");
    expect(cellState(1, 2, failed)).toBe("done");
    expect(cellState(1, 1, { ...base, mode: "idle" })).toBe("done");
  });
});

describe("the grid caption", () => {
  const base: CellStates = { mode: "running", progress: null, unaligned: [], unlinked: [] };

  test("tells where the Stitch is", () => {
    expect(gridCaption(base, 7, 14, null)).toBe("reading and comparing…");
    expect(
      gridCaption(
        {
          ...base,
          progress: { stage: { stage: "align", row: 3, rows: 7 }, stageFraction: 0, overall: 0.4 },
        },
        7,
        14,
        null,
      ),
    ).toBe("Row 3 of 7 · 28 / 98 Shots aligned");
    expect(
      gridCaption(
        { ...base, progress: { stage: { stage: "render" }, stageFraction: 0, overall: 0.6 } },
        7,
        14,
        null,
      ),
    ).toBe("98 / 98 Shots aligned");
    expect(gridCaption({ ...base, mode: "idle" }, 7, 14, null)).toBe("98 Shots");
    expect(gridCaption({ ...base, mode: "failed" }, 7, 14, { linked: 37, total: 98 })).toBe(
      "37 / 98 Shots linked",
    );
  });
});

describe("the measured card", () => {
  test("reads the entry in words, poles and closing turns signed", () => {
    expect(measuredRows(measuredOfEntry(entry), "nadir")).toEqual([
      ["Field of view", "92.4° wide"],
      ["Turn Step", "25.6°"],
      ["Pitch Step", "29.4°"],
      ["Start pole", "1.8° short of the Nadir"],
      ["Closing turn", "0.4° long"],
      ["Drift per Shot", "under 0.3°"],
    ]);
    const exact = measuredRows(
      measuredOfEntry({ ...entry, startPoleOffsetDeg: -2.5, closingTurnDeg: 0.0, driftDeg: 0 }),
      "zenith",
    );
    expect(exact[3]).toEqual(["Start pole", "2.5° past the Zenith"]);
    expect(exact[4]).toEqual(["Closing turn", "closes"]);
    expect(exact[5]).toEqual(["Drift per Shot", "under 0.0°"]);
  });

  test("shows the field of view alone before the Rows are placed", () => {
    const early = measuredRows(
      {
        fieldOfViewDeg: 92.4,
        turnStepDeg: null,
        pitchStepDeg: null,
        startPoleOffsetDeg: null,
        closingTurnDeg: null,
        driftDeg: null,
      },
      "nadir",
    );
    expect(early[0]).toEqual(["Field of view", "92.4° wide"]);
    expect(early.slice(1).map(([, v]) => v)).toEqual(["…", "…", "…", "…", "…"]);
  });
});

test("names join with the last one apart", () => {
  expect(nameList([], "and")).toBe("");
  expect(nameList(["r07_c03"], "and")).toBe("r07_c03");
  expect(nameList(["r07_c03", "r07_c09", "r07_c12"], "and")).toBe("r07_c03, r07_c09 and r07_c12");
});

test("the expected size is about π times the Shot width, in 64s", () => {
  expect(expectedSize(1669)).toEqual({ w: 5184, h: 2592 });
  expect(expectedSize(10)).toEqual({ w: 64, h: 32 });
});
