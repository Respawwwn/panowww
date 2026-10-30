<script lang="ts">
// All Sessions (spec §11.9): the global table with a Game column, the
// count and total size beside the title, the folder footer.
import Button from "../components/Button.svelte";
import SessionsTable from "../components/SessionsTable.svelte";
import WindowFrame from "../components/WindowFrame.svelte";
import { api } from "../lib/api";
import { formatBytes } from "../lib/format";
import { settings } from "../lib/settings.svelte";
import { interpolate, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

let count = $state(0);
let total = $state(0);
</script>

<WindowFrame title={strings.sessions.title} width={960} onclose={() => windows.close()}>
  {#snippet aside()}
    <span class="t-label">{interpolate(strings.sessions.summary, { n: count, size: formatBytes(total) })}</span>
  {/snippet}
  <SessionsTable
    showGame
    onloaded={(rows, bytes) => {
      count = rows.length;
      total = bytes;
    }}
  />
  {#snippet footer()}
    <span class="t-tip">{interpolate(strings.sessions.folderShort, { path: settings.value.sessionsFolder })}</span>
    <span class="spacer"></span>
    <Button size="sm" onclick={() => api.settings.revealPath(settings.value.sessionsFolder)}>{strings.sessions.revealFolder}</Button>
    <Button size="sm" variant="ghost" onclick={() => windows.show("settings")}>{strings.sessions.changeInSettings}</Button>
  {/snippet}
</WindowFrame>

<style>
  .spacer {
    flex: 1;
  }
</style>
