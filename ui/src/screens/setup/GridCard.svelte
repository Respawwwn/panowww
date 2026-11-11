<script lang="ts">
// Card 2 · Grid & timing (spec §11.2, §13.2): Start pole, Shots per Row,
// Rows, Wait after move, Wait after Shot.
import CollapsibleCard from "../../components/CollapsibleCard.svelte";
import Field from "../../components/Field.svelte";
import Segmented from "../../components/Segmented.svelte";
import type { GameEditor } from "../../lib/editor.svelte";
import { type Flash, flashOf, gridSummary } from "../../lib/game";
import { strings } from "../../lib/strings";

type Props = {
  editor: GameEditor;
  open: boolean;
  ontoggle: () => void;
  /** The field the Calibrate card just changed, to glow. */
  flash: Flash | null;
};

let { editor, open, ontoggle, flash }: Props = $props();

function number(value: string): number {
  const n = Number(value);
  return Number.isFinite(n) ? n : 0;
}
</script>

<CollapsibleCard n={2} title={strings.setup.grid.title} summary={gridSummary(editor.game)} {open} {ontoggle}>
  <Field
    label={strings.setup.grid.startPoleLabel}
    tip={editor.game.startPole === "nadir" ? strings.setup.grid.startPole.nadir : strings.setup.grid.startPole.zenith}
  >
    <Segmented
      items={[
        { id: "nadir", label: strings.setup.grid.nadir },
        { id: "zenith", label: strings.setup.grid.zenith },
      ]}
      value={editor.game.startPole}
      onchange={(id) => editor.update({ startPole: id as "nadir" | "zenith" })}
    />
  </Field>
  <Field label={strings.setup.grid.shotsPerRowLabel} id="shots-per-row" flash={flashOf(flash, "shotsPerRow")} error={editor.error("shotsPerRow")} tip={strings.setup.grid.shotsPerRow}>
    <input id="shots-per-row" type="number" min="3" max="72" value={editor.game.shotsPerRow} oninput={(e) => editor.update({ shotsPerRow: number(e.currentTarget.value) })} />
  </Field>
  <Field label={strings.setup.grid.rowsLabel} id="rows" flash={flashOf(flash, "rows")} error={editor.error("rows")} tip={strings.setup.grid.rows}>
    <input id="rows" type="number" min="2" max="36" value={editor.game.rows} oninput={(e) => editor.update({ rows: number(e.currentTarget.value) })} />
  </Field>
  <Field label={strings.setup.grid.waitAfterMoveLabel} id="wait-move" error={editor.error("waitAfterMoveMs")} tip={strings.setup.grid.waitAfterMove}>
    <input id="wait-move" type="number" min="0" max="10000" step="50" value={editor.game.waitAfterMoveMs} oninput={(e) => editor.update({ waitAfterMoveMs: number(e.currentTarget.value) })} />
    <span class="unit">ms</span>
  </Field>
  <Field label={strings.setup.grid.waitAfterShotLabel} id="wait-shot" error={editor.error("waitAfterShotMs")} tip={strings.setup.grid.waitAfterShot}>
    <input id="wait-shot" type="number" min="0" max="10000" step="50" value={editor.game.waitAfterShotMs} oninput={(e) => editor.update({ waitAfterShotMs: number(e.currentTarget.value) })} />
    <span class="unit">ms</span>
  </Field>
</CollapsibleCard>
