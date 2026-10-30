<script lang="ts">
// The Sessions tab of a Game (spec §11.9): its table and the footer line
// naming the Sessions folder, linking to Settings.
import SessionsTable from "../components/SessionsTable.svelte";
import { settings } from "../lib/settings.svelte";
import { interpolate, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

type Props = { gameId: string };

let { gameId }: Props = $props();
</script>

<div class="tab">
  <SessionsTable {gameId} />
  <p class="t-tip footer">
    <button type="button" class="link" onclick={() => windows.show("settings")}>
      {interpolate(strings.sessions.folder, { path: settings.value.sessionsFolder })}
    </button>
  </p>
</div>

<style>
  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .footer {
    margin: 0;
  }

  .link {
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--muted);
    font: inherit;
    text-align: left;
  }

  .link:hover {
    color: var(--lav-fg);
  }
</style>
