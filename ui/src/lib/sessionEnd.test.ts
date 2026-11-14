import { describe, expect, test } from "vitest";
import { nextVariant, type SessionEndState, sessionActions } from "./sessionEnd";

describe("the Next line", () => {
  test("picks one of four variants from the switch and the Stitchers", () => {
    expect(nextVariant(true, ["ptgui"])).toBe("stitchAndOpen");
    expect(nextVariant(true, [])).toBe("stitchOrDrop");
    expect(nextVariant(false, ["ptgui", "autopanoGiga"])).toBe("openIn");
    expect(nextVariant(false, [])).toBe("drop");
  });
});

describe("the actions row", () => {
  const base: SessionEndState = {
    status: "completed",
    stitchOn: false,
    stitchers: [],
    hasPanorama: false,
    stitching: false,
  };
  const ids = (state: Partial<SessionEndState>) =>
    sessionActions({ ...base, ...state }).map((a) =>
      "stitcher" in a
        ? `${a.id}:${a.stitcher}${a.primary ? "*" : ""}`
        : `${a.id}${a.primary ? "*" : ""}`,
    );

  test("without the switch, the first Stitcher or Zip is primary, as before", () => {
    expect(ids({})).toEqual(["zip*", "reveal"]);
    expect(ids({ stitchers: ["ptgui"] })).toEqual(["openIn:ptgui*", "zip", "reveal"]);
    expect(ids({ stitchers: ["ptgui", "autopanoGiga"] })).toEqual([
      "openIn:ptgui*",
      "openIn:autopanoGiga",
      "zip",
      "reveal",
    ]);
  });

  test("with the switch, Stitch leads and nothing else is primary", () => {
    expect(ids({ stitchOn: true })).toEqual(["stitch*", "zip", "reveal"]);
    expect(ids({ stitchOn: true, stitchers: ["autopanoGiga"] })).toEqual([
      "stitch*",
      "openIn:autopanoGiga",
      "zip",
      "reveal",
    ]);
  });

  test("a Panorama turns the primary into Preview, with the panel's Stitch beside it", () => {
    expect(ids({ stitchOn: true, hasPanorama: true, stitchers: ["ptgui"] })).toEqual([
      "previewPanorama*",
      "stitchPanel",
      "openIn:ptgui*".replace("*", ""),
      "zip",
      "reveal",
    ]);
  });

  test("a Stitch in progress shows its button instead", () => {
    expect(ids({ stitchOn: true, stitching: true, hasPanorama: true })).toEqual([
      "stitching*",
      "zip",
      "reveal",
    ]);
  });

  test("an interrupted or failed Session gets neither the Stitch nor Open in", () => {
    expect(ids({ status: "interrupted", stitchOn: true, stitchers: ["ptgui"] })).toEqual([
      "zip*",
      "reveal",
    ]);
    expect(ids({ status: "failed", stitchOn: true, hasPanorama: true })).toEqual([
      "zip*",
      "reveal",
    ]);
  });

  test("a Panorama without the switch is ignored: the row is the old one", () => {
    expect(ids({ hasPanorama: true, stitchers: ["ptgui"] })).toEqual([
      "openIn:ptgui*",
      "zip",
      "reveal",
    ]);
  });
});
