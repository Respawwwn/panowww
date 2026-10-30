import { expect, test } from "vitest";
import { nothingMovedItems } from "./nothingMoved";
import { strings } from "./strings";

test("the checklist has five items in order, filtered by device and OS", () => {
  const mac = nothingMovedItems("mouse", "macos");
  expect(mac).toHaveLength(5);
  expect(mac[0]).toBe(strings.nothingMoved.fullscreen);
  expect(mac[1]).toBe(strings.nothingMoved.focus);
  expect(mac[2]).toBe(strings.nothingMoved.mouse);
  expect(mac[3]).toBe(strings.nothingMoved.mac.controllerMouse);
  expect(mac[4]).toBe(strings.nothingMoved.antiCheat);

  expect(nothingMovedItems("keyboard", "macos")[2]).toBe(strings.nothingMoved.keyboard);
  expect(nothingMovedItems("gamepad", "windows")[2]).toBe(strings.nothingMoved.gamepad);
  expect(nothingMovedItems("gamepad", "windows")[3]).toBe(strings.nothingMoved.win.rawInput);
});
