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
