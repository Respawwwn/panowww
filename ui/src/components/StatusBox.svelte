<script lang="ts">
// Lavender status box with a green / red / amber dot (spec §12): the
// Capture zone status, the permission rows.
import type { Snippet } from "svelte";

type Props = {
  tone?: "ok" | "bad" | "warn";
  children?: Snippet;
};

let { tone = "ok", children }: Props = $props();
</script>

<div class="status">
  <span class="dot {tone}"></span>
  <span class="text">{@render children?.()}</span>
</div>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--lav);
    color: var(--lav-fg);
    font-size: 12.5px;
    font-weight: 500;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
    background: var(--ok);
  }

  .dot.bad {
    background: var(--bad);
  }

  .dot.warn {
    background: var(--warn);
  }

  .text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
