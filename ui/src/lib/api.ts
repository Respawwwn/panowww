// The one place the UI talks to Rust: one typed wrapper per command of the
// app crate. Types come from `generated/`, written by ts-rs (`just
// generate`). Commands exist in Rust once issue 11 lands; until then every
// call is mocked in tests with `mockIPC`.
import { Channel, invoke } from "@tauri-apps/api/core";
import type { DisplayInfo } from "./generated/DisplayInfo";
import type { Game } from "./generated/Game";
import type { GameEntry } from "./generated/GameEntry";
import type { Import } from "./generated/Import";
import type { Pong } from "./generated/Pong";
import type { RunOutcome } from "./generated/RunOutcome";
import type { SessionSummary } from "./generated/SessionSummary";
import type { Settings } from "./generated/Settings";
import type { Zone } from "./generated/Zone";
import type { ZoneStatus } from "./generated/ZoneStatus";

/** The one error shape the UI sees (spec §4.7). */
export type AppErrorKind =
  | "validation"
  | "notFound"
  | "store"
  | "input"
  | "capture"
  | "permission"
  | "busy"
  | "internal";

export interface AppError {
  kind: AppErrorKind;
  message: string;
  /** For `validation`: one entry per failing field. */
  fields?: { field: string; message: string }[];
}

/** Which run the Calibrate card or the Start card asks for. */
export type RunKindDto =
  | { kind: "session" }
  | { kind: "turnTest" }
  | { kind: "pitchTest" }
  | { kind: "stepOnce"; axis: "turn" | "pitch" }
  | { kind: "checkOverlap"; axis: "turn" | "pitch" };

/** What the Runner streams on the progress Channel (spec §4.4). */
export type RunEvent =
  | { event: "countdown"; secondsLeft: number }
  | { event: "started"; expected: number }
  | {
      event: "shot";
      index: number;
      row: number;
      col: number;
      captureMs: number;
      /** data: URL of the live JPEG thumbnail, or null when it could not be encoded. */
      thumbnail: string | null;
    }
  | { event: "moved"; row: number; col: number }
  | { event: "sample"; slot: "first" | "second"; image: string }
  | { event: "screenUnchanged" }
  | { event: "preview"; image: string; displayW: number; displayH: number; zone: Zone }
  | { event: "finished"; outcome: RunOutcome; folder: string | null };

/** Why a countdown could not start (spec §11.5 error state). */
export type StartFailure =
  | { reason: "accessibility" }
  | { reason: "screenRecording" }
  | { reason: "vigem" }
  | { reason: "displayGone"; display: string }
  | { reason: "captureUnavailable"; message: string };

export interface ZoneState {
  zone: Zone | null;
  status: ZoneStatus | null;
  /** 1-based index of the zone's display among the connected ones, for `Display {n}`. */
  displayIndex: number | null;
}

export interface Estimate {
  durationMs: number;
  /** `≈ 4 min 20 s` */
  duration: string;
  shots: number;
}

export interface Permissions {
  accessibility: boolean;
  screenRecording: boolean;
  /** Windows only; always true elsewhere. */
  vigem: boolean;
}

export type PermissionKind = "accessibility" | "screenRecording";

export interface UpdateInfo {
  version: string;
  url: string;
}

export const api = {
  /** Proves the IPC works and returns the app version. */
  ping: () => invoke<Pong>("ping"),

  games: {
    list: () => invoke<GameEntry[]>("list_games"),
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
    revealFile: (path: string) => invoke<void>("reveal_path", { path }),
    deleteFile: (path: string) => invoke<void>("delete_game_file", { path }),
    estimate: (game: Game) => invoke<Estimate>("estimate_session", { game }),
    adjustStep: (
      id: string,
      axis: "turn" | "pitch",
      observation: "overshot" | "undershot",
      magnitude: "fine" | "coarse",
    ) => invoke<Game>("adjust_step", { id, axis, observation, magnitude }),
  },

  zone: {
    get: (gameId: string) => invoke<ZoneState>("get_zone", { gameId }),
    /** Opens the picker windows; resolves once saved or cancelled. */
    pick: (gameId: string) => invoke<ZoneState>("pick_zone", { gameId }),
    show: (gameId: string, on: boolean) => invoke<void>("show_zone", { gameId, on }),
    displays: () => invoke<DisplayInfo[]>("list_displays"),
  },

  run: {
    /**
     * Pre-flight, stream, countdown, then the run. Events arrive on `onEvent`
     * until `finished`. Rejects with a `StartFailure` when the countdown
     * cannot start; ignored while a run is in progress.
     */
    start: (gameId: string, kind: RunKindDto, onEvent: (event: RunEvent) => void) => {
      const channel = new Channel<RunEvent>();
      channel.onmessage = onEvent;
      return invoke<void>("start_run", { gameId, kind, onEvent: channel });
    },
    /** Cancels a countdown or stops a run; `release_all` first. */
    stop: () => invoke<void>("stop_run"),
    setPreview: (on: boolean) => invoke<void>("set_preview", { on }),
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
    save: (settings: Settings) => invoke<void>("save_settings", { settings }),
    /** Folder picker; null when cancelled. */
    chooseSessionsFolder: () => invoke<string | null>("choose_sessions_folder"),
    revealLogs: () => invoke<void>("reveal_logs"),
    revealPath: (path: string) => invoke<void>("reveal_path", { path }),
    /** Re-registers the global shortcuts; rejects with the failing combo. */
    applyShortcuts: () => invoke<void>("apply_shortcuts"),
  },

  onboarding: {
    permissions: () => invoke<Permissions>("permissions"),
    request: (permission: PermissionKind) => invoke<void>("request_permission", { permission }),
    openSystemSettings: (permission: PermissionKind) =>
      invoke<void>("open_system_settings", { permission }),
    openUrl: (url: string) => invoke<void>("open_url", { url }),
  },

  update: {
    /** null when up to date, offline, or the switch is off. */
    check: () => invoke<UpdateInfo | null>("check_for_updates"),
  },
};

export type Api = typeof api;
