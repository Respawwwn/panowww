<script lang="ts">
// Thin lavender bar filling purple (spec §11.6).
type Props = { value: number; max?: number };

let { value, max = 1 }: Props = $props();

const share = $derived(max > 0 ? Math.min(1, Math.max(0, value / max)) : 0);
</script>

<div class="bar" role="progressbar" aria-valuenow={value} aria-valuemin={0} aria-valuemax={max}>
  <i style:width="{share * 100}%"></i>
</div>

<style>
  .bar {
    height: 8px;
    border-radius: 999px;
    background: var(--lav);
    overflow: hidden;
  }

  i {
    display: block;
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
    transition: width 0.2s ease-out;
  }
</style>
