import { expect, test } from "vitest";
import { nav } from "./nav.svelte";

test("navigation is state", () => {
  expect(nav.screen).toEqual({ kind: "empty" });
  expect(nav.gameId).toBeNull();

  nav.showGame("g1");
  expect(nav.screen).toEqual({ kind: "game", id: "g1", tab: "setup" });
  expect(nav.gameId).toBe("g1");
  expect(nav.busy).toBe(false);

  nav.showCountdown("g1");
  expect(nav.busy).toBe(true);
  nav.showRunning("g1");
  expect(nav.busy).toBe(true);
  nav.showEnded("g1", "/Sessions/x");
  expect(nav.busy).toBe(false);
  expect(nav.gameId).toBe("g1");

  nav.showDamaged("/games/broken.panowww.json");
  expect(nav.gameId).toBeNull();
});
