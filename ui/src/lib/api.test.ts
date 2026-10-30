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
