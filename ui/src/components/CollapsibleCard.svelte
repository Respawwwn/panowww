<script lang="ts">
// Collapsible card of the Setup left pane (spec §11.2): numbered badge,
// title, one-line summary, chevron; one open at a time is the parent's job.
import type { Snippet } from "svelte";
import Badge from "./Badge.svelte";

type Props = {
  n: number | string;
  title: string;
  summary?: string;
  open: boolean;
  ontoggle?: () => void;
  children?: Snippet;
};

let { n, title, summary = "", open, ontoggle, children }: Props = $props();
</script>

<section class="card" class:open>
  <button type="button" class="head" aria-expanded={open} onclick={ontoggle}>
    <Badge {n} />
    <b>{title}</b>
    <span class="sum">{summary}</span>
    <svg class="chev" viewBox="0 0 16 16" aria-hidden="true">
      <path d={open ? "M4 10l4-4 4 4" : "M4 6l4 4 4-4"} />
    </svg>
  </button>
  {#if open}
    <div class="body">{@render children?.()}</div>
  {/if}
</section>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 14px;
    border: 0;
    background: transparent;
    text-align: left;
    color: var(--fg);
  }

  .head b {
    font-weight: 600;
    font-size: 14px;
  }

  .sum {
    margin-left: auto;
    color: var(--muted);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chev {
    width: 16px;
    height: 16px;
    flex: none;
    stroke: var(--faint);
    fill: none;
    stroke-width: 1.75;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 4px 14px 14px 50px;
  }
</style>
