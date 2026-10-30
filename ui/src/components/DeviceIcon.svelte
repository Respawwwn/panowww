<script lang="ts">
// Minimalist stroke icons of the three Input devices (spec §11.1).
import type { Device } from "../lib/generated/Device";

type Props = { device: Device; size?: number };

let { device, size = 17 }: Props = $props();
</script>

<svg class="ic" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  {#if device === "keyboard"}
    <rect x="2.5" y="6" width="19" height="12" rx="2" />
    <path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M6 14h.01M18 14h.01M9 14h6" />
  {:else if device === "mouse"}
    <rect x="7" y="3" width="10" height="18" rx="5" />
    <path d="M12 3v6M7 9h10" />
  {:else}
    <path d="M6.5 8h11a4 4 0 0 1 4 4.5l-.7 4a2.5 2.5 0 0 1-4.3 1.2L15 16H9l-1.5 1.7A2.5 2.5 0 0 1 3.2 16.5l-.7-4A4 4 0 0 1 6.5 8z" />
    <path d="M8 12h3M9.5 10.5v3M15.5 11.5h.01M17.5 13h.01" />
  {/if}
</svg>

<style>
  .ic {
    stroke: currentColor;
    fill: none;
    stroke-width: 1.75;
    stroke-linecap: round;
    stroke-linejoin: round;
    flex: none;
  }
</style>
