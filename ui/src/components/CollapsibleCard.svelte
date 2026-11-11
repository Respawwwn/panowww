<script lang="ts">
// Collapsible card of the Setup left pane (spec §11.2): numbered badge,
// title, one-line summary, chevron; one open at a time is the parent's job.
// `flash` (a counter) glows the header once, when a field inside changed
// from elsewhere.
import type { Snippet } from "svelte";
import Badge from "./Badge.svelte";

type Props = {
  n: number | string;
  title: string;
  summary?: string;
  open: boolean;
  ontoggle?: () => void;
  flash?: number;
  children?: Snippet;
};

let { n, title, summary = "", open, ontoggle, flash = 0, children }: Props = $props();
</script>

<section class="card" class:open>
  {#key flash}
    <button type="button" class="head" class:flash={flash > 0} aria-expanded={open} onclick={ontoggle}>
      <Badge {n} />
      <b>{title}</b>
      <span class="sum">{summary}</span>
      <svg class="chev" viewBox="0 0 16 16" aria-hidden="true">
        <path d={open ? "M4 10l4-4 4 4" : "M4 6l4 4 4-4"} />
      </svg>
    </button>
  {/key}
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

  .head.flash {
    animation: head-flash 0.9s ease-out;
  }

  @keyframes head-flash {
    0% {
      background: var(--lav);
    }
    100% {
      background: transparent;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .head.flash {
      animation: head-flash 0.9s steps(1, end);
    }
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 4px 14px 14px 50px;
  }
</style>
