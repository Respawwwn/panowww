<script lang="ts">
// A global shortcut as key caps with *Record…* and *Reset* (spec §11.12):
// the recorder captures the next combo, refuses the ones §10 forbids, and
// shows the registration error when the OS refuses it.
import type { Shortcut } from "../lib/generated/Shortcut";
import { shortcutFromKeyEvent, shortcutProblem } from "../lib/recorder";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";
import Button from "./Button.svelte";
import KeyCaps from "./KeyCaps.svelte";

type Props = {
  shortcut: Shortcut;
  defaultShortcut: Shortcut;
  onchange: (shortcut: Shortcut) => void;
  /** The OS refused this combo (from the save). */
  error?: string | null;
};

let { shortcut, defaultShortcut, onchange, error = null }: Props = $props();

const platform = detectPlatform();
let recording = $state(false);
let problem = $state<string | null>(null);

function onKeyDown(event: KeyboardEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Escape") {
    recording = false;
    return;
  }
  const next = shortcutFromKeyEvent(event);
  if (!next) {
    return;
  }
  const refused = shortcutProblem(next, platform);
  if (refused) {
    problem = refused;
    return;
  }
  problem = null;
  recording = false;
  onchange(next);
}

const isDefault = $derived(
  shortcut.code === defaultShortcut.code &&
    [...shortcut.modifiers].sort().join() === [...defaultShortcut.modifiers].sort().join(),
);
</script>

<svelte:window onkeydown={recording ? onKeyDown : undefined} />

<div class="recorder">
  <div class="row">
    {#if recording}
      <span class="prompt">{strings.settings.recorder.press}</span>
    {:else}
      <KeyCaps {shortcut} {platform} />
    {/if}
    <Button size="sm" onclick={() => ((recording = !recording), (problem = null))}>
      {recording ? strings.setup.input.keyboard.cancel : strings.settings.record}
    </Button>
    <Button size="sm" variant="ghost" disabled={isDefault} onclick={() => onchange(defaultShortcut)}>
      {strings.settings.reset}
    </Button>
  </div>
  {#if problem || error}
    <p class="error">{problem ?? error}</p>
  {/if}
</div>

<style>
  .recorder {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }

  .prompt {
    font-size: 12px;
    color: var(--lav-fg);
    padding: 2px 8px;
    border-radius: 5px;
    outline: 2px solid var(--accent);
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
