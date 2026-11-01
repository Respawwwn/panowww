<script lang="ts">
// A Session opened from a table (spec §11.7): its folder's record read
// again, shown by the Session end screen with on-demand thumbnails.
import { api, isAppError } from "../lib/api";
import { games } from "../lib/games.svelte";
import { type SessionView, viewFromRecord } from "../lib/sessionView";
import { strings } from "../lib/strings";
import SessionEnd from "./SessionEnd.svelte";

type Props = { gameId: string; folder: string };

let { gameId, folder }: Props = $props();

let view = $state<SessionView | null>(null);
let error = $state<string | null>(null);

$effect(() => {
  const id = gameId;
  const f = folder;
  view = null;
  error = null;
  api.sessions
    .list(id)
    .then((rows) => {
      const row = rows.find((r) => r.folder === f);
      if (row) {
        view = viewFromRecord(row.record, row.folder);
      } else {
        error = strings.sessions.gone;
      }
    })
    .catch((e) => {
      error = isAppError(e) ? e.message : String(e);
    });
});
</script>

{#if view}
  <SessionEnd {view} />
{:else if error}
  <div class="pane">
    <p class="error">{error}</p>
    <button type="button" class="link" onclick={() => games.select(gameId)}>{strings.session.end.backToGame}</button>
  </div>
{/if}

<style>
  .pane {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error {
    margin: 0;
    font-size: 12.5px;
    color: var(--bad-fg);
  }

  .link {
    align-self: flex-start;
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--lav-fg);
    font: inherit;
    font-size: 12.5px;
    font-weight: 500;
  }
</style>
