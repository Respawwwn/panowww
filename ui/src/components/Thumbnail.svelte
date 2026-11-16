<script lang="ts">
// One cell of the Shots grid (spec §11.6, §16.3): a real 16:9 thumbnail
// with its `rNN_cNN` overlaid, purple outline on the next cell, dashed when
// pending; for the Stitch, dimmed while waiting, amber when placed from
// the grid, red when it found no control points.
type Props = {
  label: string;
  src?: string | null;
  state?: "done" | "now" | "todo" | "dim" | "warn" | "bad";
};

let { label, src = null, state = "done" }: Props = $props();
</script>

<div class="th {state}">
  {#if src}
    <img {src} alt={label} />
  {/if}
  <span>{label}</span>
</div>

<style>
  .th {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: 6px;
    overflow: hidden;
    background: var(--lav);
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  span {
    position: absolute;
    left: 4px;
    bottom: 3px;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.02em;
    color: #fff;
    text-shadow: 0 1px 2px rgb(0 0 0 / 70%);
  }

  .now {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .now span,
  .todo span {
    color: var(--lav-fg);
    text-shadow: none;
  }

  .todo {
    background: transparent;
    border: 1px dashed var(--line2);
  }

  .todo span {
    color: var(--faint);
  }

  .dim img {
    opacity: 0.22;
  }

  .dim span {
    color: var(--faint);
    text-shadow: none;
  }

  .warn,
  .bad {
    outline: 2px solid var(--warn);
    outline-offset: 1px;
  }

  .bad {
    outline-color: var(--bad);
  }

  .warn img,
  .bad img {
    opacity: 0.35;
  }
</style>
