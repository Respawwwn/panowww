<script lang="ts">
// Import of a Game whose id is already known (spec §5): Replace keeps the
// local zone; Keep both gives the copy a new id and a suffixed name.
import { games } from "../lib/games.svelte";
import { interpolate, strings } from "../lib/strings";
import Button from "./Button.svelte";
</script>

{#if games.pendingImport}
  <div class="scrim" role="presentation">
    <div class="card prompt" role="alertdialog" aria-labelledby="import-title">
      <h2 id="import-title" class="t-section">{strings.import.title}</h2>
      <p>
        {interpolate(strings.import.body, {
          existing: games.pendingImport.existing.name,
          incoming: games.pendingImport.incoming.name,
        })}
      </p>
      <div class="actions">
        <Button variant="ghost" onclick={() => games.resolveImport("cancel")}>{strings.import.cancel}</Button>
        <Button onclick={() => games.resolveImport("keepBoth")}>{strings.import.keepBoth}</Button>
        <Button variant="primary" onclick={() => games.resolveImport("replace")}>{strings.import.replace}</Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--dim);
  }

  .prompt {
    width: 420px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  h2,
  p {
    margin: 0;
  }

  p {
    color: var(--muted);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
