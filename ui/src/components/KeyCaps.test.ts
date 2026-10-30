import { flushSync, mount, unmount } from "svelte";
import { expect, test } from "vitest";
import KeyCaps from "./KeyCaps.svelte";

test("renders a shortcut as caps joined by +", () => {
  const target = document.createElement("div");
  const component = mount(KeyCaps, {
    target,
    props: { shortcut: { modifiers: ["ctrl", "shift"], code: "Digit9" }, platform: "macos" },
  });
  flushSync();

  const caps = Array.from(target.querySelectorAll("kbd")).map((k) => k.textContent);
  expect(caps).toEqual(["Ctrl", "Shift", "9"]);
  expect(target.textContent?.replace(/\s+/g, "")).toBe("Ctrl+Shift+9");

  unmount(component);
});
