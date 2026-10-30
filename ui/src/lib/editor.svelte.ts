// The Game Setup form state (spec §11.2): edits apply to a working copy,
// every change is sent to Rust, which validates and saves it (autosave,
// no Save button); the validation messages come back per field. The
// estimate follows every saved change.
import { api, isAppError } from "./api";
import { defaultInput } from "./game";
import type { Device } from "./generated/Device";
import type { Estimate } from "./generated/Estimate";
import type { Game } from "./generated/Game";
import type { Input } from "./generated/Input";

/** How long after the last keystroke the Game is sent to Rust. */
export const AUTOSAVE_DELAY_MS = 250;

export type FieldErrors = Record<string, string>;

export class GameEditor {
  game = $state<Game>() as Game;
  errors = $state<FieldErrors>({});
  estimate = $state<Estimate | null>(null);
  /** The last save failed for a reason other than validation. */
  saveError = $state<string | null>(null);
  saving = $state(false);

  private timer: ReturnType<typeof setTimeout> | null = null;
  private onSaved: (game: Game) => void;
  private delay: number;

  constructor(game: Game, onSaved: (game: Game) => void = () => {}, delay = AUTOSAVE_DELAY_MS) {
    // The Game may be a reactive proxy from the games store; snapshot it
    // first so the copy is a plain object.
    this.game = structuredClone($state.snapshot(game)) as Game;
    this.onSaved = onSaved;
    this.delay = delay;
    void this.refreshEstimate();
  }

  /** Changes Game-level fields and schedules the save. */
  update(patch: Partial<Omit<Game, "input" | "id" | "format" | "version">>): void {
    this.game = { ...this.game, ...patch };
    this.schedule();
  }

  /** Changes fields of the input block and schedules the save. */
  updateInput(patch: Partial<Input>): void {
    this.game = { ...this.game, input: { ...this.game.input, ...patch } as Input };
    this.schedule();
  }

  /** Switching device replaces the block with that device's defaults. */
  setDevice(device: Device): void {
    if (this.game.input.device === device) {
      return;
    }
    this.game = { ...this.game, input: defaultInput(device) };
    this.schedule();
  }

  /** Drag needs a button; move must not carry one. */
  setMouseMode(mode: "move" | "drag"): void {
    if (this.game.input.device !== "mouse") {
      return;
    }
    const { dragButton, ...rest } = this.game.input;
    this.game = {
      ...this.game,
      input:
        mode === "drag" ? { ...rest, mode, dragButton: dragButton ?? "right" } : { ...rest, mode },
    } as Game;
    this.schedule();
  }

  /** Takes a Game saved elsewhere (an adjust) without sending it back. */
  replace(game: Game): void {
    this.game = structuredClone($state.snapshot(game)) as Game;
    this.errors = {};
    void this.refreshEstimate();
  }

  /** The message of one field, by its JSON path (`rows`, `input.turnDelta`). */
  error(field: string): string | null {
    return this.errors[field] ?? null;
  }

  /** Saves now instead of after the delay (on blur, before leaving). */
  async flush(): Promise<void> {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    await this.save();
  }

  dispose(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  private schedule(): void {
    if (this.timer) {
      clearTimeout(this.timer);
    }
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.save();
    }, this.delay);
  }

  private async save(): Promise<void> {
    const snapshot = structuredClone($state.snapshot(this.game)) as Game;
    this.saving = true;
    try {
      await api.games.save(snapshot);
      this.errors = {};
      this.saveError = null;
      this.onSaved(snapshot);
      await this.refreshEstimate();
    } catch (error) {
      if (isAppError(error) && error.kind === "validation") {
        this.errors = Object.fromEntries((error.fields ?? []).map((f) => [f.field, f.message]));
        this.saveError = null;
      } else {
        this.saveError = isAppError(error) ? error.message : String(error);
      }
    } finally {
      this.saving = false;
    }
  }

  private async refreshEstimate(): Promise<void> {
    try {
      this.estimate = await api.games.estimate(structuredClone($state.snapshot(this.game)) as Game);
    } catch {
      this.estimate = null;
    }
  }
}
