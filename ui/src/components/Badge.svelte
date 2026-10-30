<script lang="ts">
// Numbered lavender badge (spec §12): the step numbers of the Setup cards
// and the Calibrate tools.
type Props = { n: number | string };

let { n }: Props = $props();
</script>

<span class="badge">{n}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--lav);
    color: var(--lav-fg);
    font-size: 12px;
    font-weight: 700;
    flex: none;
  }
</style>
