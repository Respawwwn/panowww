<script lang="ts">
// A string of the strings module with its `[start]` / `[stop]` markers
// drawn as key caps from the current Settings.
import { settings } from "../lib/settings.svelte";
import { type InterpolationValues, interpolate, splitKeyCaps } from "../lib/strings";
import KeyCaps from "./KeyCaps.svelte";

type Props = {
  text: string;
  values?: InterpolationValues;
  class?: string;
};

let { text, values = {}, class: className = "" }: Props = $props();

const segments = $derived(splitKeyCaps(interpolate(text, values)));
</script>

<span class={className}>
  {#each segments as segment, i (i)}
    {#if segment.kind === "text"}{segment.text}{:else}<KeyCaps
        shortcut={settings.shortcut(segment.shortcut)}
      />{/if}
  {/each}
</span>
