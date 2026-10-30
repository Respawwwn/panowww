<script lang="ts">
// Lavender sidebar (spec §11.1): mark and wordmark, the Games list (name
// and device icon, amber ⚠ for a damaged file, selected row purple), the
// + New / Import… toolbar, the All Sessions / Settings / About footer.
// Greyed to 45 % during a countdown or a run.
import { games } from "../lib/games.svelte";
import { nav } from "../lib/nav.svelte";
import { strings } from "../lib/strings";
import DeviceIcon from "./DeviceIcon.svelte";

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
</script>

<aside class="sidebar" class:dim={nav.busy}>
  <header class="brand" data-tauri-drag-region>
    <span class="mark" aria-hidden="true"></span>
    <span>{strings.app.name}</span>
  </header>

  <nav class="games" aria-label="Games">
    <ul>
      {#each games.entries as entry (entry.path)}
        {#if entry.kind === "game"}
          <li>
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
  </nav>

  <div class="actions">
    <button type="button" class="tbtn" disabled={nav.busy} onclick={() => games.create()}>+ {strings.sidebar.new}</button>
    <button type="button" class="tbtn" disabled={nav.busy} onclick={() => games.import()}>{strings.sidebar.import}</button>
  </div>

  <footer class="foot">
    <button type="button" class="footrow" disabled={!onsessions || nav.busy} onclick={onsessions}>
      {strings.sidebar.allSessions}
    </button>
    <button type="button" class="footrow" disabled={!onsettings || nav.busy} onclick={onsettings}>
      {strings.sidebar.settings}
      <kbd>⌘,</kbd>
    </button>
    <button type="button" class="footrow" disabled={!onabout || nav.busy} onclick={onabout}>
      {strings.sidebar.about}
    </button>
  </footer>
</aside>

<style>
  .sidebar {
    grid-area: sidebar;
    display: flex;
    flex-direction: column;
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
    gap: 8px;
    padding: 4px 16px 12px;
    font-weight: 700;
    font-size: 14px;
    letter-spacing: -0.01em;
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: #6454b5;
    flex: none;
  }

  .games {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 10px;
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

  .actions {
    display: flex;
    gap: 6px;
    padding: 10px;
  }

  .tbtn {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 1px 10px 0;
    border-radius: var(--rs);
    background: transparent;
    border: 1px solid transparent;
    color: var(--lav-fg);
    font-size: 12px;
    font-weight: 500;
    line-height: 1;
  }

  .tbtn:hover:not(:disabled) {
    background: var(--hover);
  }

  .tbtn:disabled,
  .footrow:disabled {
    opacity: 0.45;
  }

  .foot {
    margin-top: auto;
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

  .footrow kbd {
    margin-left: auto;
    font: inherit;
    font-size: 11px;
    color: var(--faint);
  }
</style>
