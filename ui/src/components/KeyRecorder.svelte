<script lang="ts">
// One physical key of a keyboard Game as a key cap with *Change…*: a
// recorder that captures the next physical key (spec §11.2).
import { keyLabel } from "../lib/shortcut";
import { strings } from "../lib/strings";
import Button from "./Button.svelte";

type Props = {
  code: string;
  onchange: (code: string) => void;
  disabled?: boolean;
};

let { code, onchange, disabled = false }: Props = $props();

let recording = $state(false);

function onKeyDown(event: KeyboardEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Escape") {
    recording = false;
    return;
  }
  // Modifiers alone keep the recorder open; the validator refuses them
  // anyway, so only a real key ends it.
  if (
    [
      "ShiftLeft",
      "ShiftRight",
      "ControlLeft",
      "ControlRight",
      "AltLeft",
      "AltRight",
      "MetaLeft",
      "MetaRight",
    ].includes(event.code)
  ) {
    return;
  }
  recording = false;
  onchange(event.code);
}
</script>

<svelte:window onkeydown={recording ? onKeyDown : undefined} />

<span class="recorder">
  <kbd class:recording>{recording ? strings.setup.input.keyboard.pressKey : keyLabel(code)}</kbd>
  <Button size="sm" {disabled} onclick={() => (recording = !recording)}>
    {recording ? strings.setup.input.keyboard.cancel : strings.setup.input.keyboard.change}
  </Button>
</span>

<style>
  .recorder {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    height: 24px;
    min-width: 32px;
    padding: 1px 9px 0;
    border-radius: 5px;
    background: var(--lav);
    color: var(--lav-fg);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
  }

  kbd.recording {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    font-weight: 500;
  }
</style>
