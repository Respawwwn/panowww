<script lang="ts">
// App-shell bar at the bottom of every screen (spec §11.1). It only shows
// the update banner, which issue 18 wires; until then it stays empty.
let message = $state<string | null>(null);
</script>

{#if message}
  <div class="bar" role="status">{message}</div>
{/if}

<style>
  .bar {
    grid-area: bottom;
    padding: 8px 24px;
    background: var(--lavender);
    color: var(--lavender-text);
    border-top: 1px solid var(--line);
    font-size: 12px;
  }
</style>
