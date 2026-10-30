<script lang="ts">
// Tips (spec §12): a muted 11.5 px line under a field, or a box in lavender
// (neutral), amber (warning) or green (verdict). Shortcut markers in the
// text render as key caps.
import type { InterpolationValues } from "../lib/strings";
import RichLine from "./RichLine.svelte";

type Props = {
  text: string;
  values?: InterpolationValues;
  /** `line` is the plain muted line; the others are boxes. */
  kind?: "line" | "lav" | "warn" | "ok";
};

let { text, values, kind = "line" }: Props = $props();
</script>

{#if kind === "line"}
  <p class="tip"><RichLine {text} {values} /></p>
{:else}
  <div class="tipbox {kind}"><RichLine {text} {values} /></div>
{/if}

<style>
  .tip {
    margin: 0;
    font-size: 11.5px;
    color: var(--muted);
  }

  .tipbox {
    font-size: 11.5px;
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--lav);
    color: var(--lav-fg);
  }

  .tipbox.warn {
    background: var(--warn-soft);
    color: var(--warn-fg);
  }

  .tipbox.ok {
    background: var(--ok-soft);
    color: var(--ok-fg);
  }
</style>
