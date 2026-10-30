<script lang="ts">
// Lavender sidebar (spec §11.1): mark and wordmark, the Games list, the
// + New / Import… toolbar, the All Sessions / Settings / About footer.
// Greyed to 45 % during a countdown or a run. The list arrives with issue 15.
import { nav } from "../lib/nav.svelte";
import { strings } from "../lib/strings";

type Props = {
  onnew?: () => void;
  onimport?: () => void;
  onsessions?: () => void;
  onsettings?: () => void;
  onabout?: () => void;
};

let { onnew, onimport, onsessions, onsettings, onabout }: Props = $props();
</script>

<aside class="sidebar" class:dim={nav.busy}>
  <header class="brand" data-tauri-drag-region>
    <span class="mark" aria-hidden="true"></span>
    <span>{strings.app.name}</span>
  </header>

  <nav class="games" aria-label="Games">
    <!-- Game rows arrive with issue 15. -->
  </nav>

  <div class="actions">
    <button type="button" class="tbtn" disabled={!onnew || nav.busy} onclick={onnew}>+ {strings.sidebar.new}</button>
    <button type="button" class="tbtn" disabled={!onimport || nav.busy} onclick={onimport}>{strings.sidebar.import}</button>
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
