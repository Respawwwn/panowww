<script lang="ts">
// The round `?` of a failed Session row (spec §11.9): purple while its
// popover is open.
type Props = {
  on?: boolean;
  label: string;
  onclick?: () => void;
};

let { on = false, label, onclick }: Props = $props();
</script>

<button type="button" class="help" class:on aria-label={label} aria-expanded={on} {onclick}>?</button>

<style>
  .help {
    width: 20px;
    height: 20px;
    padding: 1px 0 0;
    border-radius: 50%;
    border: 1px solid var(--line2);
    background: var(--btn);
    color: var(--muted);
    font-size: 11px;
    font-weight: 700;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
  }

  .help.on {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }
</style>
