<script lang="ts">
// A shortcut as flat lavender key caps joined by "+", never button-like
// (spec §12). The caps come from a Settings shortcut, so a rebinding shows
// everywhere at once.
import type { Shortcut } from "../lib/generated/Shortcut";
import { detectPlatform, type Platform, shortcutCaps } from "../lib/shortcut";

type Props = {
  shortcut: Shortcut;
  platform?: Platform;
};

let { shortcut, platform = detectPlatform() }: Props = $props();

const caps = $derived(shortcutCaps(shortcut, platform));
</script>

<span class="keys">
  {#each caps as cap, i (i)}
    {#if i > 0}<span class="plus">+</span>{/if}
    <kbd>{cap}</kbd>
  {/each}
</span>

<style>
  .keys {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--muted);
    vertical-align: middle;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 1px 7px 0;
    border-radius: 5px;
    background: var(--lav);
    color: var(--lav-fg);
    font: inherit;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1;
  }
</style>
