import { afterEach, expect, test } from "vitest";
import { screenForRun } from "./flow";
import type { Screen } from "./nav.svelte";
import { applyRunEvent, run } from "./run.svelte";

afterEach(() => run.reset());

/** Applies the events and returns the screen after each, from `start`. */
function drive(start: Screen, events: Parameters<typeof applyRunEvent>[0][]): Screen[] {
  let screen = start;
  return events.map((event) => {
    applyRunEvent(event);
    screen = screenForRun(run.current, run.last, screen) ?? screen;
    return screen;
  });
}

const setup: Screen = { kind: "game", id: "g1", tab: "setup" };

test("a Session goes countdown → running → ended with its folder", () => {
  run.expectShortcut("g1");
  const screens = drive(setup, [
    { event: "countdown", secondsLeft: 3 },
    { event: "countdown", secondsLeft: 2 },
    { event: "started", expected: 6 },
    { event: "shot", index: 1, row: 1, col: 1, captureMs: 40, thumbnail: null },
    { event: "moved", row: 1, col: 2 },
    { event: "finished", outcome: { status: "completed", shots: 6 }, folder: "/S/Game/2026" },
  ]);
  expect(screens.map((s) => s.kind)).toEqual([
    "countdown",
    "countdown",
    "running",
    "running",
    "running",
    "ended",
  ]);
  expect(screens.at(-1)).toEqual({ kind: "ended", gameId: "g1", folder: "/S/Game/2026" });
});

test("a Test returns to the Game, a cancelled countdown too", () => {
  run.expect("g1", { kind: "turnTest" });
  const test1 = drive(setup, [
    { event: "countdown", secondsLeft: 5 },
    { event: "started", expected: 0 },
    { event: "finished", outcome: { status: "completed", shots: 0 }, folder: null },
  ]);
  expect(test1.map((s) => s.kind)).toEqual(["countdown", "running", "game"]);

  run.reset();
  run.expectShortcut("g1");
  const cancelled = drive(setup, [{ event: "countdown", secondsLeft: 5 }, { event: "cancelled" }]);
  expect(cancelled.map((s) => s.kind)).toEqual(["countdown", "game"]);
});

test("a run that cannot start keeps the countdown screen in its error state", () => {
  run.expectShortcut("g1");
  const screens = drive(setup, [{ event: "cannotStart", reason: "screenRecording" }]);
  expect(screens[0]).toEqual({ kind: "countdown", gameId: "g1" });
  // Back is the user's: the flow leaves the screen alone afterwards.
  expect(screenForRun(run.current, run.last, screens[0] as Screen)).toBeNull();
});

test("the flow never moves a screen that is not a run screen", () => {
  run.expectShortcut("g1");
  applyRunEvent({ event: "countdown", secondsLeft: 1 });
  applyRunEvent({ event: "cancelled" });
  expect(screenForRun(run.current, run.last, { kind: "empty" })).toBeNull();
});
