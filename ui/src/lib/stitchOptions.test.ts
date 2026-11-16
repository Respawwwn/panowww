import { expect, test } from "vitest";
import { defaultSettings } from "./settings.svelte";
import {
  clampQuality,
  DEFAULT_OPTIONS,
  optionsFor,
  optionsOfEntry,
  withOptions,
} from "./stitchOptions";

test("a Game without saved options gets the defaults", () => {
  expect(optionsFor(defaultSettings, "g1")).toEqual(DEFAULT_OPTIONS);
});

test("each Game remembers its own options", () => {
  const off = { ...DEFAULT_OPTIONS, antiGhost: false };
  const s1 = withOptions(defaultSettings, "g1", off);
  const s2 = withOptions(s1, "g2", { ...DEFAULT_OPTIONS, jpegQuality: 92 });
  expect(optionsFor(s2, "g1")).toEqual(off);
  expect(optionsFor(s2, "g2").jpegQuality).toBe(92);
  expect(optionsFor(s2, "g3")).toEqual(DEFAULT_OPTIONS);
  expect(s2.experimental.stitch, "the switch is left alone").toBe(false);
  expect(defaultSettings.experimental.stitchOptions, "nothing mutated").toBeUndefined();
});

test("the quality stays a whole percent between 50 and 100", () => {
  expect(clampQuality(95.4)).toBe(95);
  expect(clampQuality(20)).toBe(50);
  expect(clampQuality(140)).toBe(100);
});

test("a Panorama tells which options made it", () => {
  expect(
    optionsOfEntry({
      file: "panorama.jpg",
      preview: "panorama-preview.jpg",
      width: 10,
      height: 5,
      bytes: 1,
      stitchedAt: "",
      durationMs: 1,
      appVersion: "",
      fieldOfViewDeg: 90,
      turnStepDeg: 25,
      pitchStepDeg: 25,
      startPoleOffsetDeg: 0,
      closingTurnDeg: 0,
      driftDeg: 0,
      unaligned: [],
      droppedSteps: [],
      antiGhost: false,
      unalignedShots: "leaveOut",
      jpegQuality: 90,
    }),
  ).toEqual({ antiGhost: false, unalignedShots: "leaveOut", jpegQuality: 90 });
});
