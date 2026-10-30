<script lang="ts">
// A labelled switch (spec §11.12): Display preview, Check for updates.
type Props = {
  checked: boolean;
  label: string;
  disabled?: boolean;
  onchange?: (checked: boolean) => void;
};

let { checked, label, disabled = false, onchange }: Props = $props();
</script>

<label class="switch" class:disabled>
  <input type="checkbox" role="switch" {checked} {disabled} onchange={(e) => onchange?.(e.currentTarget.checked)} />
  <span class="track" aria-hidden="true"><span class="knob"></span></span>
  <span class="text">{label}</span>
</label>

<style>
  .switch {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }

  .switch.disabled {
    opacity: 0.45;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .track {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: 999px;
    background: var(--line2);
    transition: background 0.15s ease-out;
    flex: none;
  }

  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgb(0 0 0 / 20%);
    transition: transform 0.15s ease-out;
  }

  input:checked + .track {
    background: var(--accent);
  }

  input:checked + .track .knob {
    transform: translateX(14px);
  }

  input:focus-visible + .track {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
