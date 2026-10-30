<script lang="ts">
// Buttons of spec §12: primary pill (purple), secondary (white with a
// lavender border), ghost, danger (soft red). Sizes sm / md / lg.
import type { Snippet } from "svelte";
import type { HTMLButtonAttributes } from "svelte/elements";

type Props = HTMLButtonAttributes & {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  children?: Snippet;
};

let { variant = "secondary", size = "md", children, ...rest }: Props = $props();
</script>

<button type="button" class="btn {variant} {size}" {...rest}>
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    padding: 1px 12px 0; /* 1 px optical nudge for Inter */
    border: 1px solid var(--btn-line);
    border-radius: 999px;
    background: var(--btn);
    color: var(--fg);
    font-size: 12px;
    font-weight: 500;
    line-height: 1;
    box-shadow: var(--shadow);
    white-space: nowrap;
  }

  .btn:hover:not(:disabled) {
    background: var(--hover);
  }

  .btn:disabled {
    opacity: 0.45;
  }

  .sm {
    height: 24px;
    padding-inline: 10px;
    font-size: 11px;
  }

  .lg {
    height: 36px;
    padding-inline: 16px;
    font-size: 13px;
  }

  .primary {
    border-color: transparent;
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
    box-shadow: none;
  }

  .primary.lg {
    padding-inline: 18px;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-deep);
  }

  .ghost {
    border-color: transparent;
    background: transparent;
    color: var(--lav-fg);
    box-shadow: none;
  }

  .danger {
    color: var(--bad-fg);
    border-color: var(--bad-soft);
    background: var(--bad-soft);
    box-shadow: none;
  }
</style>
