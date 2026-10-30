import { afterEach, expect, test } from "vitest";
import { applyRunEvent, run } from "./run.svelte";

afterEach(() => run.reset());

test("a Test streams from countdown to finished and leaves a result", () => {
  run.expectShortcut(null);
  applyRunEvent({ event: "countdown", secondsLeft: 5 });
  expect(run.current?.phase).toBe("countdown");
  applyRunEvent({ event: "started", expected: 0 });
  applyRunEvent({ event: "moved", row: 1, col: 2 });
  applyRunEvent({ event: "moved", row: 1, col: 3 });
  applyRunEvent({ event: "screenUnchanged" });
  expect(run.current?.steps).toBe(2);
  expect(run.current?.screenUnchanged).toBe(true);
  applyRunEvent({ event: "finished", outcome: { status: "completed", shots: 0 }, folder: null });
  expect(run.current).toBeNull();
  expect(run.last?.steps).toBe(2);
  expect(run.last?.screenUnchanged).toBe(true);
  expect(run.last?.ended).toEqual({
    by: "finished",
    outcome: { status: "completed", shots: 0 },
    folder: null,
  });
});

test("samples and a cancelled countdown are recorded", () => {
  applyRunEvent({ event: "countdown", secondsLeft: 2 });
  applyRunEvent({ event: "started", expected: 0 });
  applyRunEvent({ event: "sample", slot: "first", image: "data:a" });
  applyRunEvent({ event: "sample", slot: "second", image: "data:b" });
  expect(run.current?.samples).toEqual({ first: "data:a", second: "data:b" });
  applyRunEvent({ event: "cancelled" });
  expect(run.last?.ended).toEqual({ by: "cancelled" });
  expect(run.last?.samples.second).toBe("data:b");
});

test("a run that cannot start reports why", () => {
  applyRunEvent({ event: "cannotStart", reason: "accessibility" });
  expect(run.last?.ended).toEqual({ by: "cannotStart", failure: { reason: "accessibility" } });
  expect(run.busy).toBe(false);
});
