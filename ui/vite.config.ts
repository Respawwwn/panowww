import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

// Tauri expects a fixed port and reads TAURI_* variables during `tauri dev`.
export default defineConfig(({ mode }) => ({
  plugins: [svelte()],
  // Vitest resolves Node conditions by default; Svelte components need the
  // browser build to mount in jsdom.
  resolve: mode === "test" ? { conditions: ["browser"] } : undefined,
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
    include: ["src/**/*.test.ts"],
  },
}));
