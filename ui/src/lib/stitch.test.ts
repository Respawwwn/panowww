import { describe, expect, test } from "vitest";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { StitchEvent } from "./generated/StitchEvent";
import {
  initialStitchState,
  percent,
  reduceStitch,
  remainingMs,
  stageMs,
  stageState,
  startStitch,
} from "./stitch.svelte";

const entry: PanoramaEntry = {
  file: "panorama.jpg",
  preview: "panorama-preview.jpg",
  width: 5184,
  height: 2592,
  bytes: 5_100_000,
  stitchedAt: "2026-11-14T16:00:00+01:00",
  durationMs: 5200,
  appVersion: "1.1.0",
  fieldOfViewDeg: 90,
  turnStepDeg: 25.7,
  pitchStepDeg: 30,
  startPoleOffsetDeg: 0,
  closingTurnDeg: 0,
  driftDeg: 0.2,
  unaligned: [],
  droppedSteps: [],
  antiGhost: true,
};

const progress = (
  overall: number,
  stage: "read" | "render" | "save" = "render",
): StitchEvent & { event: "progress" } => ({
  event: "progress",
  stage: { stage },
  stageFraction: 0.5,
  overall,
});

const play = (events: StitchEvent[], folder: string | null = "/S/a") =>
  events.reduce(
    (state, event, i) => reduceStitch(state, event, 1000 + i),
    folder === null ? initialStitchState : startStitch(initialStitchState, folder, 999),
  );

describe("the stitch reducer", () => {
  test("a started Stitch carries its folder through progress to the Panorama", () => {
    const state = play([progress(0.1), progress(0.6), { event: "finished", entry }]);
    expect(state.running).toBeNull();
    expect(state.entries["/S/a"]).toEqual(entry);
    expect(state.last).toEqual({
      folder: "/S/a",
      closing: { event: "finished", entry },
      startedAt: 999,
      endedAt: 1002,
      stages: { render: { startedAt: 1000, endedAt: 1002 } },
      progress: { stage: { stage: "render" }, stageFraction: 0.5, overall: 0.6 },
      measured: null,
    });
  });

  test("a report that measured something is kept until the next one that does", () => {
    const measured = {
      fieldOfViewDeg: 92.4,
      turnStepDeg: null,
      pitchStepDeg: null,
      startPoleOffsetDeg: null,
      closingTurnDeg: null,
      driftDeg: null,
    };
    const state = play([{ ...progress(0.05, "read"), measured }, progress(0.1, "read")]);
    expect(state.running?.measured).toEqual(measured);
    const ended = reduceStitch(state, { event: "failed", error: { kind: "notCompleted" } }, 2000);
    expect(ended.last?.measured).toEqual(measured);
  });

  test("stages are timed from their first report to the next stage's", () => {
    const state = play([progress(0.02, "read"), progress(0.05, "read"), progress(0.7, "render")]);
    expect(state.running?.stages).toEqual({
      read: { startedAt: 1000, endedAt: 1002 },
      render: { startedAt: 1002, endedAt: null },
    });
    expect(stageMs(state.running?.stages.read, 5000)).toBe(2);
    expect(stageMs(state.running?.stages.render, 5000)).toBe(3998);
    expect(stageMs(undefined, 5000)).toBeNull();
    const ended = reduceStitch(state, { event: "cancelled", reason: "button" }, 1010);
    expect(ended.last?.stages.render).toEqual({ startedAt: 1002, endedAt: 1010 });
  });

  test("progress keeps the folder and the start time", () => {
    const state = play([progress(0.1), progress(0.2)]);
    expect(state.running).toEqual({
      folder: "/S/a",
      startedAt: 999,
      progress: { stage: { stage: "render" }, stageFraction: 0.5, overall: 0.2 },
      stages: { render: { startedAt: 1000, endedAt: null } },
      measured: null,
    });
    expect(percent(state.running?.progress ?? null)).toBe(20);
    expect(percent(null)).toBe(0);
  });

  test("a cancel or a failure ends the Stitch and writes no entry", () => {
    const cancelled = play([progress(0.3), { event: "cancelled", reason: "sessionStarted" }]);
    expect(cancelled.running).toBeNull();
    expect(cancelled.entries).toEqual({});
    expect(cancelled.last?.closing).toEqual({ event: "cancelled", reason: "sessionStarted" });

    const failed = play([
      { event: "failed", error: { kind: "tooFewLinked", linked: 3, total: 98, unlinked: [] } },
    ]);
    expect(failed.last?.closing).toEqual({
      event: "failed",
      error: { kind: "tooFewLinked", linked: 3, total: 98, unlinked: [] },
    });
  });

  test("events before the folder is known still track the Stitch", () => {
    const state = play([progress(0.4)], null);
    expect(state.running).toEqual({
      folder: null,
      startedAt: 1000,
      progress: { stage: { stage: "render" }, stageFraction: 0.5, overall: 0.4 },
      stages: { render: { startedAt: 1000, endedAt: null } },
      measured: null,
    });
    const done = reduceStitch(state, { event: "finished", entry }, 2000);
    expect(done.entries, "no folder to file it under").toEqual({});
    expect(done.last?.folder).toBeNull();
  });

  test("a new Stitch keeps the Panoramas of the earlier ones", () => {
    const first = play([{ event: "finished", entry }]);
    const second = reduceStitch(
      startStitch(first, "/S/b", 3000),
      { event: "finished", entry: { ...entry, width: 4096, height: 2048 } },
      3001,
    );
    expect(Object.keys(second.entries)).toEqual(["/S/a", "/S/b"]);
    expect(second.entries["/S/b"]?.width).toBe(4096);
  });
});

describe("the remaining time", () => {
  test("says nothing at the very start, then follows the Stitch's own pace", () => {
    expect(remainingMs(500, 0.01)).toBeNull();
    expect(remainingMs(0, 0.5)).toBeNull();
    expect(remainingMs(2000, 0.25)).toBe(6000);
    expect(remainingMs(9000, 0.9)).toBe(1000);
    expect(remainingMs(12000, 1)).toBe(0);
  });
});

describe("the stage list", () => {
  test("marks the stages before the current one done, after it to do", () => {
    expect(stageState("read", "align", false)).toBe("done");
    expect(stageState("align", "align", false)).toBe("now");
    expect(stageState("render", "align", false)).toBe("todo");
    expect(stageState("align", "align", true)).toBe("bad");
    expect(stageState("read", null, false)).toBe("todo");
  });
});
