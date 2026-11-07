<script lang="ts">
// Full-width header row of a Game (spec §11.1): title left, ghost actions
// right (Duplicate / Share / Delete, or Reveal / Delete for a damaged file).
import type { Snippet } from "svelte";

type Props = {
  title: string;
  /** Beside the title, on its baseline: the Sessions count and size. */
  aside?: Snippet;
  actions?: Snippet;
  /** The content below is scrolled: a soft shadow as it slides under. */
  scrolled?: boolean;
};

let { title, aside, actions, scrolled = false }: Props = $props();
</script>

<header class="header" class:scrolled>
  <div class="name">
    <h1 class="t-title">{title}</h1>
    {#if aside}
      <span class="aside">{@render aside()}</span>
    {/if}
  </div>
  {#if actions}
    <div class="actions">{@render actions()}</div>
  {/if}
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 24px 12px;
    flex: none;
    position: relative;
    z-index: 1;
    transition: box-shadow 0.15s ease;
  }

  .header.scrolled {
    box-shadow: var(--scroll-shadow);
  }

  .name {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-width: 0;
  }

  h1 {
    margin: 0;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .aside {
    flex: none;
    color: var(--muted);
    font-size: 12.5px;
  }

  .actions {
    display: flex;
    gap: 8px;
    flex: none;
  }
</style>
