// The one place the UI talks to Rust: one typed wrapper per command of the
// app crate, over the types ts-rs writes into `generated/` (`just
// generate`). Every command is mockable with `mockIPC`.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppError } from "./generated/AppError";
import type { AxisDto } from "./generated/AxisDto";
import type { DisplayInfo } from "./generated/DisplayInfo";
import type { Estimate } from "./generated/Estimate";
import type { Game } from "./generated/Game";
import type { GameEntry } from "./generated/GameEntry";
import type { Import } from "./generated/Import";
import type { OverlayState } from "./generated/OverlayState";
import type { PermissionKind } from "./generated/PermissionKind";
import type { Permissions } from "./generated/Permissions";
import type { PickerSetup } from "./generated/PickerSetup";
import type { Pong } from "./generated/Pong";
import type { Rect } from "./generated/Rect";
import type { RunEvent } from "./generated/RunEvent";
import type { RunKindDto } from "./generated/RunKindDto";
import type { RunnerPhase } from "./generated/RunnerPhase";
import type { SessionSummary } from "./generated/SessionSummary";
import type { Settings } from "./generated/Settings";
import type { UpdateInfo } from "./generated/UpdateInfo";
import type { Zone } from "./generated/Zone";
import type { ZoneState } from "./generated/ZoneState";

export type {
  AppError,
  AxisDto,
  Estimate,
  PermissionKind,
  Permissions,
  RunEvent,
  RunKindDto,
  RunnerPhase,
  UpdateInfo,
  ZoneState,
};

/** The window event every `RunEvent` arrives on. */
export const RUN_EVENT = "run";

/** Whether a rejected command carried an `AppError`. */
export function isAppError(error: unknown): error is AppError {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error &&
    typeof (error as AppError).message === "string"
  );
}

export const api = {
  /** Proves the IPC works and returns the app version. */
  ping: () => invoke<Pong>("ping"),

  games: {
    list: () => invoke<GameEntry[]>("list_games"),
    /** The sidebar's order after a drag, every valid Game's id top down. */
    reorder: (ids: string[]) => invoke<void>("reorder_games", { ids }),
    create: (name: string) => invoke<Game>("create_game", { name }),
    /** Autosave on every valid change; renames the file when the name changed. */
    save: (game: Game) => invoke<string>("save_game", { game }),
    duplicate: (id: string) => invoke<Game>("duplicate_game", { id }),
    delete: (id: string) => invoke<void>("delete_game", { id }),
    /** Opens the save dialog; resolves to false when cancelled. */
    share: (id: string) => invoke<boolean>("share_game", { id }),
    /** Opens the file picker; null when cancelled. */
    import: () => invoke<Import | null>("import_game"),
    importReplace: (incoming: Game) => invoke<Game>("import_replace", { incoming }),
    importKeepBoth: (incoming: Game) => invoke<Game>("import_keep_both", { incoming }),
    /** For damaged files, which have no id. */
    deleteFile: (path: string) => invoke<void>("delete_game_file", { path }),
    revealFile: (path: string) => invoke<void>("reveal_path", { path }),
    estimate: (game: Game) => invoke<Estimate>("estimate_session", { game }),
    adjustStep: (
      id: string,
      axis: AxisDto,
      observation: "overshot" | "undershot",
      magnitude: "fine" | "coarse",
    ) => invoke<Game>("adjust_step", { id, axis, observation, magnitude }),
    /** The Game the detail area shows: what the Start shortcut runs. */
    select: (id: string | null) => invoke<void>("select_game", { id }),
  },

  zone: {
    get: (gameId: string) => invoke<ZoneState>("get_zone", { gameId }),
    set: (gameId: string, zone: Zone) => invoke<ZoneState>("set_zone", { gameId, zone }),
    /** Opens the picker windows; resolves once saved or cancelled. */
    pick: (gameId: string) => invoke<ZoneState>("pick_zone", { gameId }),
    show: (gameId: string, on: boolean) => invoke<void>("show_zone", { gameId, on }),
    displays: () => invoke<DisplayInfo[]>("list_displays"),
  },

  run: {
    /**
     * Pre-flight, stream, countdown, then the run, from a button (5 s).
     * Resolves to false when a run is in progress (the request is ignored).
     * Everything after arrives on `onEvent`, `cannotStart` included.
     */
    start: (gameId: string, kind: RunKindDto) => invoke<boolean>("start_run", { gameId, kind }),
    /** Cancels a countdown or stops a run; `release_all` first. */
    stop: () => invoke<void>("stop_run"),
    /** The Hide / Show preview button; persisted in Settings. */
    setPreview: (on: boolean) => invoke<void>("set_preview", { on }),
    phase: () => invoke<RunnerPhase>("runner_phase"),
    /** Every `RunEvent`, shortcut-started runs included. */
    onEvent: (handler: (event: RunEvent) => void): Promise<UnlistenFn> =>
      listen<RunEvent>(RUN_EVENT, (e) => handler(e.payload)),
  },

  sessions: {
    list: (gameId?: string) =>
      invoke<SessionSummary[]>("list_sessions", { gameId: gameId ?? null }),
    size: (folder: string) => invoke<number>("session_size", { folder }),
    zip: (folder: string) => invoke<string>("zip_session", { folder }),
    reveal: (folder: string) => invoke<void>("reveal_path", { path: folder }),
    delete: (folder: string) => invoke<void>("delete_session", { folder }),
    /** data: URL of the history thumbnail, generated on demand. */
    thumbnail: (folder: string, file: string) =>
      invoke<string>("session_thumbnail", { folder, file }),
  },

  settings: {
    get: () => invoke<Settings>("get_settings"),
    /** Writes the settings and re-registers the shortcuts when they changed. */
    save: (settings: Settings) => invoke<void>("save_settings", { settings }),
    /** Folder picker; null when cancelled. */
    chooseSessionsFolder: () => invoke<string | null>("choose_sessions_folder"),
    revealLogs: () => invoke<void>("reveal_logs"),
    revealPath: (path: string) => invoke<void>("reveal_path", { path }),
    /** Re-registers the global shortcuts; rejects naming the failing combo. */
    applyShortcuts: () => invoke<void>("apply_shortcuts"),
  },

  onboarding: {
    permissions: () => invoke<Permissions>("permissions"),
    request: (permission: PermissionKind) => invoke<boolean>("request_permission", { permission }),
    openSystemSettings: (permission: PermissionKind) =>
      invoke<void>("open_system_settings", { permission }),
    openUrl: (url: string) => invoke<void>("open_url", { url }),
  },

  update: {
    /** null when up to date, offline, or the switch is off. */
    check: () => invoke<UpdateInfo | null>("check_for_updates"),
  },

  /** The picker windows' side: one window per display, `label` is its own. */
  picker: {
    setup: (label: string) => invoke<PickerSetup>("picker_setup", { label }),
    save: (label: string, rect: Rect) => invoke<void>("picker_save", { label, rect }),
    cancel: () => invoke<void>("picker_cancel"),
  },

  /** The overlay window's side: what it draws, pushed by Rust. */
  overlay: {
    state: () => invoke<OverlayState | null>("overlay_state"),
    onState: (handler: (state: OverlayState) => void): Promise<UnlistenFn> =>
      listen<OverlayState>("overlay", (event) => handler(event.payload)),
  },
};

export type Api = typeof api;
