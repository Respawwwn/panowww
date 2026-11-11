<script lang="ts">
// A form row (spec §11.2): left-aligned label, the control, a tip under it,
// and the validation message inline when the value breaks a rule. `flash`
// (a counter, 0 for none) brings the row into view and glows its control
// once per change, for the Calibrate card's buttons.
import type { Snippet } from "svelte";
import type { InterpolationValues } from "../lib/strings";
import Tip from "./Tip.svelte";

type Props = {
  label: string;
  /** `for` of the label; the control carries the same id. */
  id?: string;
  tip?: string;
  tipValues?: InterpolationValues;
  error?: string | null;
  flash?: number;
  children?: Snippet;
};

let { label, id, tip, tipValues, error = null, flash = 0, children }: Props = $props();

let row = $state<HTMLDivElement | null>(null);

$effect(() => {
  if (flash > 0) {
    row?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
});
</script>

<div class="row" bind:this={row}>
  <label for={id}>{label}</label>
  <div class="ctl">
    {#key flash}
      <div class="h" class:flash={flash > 0}>{@render children?.()}</div>
    {/key}
    {#if error}
      <p class="error" role="alert">{error}</p>
    {:else if tip}
      <Tip text={tip} values={tipValues} />
    {/if}
  </div>
</div>

<style>
  .h {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }

  /* The inputs inside a field share one look (b.css `.field`). */
  .h :global(input:not([type="checkbox"])),
  .h :global(select),
  .h :global(textarea) {
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--line2);
    border-radius: var(--rs);
    background: var(--field);
    color: var(--fg);
    font-weight: 500;
    width: 84px;
  }

  .h :global(textarea) {
    height: auto;
    min-height: 56px;
    padding: 6px 10px;
    resize: vertical;
    font-weight: 400;
  }

  .h :global(input.wide),
  .h :global(select.wide),
  .h :global(textarea.wide) {
    width: 100%;
  }

  .h :global(input:focus-visible),
  .h :global(select:focus-visible),
  .h :global(textarea:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .h :global(.unit) {
    color: var(--muted);
    font-size: 12px;
  }

  /* The glow of a value the Calibrate card just changed. */
  .h.flash :global(input:not([type="checkbox"])) {
    animation: flash 0.9s ease-out;
  }

  @keyframes flash {
    0% {
      box-shadow: 0 0 0 3px var(--accent);
      background: var(--lav);
    }
    100% {
      box-shadow: 0 0 0 3px transparent;
      background: var(--field);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .h.flash :global(input:not([type="checkbox"])) {
      animation: flash-still 0.9s steps(1, end);
    }

    @keyframes flash-still {
      0% {
        box-shadow: 0 0 0 3px var(--accent);
        background: var(--lav);
      }
      100% {
        box-shadow: none;
        background: var(--field);
      }
    }
  }
</style>
