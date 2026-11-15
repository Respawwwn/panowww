<script lang="ts">
// The Options card of the Stitch panel (spec §16.3), under the stages,
// closed by default:
// anti-ghost, the Shots without control points, the JPEG quality as a
// slider of whole percents with three presets that only move it. Greyed
// out while a Stitch runs; the Stitch starts from the panel's toolbar.
import CollapsibleCard from "../../components/CollapsibleCard.svelte";
import Segmented from "../../components/Segmented.svelte";
import Switch from "../../components/Switch.svelte";
import Tip from "../../components/Tip.svelte";
import type { StitchOptions } from "../../lib/generated/StitchOptions";
import type { UnalignedShots } from "../../lib/generated/UnalignedShots";
import { clampQuality, MAX_QUALITY, MIN_QUALITY, QUALITY_PRESETS } from "../../lib/stitchOptions";
import { interpolate, strings } from "../../lib/strings";

type Props = {
  options: StitchOptions;
  /** A Stitch of this Session runs: the choices are locked. */
  running: boolean;
  /** Every change, the slider's too while it moves. */
  onchange: (options: StitchOptions) => void;
  /** A change to keep: the slider calls it once released. */
  oncommit: (options: StitchOptions) => void;
};

let { options, running, onchange, oncommit }: Props = $props();

const t = strings.stitch.options;

const unalignedItems: { id: UnalignedShots; label: string }[] = [
  { id: "placeFromGrid", label: t.placeFromGrid },
  { id: "leaveOut", label: t.leaveOut },
];
const presetItems = QUALITY_PRESETS.map((q) => ({
  id: String(q),
  label: interpolate(t.percent, { n: q }),
}));

let open = $state(false);

function set(next: Partial<StitchOptions>) {
  const changed = { ...options, ...next };
  onchange(changed);
  oncommit(changed);
}

function slide(value: number, keep: boolean) {
  const changed = { ...options, jpegQuality: clampQuality(value) };
  onchange(changed);
  if (keep) {
    oncommit(changed);
  }
}
</script>

<CollapsibleCard title={t.title} {open} ontoggle={() => (open = !open)}>
  <div class="options" class:locked={running}>

  <div class="opt">
    <Switch
      checked={options.antiGhost}
      label={t.antiGhost}
      disabled={running}
      onchange={(on) => set({ antiGhost: on })}
    />
    <Tip text={t.antiGhostHint} />
  </div>

  <div class="opt">
    <span class="label">{t.unaligned}</span>
    <Segmented
      items={unalignedItems}
      value={options.unalignedShots}
      disabled={running}
      onchange={(id) => set({ unalignedShots: id as UnalignedShots })}
    />
    {#if options.unalignedShots === "leaveOut"}
      <Tip text={t.leaveOutHint} />
    {/if}
  </div>

  <div class="opt">
    <span class="label">{t.quality}</span>
    <Segmented
      items={presetItems}
      value={String(options.jpegQuality)}
      disabled={running}
      onchange={(id) => set({ jpegQuality: Number(id) })}
    />
    <div class="slider">
      <input
        type="range"
        min={MIN_QUALITY}
        max={MAX_QUALITY}
        step="1"
        value={options.jpegQuality}
        disabled={running}
        aria-label={t.quality}
        oninput={(e) => slide(Number(e.currentTarget.value), false)}
        onchange={(e) => slide(Number(e.currentTarget.value), true)}
      />
      <span class="value">{interpolate(t.percent, { n: options.jpegQuality })}</span>
    </div>
  </div>

  </div>
</CollapsibleCard>

<style>
  .options {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .opt {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  .locked .opt {
    opacity: 0.55;
  }

  .label {
    font-size: 12px;
    color: var(--muted);
  }

  .slider {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
  }

  input[type="range"] {
    flex: 1;
    accent-color: var(--accent);
  }

  .value {
    min-width: 44px;
    text-align: right;
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

</style>
