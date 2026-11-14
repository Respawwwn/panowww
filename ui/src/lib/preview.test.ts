import { expect, test } from "vitest";
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import { initialPreviewState, type PreviewSubject, reducePreview } from "./preview.svelte";

const entry: PanoramaEntry = {
  file: "panorama.jpg",
  preview: "panorama-preview.jpg",
  width: 5184,
  height: 2592,
  bytes: 5_100_000,
  stitchedAt: "2026-11-14T16:00:00+01:00",
  durationMs: 9800,
  appVersion: "1.1.0",
  fieldOfViewDeg: 92.4,
  turnStepDeg: 25.6,
  pitchStepDeg: 29.4,
  startPoleOffsetDeg: 1.8,
  closingTurnDeg: 0.4,
  driftDeg: 0.26,
  unaligned: [],
  droppedSteps: [],
  antiGhost: true,
};

const subject: PreviewSubject = {
  folder: "/S/a",
  entry,
  gameName: "World of Warcraft",
  startedAt: 1_000,
  startPole: "nadir",
};

test("the window opens on a Panorama, fills and unfills, and closes", () => {
  const opened = reducePreview(initialPreviewState, { type: "open", subject });
  expect(opened).toEqual({ subject, fill: false, fallback: false });
  const filled = reducePreview(opened, { type: "toggleFill" });
  expect(filled.fill).toBe(true);
  expect(reducePreview(filled, { type: "toggleFill" }).fill).toBe(false);
  const closed = reducePreview(filled, { type: "close" });
  expect(closed.subject).toBeNull();
  expect(closed.fill, "a closed window is not left filled").toBe(false);
});

test("a viewer that cannot start falls back to the flat image until the next Panorama", () => {
  const opened = reducePreview(initialPreviewState, { type: "open", subject });
  const flat = reducePreview(opened, { type: "viewerFailed" });
  expect(flat.fallback).toBe(true);
  expect(reducePreview(flat, { type: "toggleFill" }).fallback, "fill keeps the fallback").toBe(
    true,
  );
  const again = reducePreview(flat, { type: "open", subject: { ...subject, folder: "/S/b" } });
  expect(again.fallback, "a new Panorama tries the viewer again").toBe(false);
});

test("nothing happens to a closed window", () => {
  expect(reducePreview(initialPreviewState, { type: "toggleFill" })).toEqual(initialPreviewState);
  expect(reducePreview(initialPreviewState, { type: "viewerFailed" })).toEqual(initialPreviewState);
});
