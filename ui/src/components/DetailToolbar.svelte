<script lang="ts">
// Toolbar of the detail area (spec §11.1, board DirectionB): the segmented
// Setup | Sessions on the left, *Start Session* on the right (the human
// asked for it during the macOS pass, over the assembly decision).
import type { DetailTab } from "../lib/nav.svelte";
import { strings } from "../lib/strings";
import Button from "./Button.svelte";
import Icon from "./Icon.svelte";
import PillTabs from "./PillTabs.svelte";

type Props = {
  tab: DetailTab;
  onchange: (tab: DetailTab) => void;
  disabled?: boolean;
  /** The Start button; hidden when absent. */
  onstart?: () => void;
  startDisabled?: boolean;
  /** A short line before the Start button: why it is disabled. */
  note?: string | null;
};

let {
  tab,
  onchange,
  disabled = false,
  onstart,
  startDisabled = false,
  note = null,
}: Props = $props();

const items: { id: DetailTab; label: string }[] = [
  { id: "setup", label: strings.detail.setup },
  { id: "sessions", label: strings.detail.sessions },
];
</script>

<div class="toolbar" data-tauri-drag-region>
  <PillTabs {items} value={tab} {disabled} onchange={(id) => onchange(id as DetailTab)} />
  {#if onstart}
    <div class="right">
      {#if note}<span class="t-tip">{note}</span>{/if}
      <Button variant="primary" disabled={disabled || startDisabled} onclick={onstart}>
        <Icon name="play" size={13} />
        {strings.detail.startSession}
      </Button>
    </div>
  {/if}
</div>

<style>
  .toolbar {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 10px;
  }
</style>
