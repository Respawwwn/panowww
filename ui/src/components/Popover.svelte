<script lang="ts">
// Popover (spec §11.10): anchored to its parent (which must be positioned),
// closes on a click elsewhere or Esc. Shared by Nothing moved? and the
// failed-row `?`.
import type { Snippet } from "svelte";

type Props = {
  open: boolean;
  onclose: () => void;
  align?: "left" | "right";
  children?: Snippet;
};

let { open, onclose, align = "left", children }: Props = $props();

let element = $state<HTMLDivElement | null>(null);

function onPointerDown(event: PointerEvent) {
  if (element && !element.contains(event.target as Node)) {
    onclose();
  }
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    onclose();
  }
}
</script>

<svelte:window onpointerdown={open ? onPointerDown : undefined} onkeydown={open ? onKeyDown : undefined} />

{#if open}
  <div class="popover {align}" role="dialog" bind:this={element}>
    {@render children?.()}
  </div>
{/if}

<style>
  .popover {
    position: absolute;
    top: calc(100% + 6px);
    z-index: 5;
    width: 300px;
    padding: 10px 12px;
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: 10px;
    box-shadow: var(--pop);
    font-size: 12px;
    text-align: left;
    color: var(--fg);
  }

  .left {
    left: 0;
  }

  .right {
    right: 0;
  }
</style>
