import { expect, test, vi } from "vitest";
import { RowConfirm } from "./confirm.svelte";

test("one row asks at a time, Cancel puts the actions back", () => {
  const c = new RowConfirm();
  expect(c.confirming).toBeNull();
  c.ask("/a");
  expect(c.confirming).toBe("/a");
  c.ask("/b");
  expect(c.confirming).toBe("/b");
  c.cancel();
  expect(c.confirming).toBeNull();
});

test("Confirm runs the action once and closes; a failure keeps the row asking", async () => {
  const c = new RowConfirm();
  const action = vi.fn().mockResolvedValue(undefined);
  expect(await c.confirm(action)).toBe(false);
  expect(action).not.toHaveBeenCalled();

  c.ask("/a");
  expect(await c.confirm(action)).toBe(true);
  expect(action).toHaveBeenCalledTimes(1);
  expect(c.confirming).toBeNull();

  c.ask("/b");
  const failing = vi
    .fn()
    .mockRejectedValue({ kind: "store", message: "Could not delete /b: busy" });
  expect(await c.confirm(failing)).toBe(false);
  expect(c.confirming).toBe("/b");
  expect(c.error).toContain("busy");
});
