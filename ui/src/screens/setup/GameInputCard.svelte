<script lang="ts">
// Card 1 · Game & input (spec §11.2, §13.1): Name, Notes, the Input
// device and that device's fields.
import Checkbox from "../../components/Checkbox.svelte";
import CollapsibleCard from "../../components/CollapsibleCard.svelte";
import Field from "../../components/Field.svelte";
import KeyRecorder from "../../components/KeyRecorder.svelte";
import Segmented from "../../components/Segmented.svelte";
import Tip from "../../components/Tip.svelte";
import type { GameEditor } from "../../lib/editor.svelte";
import { inputSummary } from "../../lib/game";
import type { Device } from "../../lib/generated/Device";
import type { Platform } from "../../lib/shortcut";
import { strings } from "../../lib/strings";

type Props = {
  editor: GameEditor;
  open: boolean;
  ontoggle: () => void;
  platform: Platform;
};

let { editor, open, ontoggle, platform }: Props = $props();

const devices: { id: Device; label: string }[] = [
  { id: "keyboard", label: "Keyboard" },
  { id: "mouse", label: "Mouse" },
  { id: "gamepad", label: "Gamepad" },
];

function number(value: string): number {
  const n = Number(value);
  return Number.isFinite(n) ? n : 0;
}
</script>

<CollapsibleCard n={1} title={strings.setup.game.title} summary={inputSummary(editor.game.input)} {open} {ontoggle}>
  <Field label={strings.setup.game.name} id="game-name" error={editor.error("name")}>
    <input
      id="game-name"
      class="wide"
      value={editor.game.name}
      oninput={(e) => editor.update({ name: e.currentTarget.value })}
      onblur={() => editor.flush()}
    />
  </Field>
  <Field label={strings.setup.game.notes} id="game-notes" error={editor.error("notes")}>
    <textarea
      id="game-notes"
      class="wide"
      placeholder={strings.setup.game.notesPlaceholder}
      value={editor.game.notes ?? ""}
      oninput={(e) => editor.update({ notes: e.currentTarget.value || undefined })}
      onblur={() => editor.flush()}
    ></textarea>
  </Field>
  <Field label={strings.setup.game.device}>
    <Segmented items={devices} value={editor.game.input.device} onchange={(id) => editor.setDevice(id as Device)} />
  </Field>

  {#if editor.game.input.device === "keyboard"}
    {@const input = editor.game.input}
    <Field label={strings.setup.input.keyboard.turnRight} error={editor.error("input.turnRightKey")}>
      <KeyRecorder code={input.turnRightKey} onchange={(code) => editor.updateInput({ turnRightKey: code })} />
    </Field>
    <Field label={strings.setup.input.keyboard.lookUp} error={editor.error("input.lookUpKey")}>
      <KeyRecorder code={input.lookUpKey} onchange={(code) => editor.updateInput({ lookUpKey: code })} />
    </Field>
    <Field
      label={strings.setup.input.keyboard.lookDown}
      error={editor.error("input.lookDownKey")}
      tip={strings.setup.input.keyboard.keys}
    >
      <KeyRecorder code={input.lookDownKey} onchange={(code) => editor.updateInput({ lookDownKey: code })} />
    </Field>
    <Field label={strings.setup.input.keyboard.turnHold} id="turn-hold" error={editor.error("input.turnHoldMs")}>
      <input id="turn-hold" type="number" min="1" max="5000" value={input.turnHoldMs} oninput={(e) => editor.updateInput({ turnHoldMs: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.msPerStep}</span>
    </Field>
    <Field
      label={strings.setup.input.keyboard.pitchHold}
      id="pitch-hold"
      error={editor.error("input.pitchHoldMs")}
      tip={strings.setup.input.keyboard.hold}
    >
      <input id="pitch-hold" type="number" min="1" max="5000" value={input.pitchHoldMs} oninput={(e) => editor.updateInput({ pitchHoldMs: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.msPerStep}</span>
    </Field>
  {:else if editor.game.input.device === "mouse"}
    {@const input = editor.game.input}
    <Field label={strings.setup.input.mouse.modeLabel} tip={strings.setup.input.mouse.mode}>
      <Segmented
        items={[
          { id: "move", label: strings.setup.input.mouse.move },
          { id: "drag", label: strings.setup.input.mouse.drag },
        ]}
        value={input.mode}
        onchange={(id) => editor.setMouseMode(id as "move" | "drag")}
      />
    </Field>
    {#if input.mode === "drag"}
      <Field label={strings.setup.input.mouse.dragButton} error={editor.error("input.dragButton")}>
        <Segmented
          items={[
            { id: "left", label: strings.setup.input.mouse.left },
            { id: "right", label: strings.setup.input.mouse.right },
            { id: "middle", label: strings.setup.input.mouse.middle },
          ]}
          value={input.dragButton ?? "right"}
          onchange={(id) => editor.updateInput({ dragButton: id as "left" | "right" | "middle" })}
        />
      </Field>
    {/if}
    <Field label={strings.setup.input.mouse.turnDelta} id="turn-delta" error={editor.error("input.turnDelta")}>
      <input id="turn-delta" type="number" min="1" max="100000" value={input.turnDelta} oninput={(e) => editor.updateInput({ turnDelta: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.mouse.unit}</span>
    </Field>
    <Field
      label={strings.setup.input.mouse.pitchDelta}
      id="pitch-delta"
      error={editor.error("input.pitchDelta")}
      tip={strings.setup.input.mouse.crossOs}
    >
      <input id="pitch-delta" type="number" min="1" max="100000" value={input.pitchDelta} oninput={(e) => editor.updateInput({ pitchDelta: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.mouse.unit}</span>
    </Field>
    <Field label={strings.setup.input.vertical}>
      <Checkbox checked={input.invertVertical} label={strings.setup.input.invertVertical} onchange={(v) => editor.updateInput({ invertVertical: v })} />
    </Field>
  {:else}
    {@const input = editor.game.input}
    {#if platform === "macos"}
      <Tip kind="warn" text={strings.setup.input.gamepad.unavailableMac} />
    {/if}
    <Field
      label={strings.setup.input.gamepad.deflectionLabel}
      id="deflection"
      error={editor.error("input.stickDeflection")}
      tip={strings.setup.input.gamepad.deflection}
    >
      <input id="deflection" type="number" min="0.1" max="1" step="0.05" value={input.stickDeflection} oninput={(e) => editor.updateInput({ stickDeflection: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.gamepad.unit}</span>
    </Field>
    <Field label={strings.setup.input.keyboard.turnHold} id="g-turn-hold" error={editor.error("input.turnHoldMs")}>
      <input id="g-turn-hold" type="number" min="1" max="5000" value={input.turnHoldMs} oninput={(e) => editor.updateInput({ turnHoldMs: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.msPerStep}</span>
    </Field>
    <Field
      label={strings.setup.input.keyboard.pitchHold}
      id="g-pitch-hold"
      error={editor.error("input.pitchHoldMs")}
      tip={strings.setup.input.gamepad.hold}
    >
      <input id="g-pitch-hold" type="number" min="1" max="5000" value={input.pitchHoldMs} oninput={(e) => editor.updateInput({ pitchHoldMs: number(e.currentTarget.value) })} />
      <span class="unit">{strings.setup.input.msPerStep}</span>
    </Field>
    <Field label={strings.setup.input.vertical}>
      <Checkbox checked={input.invertVertical} label={strings.setup.input.invertVertical} onchange={(v) => editor.updateInput({ invertVertical: v })} />
    </Field>
  {/if}
</CollapsibleCard>
