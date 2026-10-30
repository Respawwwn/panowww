<script lang="ts">
// Full-width header row of a Game (spec §11.1): title left, ghost actions
// right (Duplicate / Share / Delete, or Reveal / Delete for a damaged file).
import type { Snippet } from "svelte";

type Props = {
  title: string;
  actions?: Snippet;
};

let { title, actions }: Props = $props();
</script>

<header class="header">
  <h1 class="t-title">{title}</h1>
  <div class="actions">{@render actions?.()}</div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 24px 0;
  }

  h1 {
    margin: 0;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    gap: 4px;
    flex: none;
  }
</style>
