<script lang="ts">
// The window shell (spec §11.1): sidebar, detail, bottom bar. Navigation is
// state (`nav`), not routes; a run moves it to the countdown, running and
// end screens; `?gallery` shows the component gallery.
import { onMount } from "svelte";
import BottomBar from "./components/BottomBar.svelte";
import ImportPrompt from "./components/ImportPrompt.svelte";
import Sidebar from "./components/Sidebar.svelte";
import { screenForRun } from "./lib/flow";
import { games } from "./lib/games.svelte";
import { nav } from "./lib/nav.svelte";
import { run } from "./lib/run.svelte";
import { settings } from "./lib/settings.svelte";
import { update } from "./lib/update.svelte";
import Countdown from "./screens/Countdown.svelte";
import DamagedGame from "./screens/DamagedGame.svelte";
import EmptyDetail from "./screens/EmptyDetail.svelte";
import Gallery from "./screens/Gallery.svelte";
import GameDetail from "./screens/GameDetail.svelte";
import SessionEnd from "./screens/SessionEnd.svelte";
import SessionRunning from "./screens/SessionRunning.svelte";

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
</script>

{#if nav.screen.kind === "gallery"}
  <Gallery />
{:else}
  <div class="shell">
    <Sidebar />
    <main class="detail">
      {#if nav.screen.kind === "game" && games.selected}
        <GameDetail game={games.selected} tab={nav.screen.tab} />
      {:else if damagedEntry?.kind === "damaged"}
        <DamagedGame entry={damagedEntry} />
      {:else if nav.screen.kind === "countdown" && runGame}
        <Countdown game={runGame} />
      {:else if nav.screen.kind === "running" && runGame}
        <SessionRunning game={runGame} />
      {:else if nav.screen.kind === "ended" && runGame}
        <SessionEnd game={runGame} folder={nav.screen.folder} />
      {:else}
        <div class="pane">
          <EmptyDetail onnew={() => games.create()} onimport={() => games.import()} />
        </div>
      {/if}
    </main>
    <BottomBar />
    <ImportPrompt />
  </div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-rows: 1fr auto;
    grid-template-areas:
      "sidebar detail"
      "bottom bottom";
    height: 100%;
  }

  .detail {
    grid-area: detail;
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
</style>
