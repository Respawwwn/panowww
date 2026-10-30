// The Games of this machine as the sidebar and the detail show them, and
// the actions on them (spec §11.1, §5). Reloaded when the folder changes
// on disk (`games-changed` from the app).
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type { Game } from "./generated/Game";
import type { GameEntry } from "./generated/GameEntry";
import type { Import } from "./generated/Import";
import { nav } from "./nav.svelte";

/** Default name of a new Game, made unique by the store. */
export const NEW_GAME_NAME = "New Game";

let entries = $state<GameEntry[]>([]);
let loaded = $state(false);
/** An import that met a known id, waiting for Replace / Keep both. */
let pendingImport = $state<Extract<Import, { kind: "knownId" }> | null>(null);

export const games = {
  get entries(): GameEntry[] {
    return entries;
  },
  get loaded(): boolean {
    return loaded;
  },
  get valid(): Game[] {
    return entries.flatMap((e) => (e.kind === "game" ? [e.game] : []));
  },
  get pendingImport() {
    return pendingImport;
  },
  /** The Game shown in the detail area, if it is a valid one. */
  get selected(): Game | null {
    const id = nav.gameId;
    return id ? (this.valid.find((g) => g.id === id) ?? null) : null;
  },
  find(id: string): Game | null {
    return this.valid.find((g) => g.id === id) ?? null;
  },

  async load(): Promise<void> {
    entries = await api.games.list();
    loaded = true;
    // A Game that vanished from disk leaves the detail area.
    const id = nav.gameId;
    if (id && nav.screen.kind === "game" && !this.find(id)) {
      nav.showEmpty();
    }
  },

  /** Listens for changes on disk until the returned function is called. */
  async watch(): Promise<() => void> {
    return listen("games-changed", () => {
      void this.load();
    });
  },

  /** Replaces one Game in place after an autosave, without a reload. */
  replace(game: Game): void {
    entries = entries.map((e) => (e.kind === "game" && e.game.id === game.id ? { ...e, game } : e));
  },

  async create(): Promise<Game> {
    const game = await api.games.create(NEW_GAME_NAME);
    await this.load();
    await this.select(game.id);
    return game;
  },

  async select(id: string | null): Promise<void> {
    if (id) {
      nav.showGame(id);
    } else {
      nav.showEmpty();
    }
    await api.games.select(id);
  },

  async duplicate(id: string): Promise<void> {
    const copy = await api.games.duplicate(id);
    await this.load();
    await this.select(copy.id);
  },

  async delete(id: string): Promise<void> {
    await api.games.delete(id);
    await this.load();
    await this.select(null);
  },

  async deleteFile(path: string): Promise<void> {
    await api.games.deleteFile(path);
    await this.load();
    nav.showEmpty();
  },

  share(id: string): Promise<boolean> {
    return api.games.share(id);
  },

  /** *Import…*: adds an unknown id, or asks about a known one. */
  async import(): Promise<void> {
    const result = await api.games.import();
    if (!result) {
      return;
    }
    if (result.kind === "added") {
      // ts-rs flattens the newtype variant: the Game's fields sit beside `kind`.
      await this.load();
      await this.select(result.id);
    } else {
      pendingImport = result;
    }
  },

  async resolveImport(choice: "replace" | "keepBoth" | "cancel"): Promise<void> {
    const pending = pendingImport;
    pendingImport = null;
    if (!pending || choice === "cancel") {
      return;
    }
    const game =
      choice === "replace"
        ? await api.games.importReplace(pending.incoming)
        : await api.games.importKeepBoth(pending.incoming);
    await this.load();
    await this.select(game.id);
  },
};
