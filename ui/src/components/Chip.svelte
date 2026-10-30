<script lang="ts">
// Chips (spec §12): completed (ok), interrupted (warn), failed (bad), and a
// lavender one for neutral facts such as the Overlap percent.
import type { Snippet } from "svelte";

type Props = {
  tone?: "ok" | "warn" | "bad" | "lav";
  children?: Snippet;
};

let { tone = "lav", children }: Props = $props();
</script>

<span class="chip {tone}">{@render children?.()}</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 1px 9px 0;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
  }

  .ok {
    background: var(--ok-soft);
    color: var(--ok-fg);
  }

  .warn {
    background: var(--warn-soft);
    color: var(--warn-fg);
  }

  .bad {
    background: var(--bad-soft);
    color: var(--bad-fg);
  }

  .lav {
    background: var(--lav);
    color: var(--lav-fg);
  }
</style>
