<script lang="ts">
// The Nothing moved? popover (spec §11.10): title, intro, the five items
// for this device and OS. Shared by the Calibrate card and the failed
// Session row.
import type { Device } from "../lib/generated/Device";
import { nothingMovedItems } from "../lib/nothingMoved";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";
import Popover from "./Popover.svelte";

type Props = {
  open: boolean;
  onclose: () => void;
  device: Device;
  align?: "left" | "right";
};

let { open, onclose, device, align = "left" }: Props = $props();

const items = $derived(nothingMovedItems(device, detectPlatform()));
</script>

<Popover {open} {onclose} {align}>
  <b>{strings.nothingMoved.title}</b>
  <p>{strings.nothingMoved.intro}</p>
  <ol>
    {#each items as item, i (i)}
      <li>{item}</li>
    {/each}
  </ol>
</Popover>

<style>
  b {
    display: block;
    margin-bottom: 2px;
    font-weight: 600;
  }

  p {
    margin: 0 0 6px;
    color: var(--muted);
  }

  ol {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
