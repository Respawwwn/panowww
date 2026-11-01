<script lang="ts">
// The window shell (spec §11.1): sidebar, detail, bottom bar. Navigation is
// state (`nav`), not routes; a run moves it to the countdown, running and
// end screens; `?gallery` shows the component gallery.
import { onMount } from "svelte";
import BottomBar from "./components/BottomBar.svelte";
import ImportPrompt from "./components/ImportPrompt.svelte";
import Sidebar from "./components/Sidebar.svelte";
import { api } from "./lib/api";
import { screenForRun } from "./lib/flow";
import { games } from "./lib/games.svelte";
import { nav } from "./lib/nav.svelte";
import { run } from "./lib/run.svelte";
import { viewFromLast } from "./lib/sessionView";
import { settings } from "./lib/settings.svelte";
import { detectPlatform } from "./lib/shortcut";
import { applyAppearance } from "./lib/theme";
import { update } from "./lib/update.svelte";
import { windows } from "./lib/windows.svelte";
import About from "./screens/About.svelte";
import AllSessions from "./screens/AllSessions.svelte";
import Countdown from "./screens/Countdown.svelte";
import DamagedGame from "./screens/DamagedGame.svelte";
import EmptyDetail from "./screens/EmptyDetail.svelte";
import Gallery from "./screens/Gallery.svelte";
import GameDetail from "./screens/GameDetail.svelte";
import Onboarding from "./screens/Onboarding.svelte";
import SessionEnd from "./screens/SessionEnd.svelte";
import SessionHistory from "./screens/SessionHistory.svelte";
import SessionRunning from "./screens/SessionRunning.svelte";
import Settings from "./screens/Settings.svelte";

onMount(() => {
  if (nav.screen.kind === "gallery") {
    return;
  }
  let unwatch: (() => void) | null = null;
  let unrun: (() => void) | null = null;
  void (async () => {
    await Promise.allSettled([settings.load(), games.load()]);
    unwatch = await games.watch().catch(() => null);
    unrun = await run.subscribe().catch(() => null);
    // Onboarding shows whenever a permission is missing at launch (§11.11).
    const permissions = await api.onboarding.permissions().catch(() => null);
    if (
      permissions &&
      !(permissions.accessibility && permissions.screenRecording && permissions.vigem)
    ) {
      windows.show("onboarding");
    }
    await update.check();
  })();
  return () => {
    unwatch?.();
    unrun?.();
  };
});

// A run drives the screen (spec §11.5 to §11.7). A shortcut-started run
// is attributed to the selected Game.
$effect(() => {
  run.expectShortcut(nav.gameId);
});

// Light / dark / the OS appearance, from Settings (spec §12).
$effect(() => {
  applyAppearance(settings.value.appearance);
});

$effect(() => {
  const next = screenForRun(run.current, run.last, nav.screen);
  if (next) {
    nav.show(next);
  }
});

const damagedEntry = $derived.by(() => {
  const screen = nav.screen;
  return screen.kind === "damaged"
    ? games.entries.find((e) => e.kind === "damaged" && e.path === screen.path)
    : null;
});

/** The Game of a run screen, from the list. */
const runGame = $derived.by(() => {
  const screen = nav.screen;
  return "gameId" in screen ? games.find(screen.gameId) : null;
});

/** The run that just ended, for the end screen. */
const endedView = $derived.by(() => {
  const screen = nav.screen;
  const last = run.last;
  return screen.kind === "ended" && runGame && last
    ? viewFromLast(last, runGame, screen.folder)
    : null;
});

// Windows radii and chrome (spec §12): the shell carries the platform.
const platform = detectPlatform();

// Cmd+, / Ctrl+, opens Settings (§11).
function onKeyDown(event: KeyboardEvent) {
  if (event.key === "," && (event.metaKey || event.ctrlKey) && !nav.busy) {
    event.preventDefault();
    windows.show("settings");
  }
}
</script>

<svelte:window onkeydown={onKeyDown} />

{#if nav.screen.kind === "gallery"}
  <Gallery />
{:else}
  <div class="shell" data-platform={platform === "windows" ? "windows" : undefined}>
    <Sidebar
      onsessions={() => nav.showAllSessions()}
      onsettings={() => windows.show("settings")}
      onabout={() => windows.show("about")}
    />
    <main class="detail">
      {#if !["game", "ended", "session", "running", "countdown", "allSessions"].includes(nav.screen.kind)}
        <!-- The hidden title bar: the strip under it moves the window. The
             Game detail and the Session screens have a toolbar that is its
             own drag region. -->
        <div class="drag" data-tauri-drag-region></div>
      {/if}
      {#if nav.screen.kind === "game" && games.selected}
        <GameDetail game={games.selected} tab={nav.screen.tab} />
      {:else if damagedEntry?.kind === "damaged"}
        <DamagedGame entry={damagedEntry} />
      {:else if nav.screen.kind === "countdown" && runGame}
        <Countdown game={runGame} />
      {:else if nav.screen.kind === "running" && runGame}
        <SessionRunning game={runGame} />
      {:else if nav.screen.kind === "ended" && runGame && endedView}
        <SessionEnd view={endedView} />
      {:else if nav.screen.kind === "session"}
        <SessionHistory gameId={nav.screen.gameId} folder={nav.screen.folder} />
      {:else if nav.screen.kind === "allSessions"}
        <AllSessions />
      {:else}
        <div class="pane">
          <EmptyDetail onnew={() => games.create()} onimport={() => games.import()} />
        </div>
      {/if}
    </main>
    <BottomBar />
    <ImportPrompt />
    {#if windows.open === "settings"}
      <Settings />
    {:else if windows.open === "about"}
      <About />
    {:else if windows.open === "onboarding"}
      <Onboarding />
    {/if}
  </div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    /* minmax(0, …): a long Games list must scroll, not stretch the row. */
    grid-template-rows: minmax(0, 1fr) auto;
    grid-template-areas:
      "sidebar detail"
      "bottom bottom";
    height: 100%;
  }

  .detail {
    grid-area: detail;
    position: relative;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .pane {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 24px;
  }
  .drag {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 48px;
    z-index: 1;
  }
  /* The native caption bar moves the window on Windows. */
  .shell[data-platform="windows"] .drag {
    display: none;
  }
</style>
