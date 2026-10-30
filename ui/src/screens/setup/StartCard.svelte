<script lang="ts">
// Card 3 · Start (spec §11.2): the purple block with the estimate line,
// the Sessions folder, *Start Session*; under it the two shortcut lines,
// or the reason the Start is disabled.
import Button from "../../components/Button.svelte";
import RichLine from "../../components/RichLine.svelte";
import type { GameEditor } from "../../lib/editor.svelte";
import type { DisabledReason } from "../../lib/game";
import { interpolate, strings } from "../../lib/strings";

type Props = {
  editor: GameEditor;
  sessionsFolder: string;
  disabled: DisabledReason | null;
  onstart: () => void;
};

let { editor, sessionsFolder, disabled, onstart }: Props = $props();

const line = $derived(
  editor.estimate
    ? interpolate(strings.setup.start.estimate, {
        duration: editor.estimate.duration,
        shots: editor.estimate.shots,
        rows: editor.game.rows,
        shotsPerRow: editor.game.shotsPerRow,
      })
    : "",
);
</script>

<section class="start">
  <div class="text">
    <p class="line">{line}</p>
    <p class="folder" title={sessionsFolder}>{sessionsFolder}</p>
  </div>
  <Button variant="primary" size="lg" disabled={disabled !== null} onclick={onstart}>
    {strings.detail.startSession}
  </Button>
</section>
<div class="under">
  {#if disabled}
    <p class="t-tip">{strings.setup.start.disabled[disabled]}</p>
  {:else}
    <p class="t-tip"><RichLine text={strings.setup.start.shortcutStart} /></p>
    <p class="t-tip"><RichLine text={strings.setup.start.shortcutStop} /></p>
  {/if}
</div>

<style>
  .start {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 18px;
    border-radius: var(--r);
    background: var(--startcard);
    color: var(--startcard-fg);
  }

  .start :global(.btn.primary) {
    background: #fff;
    color: #4a3c96;
  }

  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .line {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }

  .folder {
    margin: 0;
    font-size: 12px;
    color: var(--startcard-sub);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .under {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    padding: 8px 4px 0;
  }

  .under p {
    margin: 0;
  }
</style>
