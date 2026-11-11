// Pure helpers around the Game type: the per-device default blocks the
// editor installs when the device changes (spec §5), mirroring
// `Input::defaults_for` in core.
import type { Device } from "./generated/Device";
import type { Game } from "./generated/Game";
import type { Input } from "./generated/Input";

/** The input block of one device. */
export type InputOf<D extends Device> = Extract<Input, { device: D }>;

/** The input block of a device as a new Game gets it. */
export function defaultInput<D extends Device>(device: D): InputOf<D> {
  const blocks: { [K in Device]: InputOf<K> } = {
    keyboard: {
      device: "keyboard",
      turnRightKey: "ArrowRight",
      lookUpKey: "ArrowUp",
      lookDownKey: "ArrowDown",
      turnHoldMs: 100,
      pitchHoldMs: 100,
    },
    mouse: {
      device: "mouse",
      mode: "move",
      turnDelta: 500,
      pitchDelta: 500,
      invertVertical: false,
    },
    gamepad: {
      device: "gamepad",
      stickDeflection: 0.8,
      turnHoldMs: 100,
      pitchHoldMs: 100,
      invertVertical: false,
    },
  };
  return structuredClone(blocks[device]);
}

/** One-line summary of the input block for the card header. */
export function inputSummary(input: Input): string {
  switch (input.device) {
    case "keyboard":
      return `Keyboard · ${input.turnHoldMs} / ${input.pitchHoldMs} ms`;
    case "mouse":
      return `Mouse · ${input.mode === "drag" ? `drag ${input.dragButton ?? ""}`.trim() : "move"} · ${input.turnDelta} / ${input.pitchDelta}`;
    case "gamepad":
      return `Gamepad · ${input.stickDeflection} · ${input.turnHoldMs} / ${input.pitchHoldMs} ms`;
  }
}

/** One-line summary of the grid and delays for the card header. */
export function gridSummary(game: Game): string {
  return `${game.shotsPerRow} × ${game.rows} · ${game.waitAfterMoveMs} / ${game.waitAfterShotMs} ms`;
}

/** Why the Start card and the Calibrate tools are disabled, if they are. */
export type DisabledReason = "noZone" | "gamepadMac" | "damaged";

export function disabledReason(
  game: Game,
  hasZone: boolean,
  platform: "macos" | "windows",
): DisabledReason | null {
  if (game.input.device === "gamepad" && platform === "macos") {
    return "gamepadMac";
  }
  if (!hasZone) {
    return "noZone";
  }
  return null;
}

/** The schema bounds of the grid counts, as `core::game::validate` has them. */
export const SHOTS_PER_ROW = { min: 3, max: 72 } as const;
export const ROWS = { min: 2, max: 36 } as const;

/**
 * The count row of a Test (spec §7.5): Shots per Row for the turn axis,
 * Rows for the pitch axis, moved by `by` and clamped to the schema bounds.
 * Returns the patch for the editor.
 */
export function adjustCount(
  game: Game,
  axis: "turn" | "pitch",
  by: number,
): Pick<Game, "shotsPerRow"> | Pick<Game, "rows"> {
  if (axis === "turn") {
    const next = Math.min(SHOTS_PER_ROW.max, Math.max(SHOTS_PER_ROW.min, game.shotsPerRow + by));
    return { shotsPerRow: next };
  }
  const next = Math.min(ROWS.max, Math.max(ROWS.min, game.rows + by));
  return { rows: next };
}
