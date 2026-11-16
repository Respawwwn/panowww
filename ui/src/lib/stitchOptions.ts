// The Stitch's options (spec §16.3, Options card): what each Game
// remembers in the settings, what a Panorama was made with, and the
// quality presets of the slider. Pure, so tested as data.
import type { PanoramaEntry } from "./generated/PanoramaEntry";
import type { Settings } from "./generated/Settings";
import type { StitchOptions } from "./generated/StitchOptions";

export const DEFAULT_OPTIONS: StitchOptions = {
  antiGhost: true,
  unalignedShots: "placeFromGrid",
  jpegQuality: 100,
};

export const MIN_QUALITY = 50;
export const MAX_QUALITY = 100;
/** The buttons beside the slider: they only move it. */
export const QUALITY_PRESETS = [100, 95, 90] as const;

/** A whole percent between 50 and 100. */
export function clampQuality(value: number): number {
  return Math.min(MAX_QUALITY, Math.max(MIN_QUALITY, Math.round(value)));
}

/** The options the Game remembers, or the defaults. */
export function optionsFor(settings: Settings, gameId: string): StitchOptions {
  const saved = settings.experimental.stitchOptions?.[gameId];
  return saved ? { ...DEFAULT_OPTIONS, ...saved } : { ...DEFAULT_OPTIONS };
}

/** `settings` with the Game's options replaced. */
export function withOptions(settings: Settings, gameId: string, options: StitchOptions): Settings {
  return {
    ...settings,
    experimental: {
      ...settings.experimental,
      stitchOptions: { ...settings.experimental.stitchOptions, [gameId]: options },
    },
  };
}

/** The options a Panorama on disk was made with; a block written before
 * the options lacks the last two: the defaults then. */
export function optionsOfEntry(entry: PanoramaEntry): StitchOptions {
  const partial = entry as Partial<PanoramaEntry>;
  return {
    antiGhost: partial.antiGhost ?? DEFAULT_OPTIONS.antiGhost,
    unalignedShots: partial.unalignedShots ?? DEFAULT_OPTIONS.unalignedShots,
    jpegQuality: partial.jpegQuality ?? DEFAULT_OPTIONS.jpegQuality,
  };
}
