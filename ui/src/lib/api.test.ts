import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, expect, test } from "vitest";
import { api } from "./api";

afterEach(() => {
  clearMocks();
});

test("ping goes through invoke and comes back typed", async () => {
  const calls: string[] = [];
  mockIPC((cmd) => {
    calls.push(cmd);
    if (cmd === "ping") {
      return { app: "Panowww", version: "0.0.1" };
    }
    throw new Error(`unexpected command ${cmd}`);
  });

  const pong = await api.ping();

  expect(calls).toEqual(["ping"]);
  expect(pong).toEqual({ app: "Panowww", version: "0.0.1" });
});

test("commands pass their arguments in camelCase", async () => {
  const seen: { cmd: string; args: unknown }[] = [];
  mockIPC((cmd, args) => {
    seen.push({ cmd, args });
    return null;
  });

  await api.games.delete("abc");
  await api.sessions.list();
  await api.run.setPreview(false);

  expect(seen).toEqual([
    { cmd: "delete_game", args: { id: "abc" } },
    { cmd: "list_sessions", args: { gameId: null } },
    { cmd: "set_preview", args: { on: false } },
  ]);
});

test("the Stitch commands and the Experimental switch go through as the Rust side expects", async () => {
  const seen: { cmd: string; args: unknown }[] = [];
  mockIPC((cmd, args) => {
    seen.push({ cmd, args });
    return cmd === "stitch_phase" ? { phase: "idle" } : cmd === "stitch_session" ? true : null;
  });

  const options = { antiGhost: false, unalignedShots: "leaveOut" as const, jpegQuality: 92 };
  expect(await api.stitch.start("/S/a", options)).toBe(true);
  await api.stitch.cancel();
  expect(await api.stitch.phase()).toEqual({ phase: "idle" });
  await api.stitch.preview("/S/a");
  const settings = await import("./settings.svelte").then((m) => m.defaultSettings);
  await api.settings.save({ ...settings, experimental: { stitch: true } });

  expect(seen.map((s) => s.cmd)).toEqual([
    "stitch_session",
    "stitch_cancel",
    "stitch_phase",
    "panorama_preview",
    "save_settings",
  ]);
  expect(seen[0]?.args).toEqual({ folder: "/S/a", options });
  expect(seen[3]?.args).toEqual({ folder: "/S/a" });
  const saved = seen[4]?.args as { settings: { experimental: { stitch: boolean } } } | undefined;
  expect(saved?.settings.experimental).toEqual({ stitch: true });
});
