<script lang="ts">
// Lavender sidebar (spec §11.1): mark and wordmark, the Games list (name
// and device icon, amber ⚠ for a damaged file, selected row purple), the
// + New / Import… toolbar, the All Sessions / Settings / About footer.
// Greyed to 45 % during a countdown or a run.
import { games } from "../lib/games.svelte";
import { nav } from "../lib/nav.svelte";
import { strings } from "../lib/strings";
import DeviceIcon from "./DeviceIcon.svelte";
import Icon from "./Icon.svelte";
import Mark from "./Mark.svelte";

type Props = {
  onsessions?: () => void;
  onsettings?: () => void;
  onabout?: () => void;
};

let { onsessions, onsettings, onabout }: Props = $props();

function fileName(path: string): string {
  return (
    path
      .split("/")
      .pop()
      ?.replace(/\.panowww\.json$/, "") ?? path
  );
}

const selectedPath = $derived(nav.screen.kind === "damaged" ? nav.screen.path : null);

// Reordering by drag (HTML5 drag and drop inside the window; Tauri's own
// file drag and drop is off). The dragged Game lands before the row the
// pointer is over, or at the end when dropped on the list's free space.
let dragging = $state<string | null>(null);
let over = $state<string | "end" | null>(null);

function dragStart(event: DragEvent, id: string) {
  dragging = id;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", id);
  }
}

function dragOver(event: DragEvent, target: string | "end") {
  if (!dragging || dragging === target) {
    return;
  }
  event.preventDefault();
  over = target;
}

async function drop(target: string | "end") {
  const id = dragging;
  dragging = null;
  over = null;
  if (!id || id === target) {
    return;
  }
  await games.move(id, target === "end" ? null : target);
}

function dragEnd() {
  dragging = null;
  over = null;
}
</script>

<aside class="sidebar" class:dim={nav.busy}>
  <header class="brand" data-tauri-drag-region>
    <button type="button" class="home" disabled={nav.busy} onclick={() => games.select(null)}>
      <Mark />
      <span>{strings.app.name}</span>
    </button>
  </header>

  <div class="side-title">
    <span>{strings.sidebar.games}</span>
    <span class="side-actions">
      <button type="button" class="ibtn" disabled={nav.busy} onclick={() => games.create()} title={strings.sidebar.new} aria-label={strings.sidebar.new}>
        <Icon name="plus" size={14} />
      </button>
      <button type="button" class="ibtn" disabled={nav.busy} onclick={() => games.import()} title={strings.sidebar.import} aria-label={strings.sidebar.import}>
        <Icon name="import" size={14} />
      </button>
    </span>
  </div>
  <nav class="games" aria-label={strings.sidebar.games}>
    {#if games.loaded && games.entries.length === 0}
      <p class="none">{strings.sidebar.noGames}</p>
    {/if}
    <ul>
      {#each games.entries as entry (entry.path)}
        {#if entry.kind === "game"}
          <li
            class:over={over === entry.game.id}
            class:dragged={dragging === entry.game.id}
            draggable="true"
            ondragstart={(e) => dragStart(e, entry.game.id)}
            ondragover={(e) => dragOver(e, entry.game.id)}
            ondrop={() => drop(entry.game.id)}
            ondragend={dragEnd}
          >
            <button
              type="button"
              class="game"
              class:on={nav.gameId === entry.game.id}
              onclick={() => games.select(entry.game.id)}
            >
              <DeviceIcon device={entry.game.input.device} />
              <span>{entry.game.name}</span>
            </button>
          </li>
        {:else}
          <li>
            <button
              type="button"
              class="game"
              class:on={selectedPath === entry.path}
              onclick={() => nav.showDamaged(entry.path)}
              title={strings.sidebar.damaged}
            >
              <span class="warn" aria-label={strings.sidebar.damaged}>⚠</span>
              <span>{fileName(entry.path)}</span>
            </button>
          </li>
        {/if}
      {/each}
    </ul>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="tail" class:over={over === "end"} ondragover={(e) => dragOver(e, "end")} ondrop={() => drop("end")}></div>
  </nav>


  <footer class="foot">
    <button
      type="button"
      class="footrow"
      class:on={nav.screen.kind === "allSessions"}
      disabled={!onsessions || nav.busy}
      onclick={onsessions}
    >
      <Icon name="list" />
      {strings.sidebar.allSessions}
    </button>
    <button type="button" class="footrow" disabled={!onsettings || nav.busy} onclick={onsettings}>
      <Icon name="gear" />
      {strings.sidebar.settings}
      <kbd>⌘,</kbd>
    </button>
    <button type="button" class="footrow" disabled={!onabout || nav.busy} onclick={onabout}>
      <Icon name="help" />
      {strings.sidebar.about}
    </button>
  </footer>
</aside>

<style>
  .sidebar {
    grid-area: sidebar;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
    background: var(--sidebar);
    /* Room for the macOS traffic lights under the overlay title bar. */
    padding-top: 52px;
    transition: opacity 0.15s ease-out;
  }

  .sidebar.dim {
    opacity: 0.45;
    pointer-events: none;
  }

  .brand {
    display: flex;
    align-items: center;
    padding: 4px 16px 12px;
  }

  /* The mark and the name go home; the rest of the header drags the window. */
  .home {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-left: -6px;
    padding: 2px 6px;
    border: 0;
    border-radius: var(--rs);
    background: transparent;
    color: var(--fg);
    font-weight: 700;
    font-size: 14px;
    letter-spacing: -0.01em;
  }

  .home:hover:not(:disabled) {
    background: var(--hover);
  }

  .home > :global(*) {
    pointer-events: none;
  }

  .side-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 10px 4px 16px;
    font-size: 11px;
    font-weight: 600;
    color: var(--faint);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .side-actions {
    display: flex;
    gap: 2px;
  }

  .ibtn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 0;
    border-radius: var(--rs);
    background: transparent;
    color: var(--lav-fg);
    padding: 0;
  }

  .ibtn:hover:not(:disabled) {
    background: var(--hover);
  }

  .ibtn:disabled {
    opacity: 0.45;
  }

  /* The list scrolls on its own; the footer always stays in view. */
  .games {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 10px;
  }

  .none {
    margin: 0;
    padding: 4px 6px;
    color: var(--muted);
    font-size: 12.5px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .game {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: var(--row-height);
    padding: 0 10px;
    border: 0;
    border-radius: var(--rs);
    background: transparent;
    color: var(--fg);
    font-weight: 500;
    text-align: left;
  }

  .game:hover:not(.on) {
    background: var(--hover);
  }

  li {
    position: relative;
  }

  li.dragged {
    opacity: 0.4;
  }

  li.over::before,
  .tail.over::before {
    content: "";
    position: absolute;
    left: 10px;
    right: 10px;
    top: -2px;
    height: 2px;
    border-radius: 1px;
    background: var(--accent);
  }

  .tail {
    position: relative;
    height: 10px;
  }

  .game.on {
    background: var(--accent);
    color: var(--accent-fg);
  }

  .game span:last-child {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .game :global(.ic) {
    opacity: 0.85;
  }

  .warn {
    width: 17px;
    text-align: center;
    color: var(--warn);
    flex: none;
  }

  .game.on .warn {
    color: var(--accent-fg);
  }

  .footrow:disabled {
    opacity: 0.45;
  }

  .foot {
    flex: none;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .footrow {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--rs);
    background: transparent;
    border: 0;
    color: var(--fg);
    text-align: left;
  }

  .footrow:hover:not(:disabled) {
    background: var(--hover);
  }

  .footrow.on {
    background: var(--accent);
    color: var(--accent-fg);
  }

  .footrow.on :global(.ic) {
    color: inherit;
  }

  .footrow kbd {
    margin-left: auto;
    font: inherit;
    font-size: 11px;
    color: var(--faint);
  }
</style>
