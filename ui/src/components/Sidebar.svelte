<script lang="ts">
// Lavender sidebar: mark, Games list, toolbar, footer (spec §11.1). The
// list and its actions arrive with issue 15; the frame is here.
import { strings } from "../lib/strings";
</script>

<aside class="sidebar">
  <header class="brand" data-tauri-drag-region>
    <span class="mark" aria-hidden="true"></span>
    <span class="wordmark">{strings.app.name}</span>
  </header>

  <nav class="games" aria-label="Games">
    <!-- Game rows arrive with issue 15. -->
  </nav>

  <div class="toolbar">
    <button type="button" class="ghost" disabled>+ {strings.sidebar.new}</button>
    <button type="button" class="ghost" disabled>{strings.sidebar.import}</button>
  </div>

  <footer class="footer">
    <button type="button" class="ghost" disabled>{strings.sidebar.allSessions}</button>
    <span class="spacer"></span>
    <button type="button" class="ghost icon" disabled aria-label={strings.sidebar.settings}>⚙</button>
    <button type="button" class="ghost icon" disabled aria-label={strings.sidebar.about}>?</button>
  </footer>
</aside>

<style>
  .sidebar {
    grid-area: sidebar;
    display: flex;
    flex-direction: column;
    background: var(--sidebar);
    border-right: 1px solid var(--line);
    padding: 12px;
    /* Room for the macOS traffic lights under the overlay title bar. */
    padding-top: 44px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 8px 12px;
    font-weight: 700;
    font-size: 14px;
    color: var(--lavender-text);
  }

  .mark {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    background: var(--accent);
  }

  .games {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .toolbar,
  .footer {
    display: flex;
    gap: 4px;
    padding-top: 8px;
  }

  .spacer {
    flex: 1;
  }

  .ghost {
    height: var(--control-height);
    padding: 0 10px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--lavender-text);
    font-size: 12px;
    font-weight: 500;
  }

  .ghost.icon {
    width: var(--control-height);
    padding: 0;
  }

  .ghost:disabled {
    opacity: 0.45;
  }
</style>
