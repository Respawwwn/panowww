<script lang="ts">
// A secondary window over the app (b.css `.win`): title bar, body,
// optional footer. Esc or the scrim closes it. The title bar casts a soft
// shadow once the body is scrolled, as content slides under it.
import type { Snippet } from "svelte";

type Props = {
  title: string;
  width?: number;
  /** A fixed height, for a window laid out top to bottom (About). */
  height?: number;
  onclose: () => void;
  children?: Snippet;
  footer?: Snippet;
  /** Something beside the title, such as a count. */
  aside?: Snippet;
};

let { title, width = 640, height, onclose, children, footer, aside }: Props = $props();

let scrolled = $state(false);

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    onclose();
  }
}
</script>

<svelte:window onkeydown={onKeyDown} />

<div class="scrim" role="presentation" onpointerdown={(e) => e.target === e.currentTarget && onclose()}>
  <div
    class="win"
    role="dialog"
    aria-label={title}
    style:width="min({width}px, 94vw)"
    style:height={height ? `min(${height}px, 92vh)` : undefined}
  >
    <header class="win-t" class:scrolled>
      <h2>{title}</h2>
      {@render aside?.()}
      <button type="button" class="close" aria-label="Close" onclick={onclose}>✕</button>
    </header>
    <div class="win-b" onscroll={(e) => (scrolled = e.currentTarget.scrollTop > 0)}>
      {@render children?.()}
    </div>
    {#if footer}
      <footer class="win-f">{@render footer()}</footer>
    {/if}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--dim);
  }

  .win {
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    background: var(--content);
    border-radius: var(--r);
    box-shadow: var(--pop);
    overflow: hidden;
  }

  .win-t {
    height: 44px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 14px;
    background: var(--sidebar);
    flex: none;
    position: relative;
    z-index: 1;
    transition: box-shadow 0.15s ease;
  }

  .win-t.scrolled {
    box-shadow: var(--scroll-shadow);
  }

  h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    flex: 1;
  }

  .close {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
    padding: 0;
    border-radius: 6px;
  }

  .close:hover {
    background: var(--hover);
  }

  .win-b {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .win-f {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--line);
    flex: none;
  }
</style>
