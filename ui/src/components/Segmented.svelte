<script lang="ts">
// Segmented control (spec §11.2): Input device, Move / Drag, Start pole.
// The selected segment is a white card on the lavender track.
type Item<T extends string> = { id: T; label: string };

type Props<T extends string> = {
  items: Item<T>[];
  value: T;
  onchange?: (id: T) => void;
  size?: "md" | "lg";
  disabled?: boolean;
};

let { items, value, onchange, size = "md", disabled = false }: Props<string> = $props();
</script>

<div class="seg {size}" role="radiogroup">
  {#each items as item (item.id)}
    <button
      type="button"
      role="radio"
      class:on={item.id === value}
      aria-checked={item.id === value}
      {disabled}
      onclick={() => onchange?.(item.id)}
    >
      {item.label}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    background: var(--lav);
    border-radius: 999px;
    padding: 2px;
  }

  button {
    height: 24px;
    padding: 1px 12px 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    color: var(--lav-fg);
    line-height: 1;
  }

  .lg button {
    height: 28px;
    padding-inline: 16px;
    font-size: 13px;
  }

  button.on {
    background: var(--card);
    color: var(--fg);
    box-shadow: var(--shadow);
  }

  button:disabled {
    opacity: 0.45;
  }
</style>
