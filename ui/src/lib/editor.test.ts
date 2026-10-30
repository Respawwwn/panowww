import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, test, vi } from "vitest";
import { GameEditor } from "./editor.svelte";
import { defaultInput } from "./game";
import type { Game } from "./generated/Game";

const game: Game = {
  format: "panowww-game",
  version: 1,
  id: "3f6c1e2a-0000-4000-8000-000000000001",
  name: "Elden Ring",
  startPole: "nadir",
  shotsPerRow: 13,
  rows: 6,
  waitAfterMoveMs: 300,
  waitAfterShotMs: 300,
  input: defaultInput("keyboard"),
};

/** A Rust side that validates rows and estimates from the grid. */
function mockRust() {
  const saved: Game[] = [];
  mockIPC((cmd, args) => {
    const a = args as { game: Game };
    if (cmd === "save_game") {
      if (a.game.rows < 2 || a.game.rows > 36) {
        return Promise.reject({
          kind: "validation",
          message: "Some fields are invalid.",
          fields: [{ field: "rows", message: "Rows must be between 2 and 36." }],
        });
      }
      saved.push(a.game);
      return `/games/${a.game.name}.panowww.json`;
    }
    if (cmd === "estimate_session") {
      const shots = a.game.shotsPerRow * a.game.rows;
      return { durationMs: shots * 1000, duration: `≈ ${shots} s`, shots };
    }
    throw new Error(`unexpected ${cmd}`);
  });
  return saved;
}

const flushPromises = () => new Promise((r) => setTimeout(r, 0));

afterEach(() => {
  clearMocks();
  vi.useRealTimers();
});

test("switching device replaces the block with that device's defaults", async () => {
  const saved = mockRust();
  const editor = new GameEditor(game, () => {}, 0);
  editor.setDevice("mouse");
  expect(editor.game.input).toEqual(defaultInput("mouse"));
  editor.setMouseMode("drag");
  expect(editor.game.input).toMatchObject({ mode: "drag", dragButton: "right" });
  editor.setMouseMode("move");
  expect(editor.game.input).not.toHaveProperty("dragButton");
  await flushPromises();
  await flushPromises();
  expect(saved.at(-1)?.input.device).toBe("mouse");
  editor.dispose();
});

test("an invalid change shows its message under the field and keeps the valid state saved", async () => {
  const saved = mockRust();
  const editor = new GameEditor(game, () => {}, 0);
  editor.update({ rows: 1 });
  await flushPromises();
  await flushPromises();
  expect(editor.error("rows")).toBe("Rows must be between 2 and 36.");
  expect(editor.error("name")).toBeNull();
  expect(saved).toHaveLength(0);

  editor.update({ rows: 8 });
  await flushPromises();
  await flushPromises();
  expect(editor.errors).toEqual({});
  expect(saved.at(-1)?.rows).toBe(8);
  editor.dispose();
});

test("the estimate follows every saved change", async () => {
  mockRust();
  const onSaved = vi.fn();
  const editor = new GameEditor(game, onSaved, 0);
  await flushPromises();
  expect(editor.estimate).toEqual({ durationMs: 78_000, duration: "≈ 78 s", shots: 78 });

  editor.update({ shotsPerRow: 10, rows: 4 });
  await flushPromises();
  await flushPromises();
  await flushPromises();
  expect(editor.estimate?.shots).toBe(40);
  expect(onSaved).toHaveBeenCalledWith(expect.objectContaining({ shotsPerRow: 10, rows: 4 }));
  editor.dispose();
});

test("edits within the delay are sent once", async () => {
  vi.useFakeTimers();
  const saved = mockRust();
  const editor = new GameEditor(game, () => {}, 250);
  editor.update({ name: "A" });
  editor.update({ name: "AB" });
  editor.update({ name: "ABC" });
  await vi.advanceTimersByTimeAsync(300);
  expect(saved.map((g) => g.name)).toEqual(["ABC"]);
  editor.dispose();
});
