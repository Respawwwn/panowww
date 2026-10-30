<script lang="ts">
// A labelled checkbox (b.css `.check`): Invert vertical, Show frame.
type Props = {
  checked: boolean;
  label: string;
  disabled?: boolean;
  onchange?: (checked: boolean) => void;
};

let { checked, label, disabled = false, onchange }: Props = $props();
</script>

<label class="check" class:disabled>
  <input type="checkbox" {checked} {disabled} onchange={(e) => onchange?.(e.currentTarget.checked)} />
  <span class="box" class:on={checked} aria-hidden="true">
    {#if checked}
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-6" /></svg>
    {/if}
  </span>
  <span>{label}</span>
</label>

<style>
  .check {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .check.disabled {
    opacity: 0.45;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .box {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    border: 1px solid var(--line2);
    background: var(--field);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
  }

  .box.on {
    background: var(--accent);
    border-color: var(--accent);
  }

  svg {
    width: 12px;
    height: 12px;
    stroke: var(--accent-fg);
    fill: none;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  input:focus-visible + .box {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
