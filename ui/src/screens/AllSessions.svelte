<script lang="ts">
// All Sessions (spec §11.9): the global table with a Game column, shown in
// the detail area with its own toolbar: the title left, the count and
// total size right; the folder footer at the bottom.
import Button from "../components/Button.svelte";
import SessionsTable from "../components/SessionsTable.svelte";
import { api } from "../lib/api";
import { formatBytes } from "../lib/format";
import { nav } from "../lib/nav.svelte";
import { settings } from "../lib/settings.svelte";
import { interpolate, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

let count = $state(0);
let total = $state(0);
let scrolled = $state(false);
</script>

<div class="toolbar" class:scrolled data-tauri-drag-region>
  <h1>{strings.sessions.title}</h1>
  <span class="sub">{interpolate(strings.sessions.summary, { n: count, size: formatBytes(total) })}</span>
</div>

<div class="all" onscroll={(e) => (scrolled = e.currentTarget.scrollTop > 0)}>
  <SessionsTable
    showGame
    onloaded={(rows, bytes) => {
      count = rows.length;
      total = bytes;
    }}
    onopen={(id, folder) => nav.showSession(id, folder)}
  />
  <p class="t-tip footer">
    {strings.sessions.folder} <code>{settings.value.sessionsFolder}</code> ·
    <button type="button" class="link" onclick={() => windows.show("settings")}>{strings.sessions.changeFolder}</button>
    <span class="spacer"></span>
    <Button size="sm" onclick={() => api.settings.revealPath(settings.value.sessionsFolder)}>{strings.sessions.revealFolder}</Button>
  </p>
</div>

<style>
  .toolbar {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    flex: none;
    position: relative;
    z-index: 1;
    transition: box-shadow 0.15s ease;
  }

  .toolbar.scrolled {
    box-shadow: var(--scroll-shade);
  }

  h1 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
  }

  .all {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .footer {
    margin: auto 0 0;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .spacer {
    flex: 1;
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--lav);
    color: var(--lav-fg);
  }

  .link {
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--lav-fg);
    font: inherit;
  }

  .link:hover {
    text-decoration: underline;
  }
</style>
