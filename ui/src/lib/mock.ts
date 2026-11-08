// A browser stand-in for the Rust side, for looking at the screens outside
// Tauri: `pnpm --dir ui dev` then `http://localhost:1420/?mock`. Never
// bundled into the app: `main.ts` imports it only in dev with `?mock`.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { defaultInput } from "./game";
import type { Game } from "./generated/Game";
import type { GameEntry } from "./generated/GameEntry";
import type { RunEvent } from "./generated/RunEvent";
import type { RunKindDto } from "./generated/RunKindDto";
import type { SessionSummary } from "./generated/SessionSummary";
import type { Settings } from "./generated/Settings";
import { applyRunEvent } from "./run.svelte";
import { defaultSettings } from "./settings.svelte";

function game(id: string, name: string, input: Game["input"], notes?: string): Game {
  return {
    format: "panowww-game",
    version: 1,
    id,
    name,
    ...(notes ? { notes } : {}),
    startPole: "nadir",
    shotsPerRow: 13,
    rows: 6,
    waitAfterMoveMs: 300,
    waitAfterShotMs: 300,
    input,
  };
}

export function installMock(): void {
  const entries: GameEntry[] = [
    {
      kind: "game",
      game: game(
        "00000000-0000-4000-8000-000000000001",
        "Elden Ring – photo mode",
        {
          ...defaultInput("mouse"),
          mode: "drag",
          dragButton: "right",
          turnDelta: 840,
          pitchDelta: 610,
        },
        "FOV 90, HUD off",
      ),
      path: "/Users/me/Library/Application Support/com.respawwwn.panowww/games/Elden Ring – photo mode.panowww.json",
    },
    {
      kind: "game",
      game: game("00000000-0000-4000-8000-000000000002", "Outer Wilds", defaultInput("keyboard")),
      path: "/games/Outer Wilds.panowww.json",
    },
    {
      kind: "game",
      game: game(
        "00000000-0000-4000-8000-000000000003",
        "Forza Horizon 5",
        defaultInput("gamepad"),
      ),
      path: "/games/Forza Horizon 5.panowww.json",
    },
    {
      kind: "damaged",
      path: "/games/broken.panowww.json",
      message: "This Game has 2 invalid field(s).",
      errors: [
        { field: "rows", message: "Rows must be between 2 and 36." },
        { field: "input.turnRightKey", message: "Pick a key on the keyboard." },
      ],
    },
  ];
  const sessions = sampleSessions(entries);
  const zones: Record<string, boolean> = { "00000000-0000-4000-8000-000000000001": true };
  const zoneState = (has: boolean) =>
    has
      ? {
          zone: {
            displayId: "main",
            x: 320,
            y: 180,
            w: 1920,
            h: 1080,
            displayW: 2560,
            displayH: 1440,
            scale: 2,
          },
          status: "ready",
          displayIndex: 1,
        }
      : { zone: null, status: null, displayIndex: null };
  const settings: Settings = structuredClone(defaultSettings);

  mockWindows("main");
  mockIPC((cmd, rawArgs) => {
    const args = (rawArgs ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "ping":
        return { app: "Panowww", version: "0.0.1-mock" };
      case "list_games":
        return entries;
      case "reorder_games": {
        const ids = args.ids as string[];
        entries.sort((a, b) => {
          const pa = a.kind === "game" ? ids.indexOf(a.game.id) : ids.length;
          const pb = b.kind === "game" ? ids.indexOf(b.game.id) : ids.length;
          return pa - pb;
        });
        return null;
      }
      case "save_game": {
        const g = args.game as Game;
        if (g.rows < 2 || g.rows > 36) {
          return Promise.reject({
            kind: "validation",
            message: "Some fields are invalid.",
            fields: [{ field: "rows", message: "Rows must be between 2 and 36." }],
          });
        }
        const i = entries.findIndex((e) => e.kind === "game" && e.game.id === g.id);
        if (i >= 0) {
          entries[i] = { kind: "game", game: g, path: `/games/${g.name}.panowww.json` };
        }
        return `/games/${g.name}.panowww.json`;
      }
      case "create_game": {
        const g = game(crypto.randomUUID(), String(args.name), defaultInput("keyboard"));
        entries.push({ kind: "game", game: g, path: `/games/${g.name}.panowww.json` });
        return g;
      }
      case "duplicate_game": {
        const src = entries.find((e) => e.kind === "game" && e.game.id === args.id);
        if (src?.kind !== "game") {
          return Promise.reject({ kind: "notFound", message: "The Game was not found." });
        }
        const g = { ...src.game, id: crypto.randomUUID(), name: `${src.game.name} copy` };
        entries.push({ kind: "game", game: g, path: `/games/${g.name}.panowww.json` });
        return g;
      }
      case "delete_game": {
        const i = entries.findIndex((e) => e.kind === "game" && e.game.id === args.id);
        if (i >= 0) {
          entries.splice(i, 1);
        }
        return null;
      }
      case "delete_game_file": {
        const i = entries.findIndex((e) => e.path === args.path);
        if (i >= 0) {
          entries.splice(i, 1);
        }
        return null;
      }
      case "estimate_session": {
        const g = args.game as Game;
        const shots = g.shotsPerRow * g.rows;
        const ms = 5000 + shots * (g.waitAfterShotMs + g.waitAfterMoveMs + 386);
        return {
          durationMs: ms,
          duration: `≈ ${Math.floor(ms / 60000)} min ${Math.round((ms % 60000) / 1000)} s`,
          shots,
        };
      }
      case "select_game":
      case "set_shortcuts_enabled":
      case "show_zone":
      case "overlay_state":
      case "reveal_path":
      case "open_in_stitcher":
      case "open_url":
        return null;
      case "share_game":
        return true;
      case "import_game":
        return null;
      case "get_zone":
        return zoneState(Boolean(zones[String(args.gameId)]));
      case "pick_zone": {
        // The picker windows are their own pages; here the pick just lands.
        zones[String(args.gameId)] = true;
        return new Promise((resolve) => setTimeout(() => resolve(zoneState(true)), 300));
      }
      case "get_settings":
        return settings;
      case "save_settings":
        return null;
      case "start_run":
        simulateRun(args.kind as RunKindDto);
        return true;
      case "stop_run":
        return null;
      case "runner_phase":
        return "idle";
      case "check_for_updates":
        return { version: "0.2.0", url: "https://github.com/Respawwwn/panowww/releases" };
      case "session_size":
        return 483_000_000;
      case "zip_session":
        return `${args.folder}.zip`;
      case "session_thumbnail":
        return sample(
          Number(String(args.file).slice(1, 3)) * 7 + Number(String(args.file).slice(5, 7)) * 3,
        );
      case "list_sessions":
        return sessions.filter((s) => !args.gameId || s.record.game.id === args.gameId);
      case "delete_session": {
        const i = sessions.findIndex((s) => s.folder === args.folder);
        if (i >= 0) {
          sessions.splice(i, 1);
        }
        return null;
      }
      case "request_permission":
        return true;
      case "open_system_settings":
      case "reveal_logs":
      case "apply_shortcuts":
        return null;
      case "default_sessions_folder":
        return defaultSettings.sessionsFolder;
      case "choose_sessions_folder":
        return "/Volumes/Shots/Panowww";
      case "stitcher_check":
        return args.stitcher === "ptgui"
          ? { installed: true, version: "13.10" }
          : { installed: true, version: "4.4.2" };
      case "choose_stitcher":
        return args.stitcher === "ptgui"
          ? "/Applications/PTGui Pro.app"
          : "/Applications/Autopano Giga.app";
      case "permissions":
        return { accessibility: true, screenRecording: true, vigem: true };
      default:
        return Promise.reject({ kind: "internal", message: `mock: ${cmd} is not mocked` });
    }
  });
}

/** A purple 16:9 "Sample" with a stripe at `x` percent, as a data URL. */
function sample(x: number): string {
  const svg = `<svg xmlns='http://www.w3.org/2000/svg' width='640' height='360'><rect width='640' height='360' fill='%236454b5'/><rect x='${x}%' y='40' width='40' height='280' fill='%23fff'/><circle cx='${x + 20}%' cy='180' r='30' fill='%23ffb0c2'/></svg>`;
  return `data:image/svg+xml,${svg}`;
}

/** Plays the events of a run through the store, a few hundred ms apart. */
function simulateRun(kind: RunKindDto): void {
  const steps: RunEvent[] = [
    { event: "countdown", secondsLeft: 2 },
    { event: "countdown", secondsLeft: 1 },
    { event: "started", expected: kind.kind === "session" ? 78 : 0 },
  ];
  if (kind.kind === "session") {
    steps.push({
      event: "preview",
      image: sample(30),
      displayW: 2560,
      displayH: 1440,
      zone: {
        displayId: "main",
        x: 320,
        y: 180,
        w: 1920,
        h: 1080,
        displayW: 2560,
        displayH: 1440,
        scale: 2,
      },
    });
    for (let i = 1; i <= 8; i += 1) {
      const row = Math.floor((i - 1) / 13) + 1;
      const col = ((i - 1) % 13) + 1;
      steps.push({ event: "shot", index: i, row, col, captureMs: 40, thumbnail: sample(10 * i) });
      steps.push({ event: "moved", row, col: col + 1 });
      if (i === 2) {
        steps.push({ event: "screenUnchanged" });
      }
    }
  } else if (kind.kind === "checkOverlap") {
    steps.push({ event: "sample", slot: "first", image: sample(20) });
    steps.push({ event: "moved", row: 1, col: 2 });
    steps.push({ event: "sample", slot: "second", image: sample(60) });
  } else if (kind.kind === "turnTest" || kind.kind === "pitchTest") {
    for (let i = 2; i <= 6; i += 1) {
      steps.push({ event: "moved", row: 1, col: i });
    }
    if (kind.kind === "pitchTest") {
      steps.push({ event: "screenUnchanged" });
    }
  } else if (kind.kind === "stepOnce") {
    steps.push({ event: "moved", row: 1, col: 2 });
  }
  steps.push(
    kind.kind === "session"
      ? {
          event: "finished",
          outcome: { status: "interrupted", shots: 8 },
          folder: "/Users/me/Pictures/Panowww/Elden Ring - photo mode/2026-10-30_16-30-00",
        }
      : { event: "finished", outcome: { status: "completed", shots: 0 }, folder: null },
  );
  for (const [i, step] of steps.entries()) {
    setTimeout(() => applyRunEvent(step), 200 * (i + 1));
  }
}

/** A few Sessions of the sample Games, newest first. */
function sampleSessions(entries: GameEntry[]): SessionSummary[] {
  const games = entries.flatMap((e) => (e.kind === "game" ? [e.game] : []));
  const zone = {
    displayId: "main",
    x: 320,
    y: 180,
    w: 1920,
    h: 1080,
    displayW: 2560,
    displayH: 1440,
    scale: 2,
  };
  const make = (
    game: Game,
    startedAt: string,
    status: "completed" | "interrupted" | "failed",
    taken: number,
  ): SessionSummary => ({
    folder: `/Users/me/Pictures/Panowww/${game.name}/${startedAt.slice(0, 19).replace("T", "_").replaceAll(":", "-")}`,
    record: {
      format: "panowww-session",
      version: 1,
      id: crypto.randomUUID(),
      appVersion: "0.0.1-mock",
      startedAt,
      endedAt: startedAt,
      status,
      ...(status === "failed"
        ? {
            error: {
              kind: "displayGone" as const,
              message: "The display of the Capture zone is gone.",
            },
          }
        : {}),
      game,
      zone,
      shotsExpected: game.shotsPerRow * game.rows,
      shotsTaken: taken,
      shots: [],
    },
  });
  const [a, b] = games;
  if (!a || !b) {
    return [];
  }
  return [
    make(a, "2026-10-30T14:32:00+01:00", "completed", 78),
    make(a, "2026-10-30T14:10:00+01:00", "interrupted", 31),
    make(b, "2026-10-29T22:05:00+01:00", "failed", 52),
    make(a, "2026-10-27T19:48:00+01:00", "completed", 78),
  ];
}
