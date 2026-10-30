<script lang="ts">
// Display preview card (spec §9, §11.5, §11.6): the reduced display with
// the zone frame drawn by the UI, a placeholder when off or failed, and the
// Hide / Show button. No "Live" label.
import type { Rect } from "../lib/generated/Rect";
import { strings } from "../lib/strings";
import Button from "./Button.svelte";

type Props = {
  /** data: URL of the latest frame, or null for the placeholder. */
  src: string | null;
  displayW?: number;
  displayH?: number;
  zone?: Rect | null;
  on: boolean;
  ontoggle?: (on: boolean) => void;
};

let { src, displayW = 16, displayH = 9, zone = null, on, ontoggle }: Props = $props();

const frame = $derived(
  zone && displayW > 0 && displayH > 0
    ? {
        left: `${(zone.x / displayW) * 100}%`,
        top: `${(zone.y / displayH) * 100}%`,
        width: `${(zone.w / displayW) * 100}%`,
        height: `${(zone.h / displayH) * 100}%`,
      }
    : null,
);
</script>

<div class="preview card">
  <div class="screen" style:aspect-ratio="{displayW} / {displayH}">
    {#if src && on}
      <img {src} alt="" />
      {#if frame}
        <span
          class="frame"
          style:left={frame.left}
          style:top={frame.top}
          style:width={frame.width}
          style:height={frame.height}
        ></span>
      {/if}
    {:else}
      <span class="placeholder" aria-hidden="true"></span>
    {/if}
  </div>
  <div class="foot">
    <Button size="sm" variant="ghost" onclick={() => ontoggle?.(!on)}>
      {on ? strings.session.preview.hide : strings.session.preview.show}
    </Button>
  </div>
</div>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    width: 100%;
  }

  .screen {
    position: relative;
    width: 100%;
    border-radius: 8px;
    overflow: hidden;
    background: #1b1740;
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .frame {
    position: absolute;
    box-sizing: border-box;
    border: 2px solid #a596ea;
    border-radius: 2px;
  }

  .placeholder {
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
      135deg,
      var(--lav) 0 12px,
      transparent 12px 24px
    );
    opacity: 0.35;
  }

  .foot {
    display: flex;
    justify-content: flex-end;
  }
</style>
