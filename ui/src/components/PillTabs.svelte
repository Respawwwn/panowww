<script lang="ts">
// Pill tabs (spec §12): lavender track, the selected pill purple. Used for
// Setup | Sessions in the detail toolbar and Turn | Pitch in Check overlap.
type Item<T extends string> = { id: T; label: string };

type Props<T extends string> = {
  items: Item<T>[];
  value: T;
  onchange?: (id: T) => void;
  disabled?: boolean;
};

let { items, value, onchange, disabled = false }: Props<string> = $props();
</script>

<div class="tabs" role="tablist">
  {#each items as item (item.id)}
    <button
      type="button"
      role="tab"
      class:on={item.id === value}
      aria-selected={item.id === value}
      {disabled}
      onclick={() => onchange?.(item.id)}
    >
      {item.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: inline-flex;
    gap: 4px;
    background: var(--lav);
    border-radius: 999px;
    padding: 3px;
  }

  button {
    height: 28px;
    padding: 1px 16px 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    font-size: 13px;
    font-weight: 500;
    color: var(--lav-fg);
    line-height: 1;
  }

  button.on {
    background: var(--accent);
    color: var(--accent-fg);
  }

  button:disabled {
    opacity: 0.45;
  }
</style>
