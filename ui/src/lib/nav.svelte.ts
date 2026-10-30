// Navigation is state, not routes (spec §3.2, §11). One screen at a time in
// the detail area; the sidebar greys out while a run is going on.

export type DetailTab = "setup" | "sessions";

export type Screen =
  | { kind: "empty" }
  | { kind: "game"; id: string; tab: DetailTab }
  | { kind: "damaged"; path: string }
  | { kind: "countdown"; gameId: string }
  | { kind: "running"; gameId: string }
  | { kind: "ended"; gameId: string; folder: string | null }
  | { kind: "gallery" };

let screen = $state<Screen>(initialScreen());

function initialScreen(): Screen {
  if (typeof location !== "undefined") {
    const wantsGallery =
      new URLSearchParams(location.search).has("gallery") || location.hash === "#gallery";
    if (wantsGallery) {
      return { kind: "gallery" };
    }
  }
  return { kind: "empty" };
}

export const nav = {
  get screen(): Screen {
    return screen;
  },
  /** The selected Game, whatever the screen shows about it. */
  get gameId(): string | null {
    return "id" in screen ? screen.id : "gameId" in screen ? screen.gameId : null;
  },
  /** During a countdown or a run the sidebar is greyed and inert. */
  get busy(): boolean {
    return screen.kind === "countdown" || screen.kind === "running";
  },
  showEmpty() {
    screen = { kind: "empty" };
  },
  showGame(id: string, tab: DetailTab = "setup") {
    screen = { kind: "game", id, tab };
  },
  showDamaged(path: string) {
    screen = { kind: "damaged", path };
  },
  showCountdown(gameId: string) {
    screen = { kind: "countdown", gameId };
  },
  showRunning(gameId: string) {
    screen = { kind: "running", gameId };
  },
  showEnded(gameId: string, folder: string | null) {
    screen = { kind: "ended", gameId, folder };
  },
  showGallery() {
    screen = { kind: "gallery" };
  },
};
