<script lang="ts">
// Card 3 · Start (spec §11.2, board DirectionB): the purple block with the
// estimate big, the configuration line under it, *Start Session* with its
// play icon top right, the Sessions folder path on one line.
import Button from "../../components/Button.svelte";
import Icon from "../../components/Icon.svelte";
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

const big = $derived(
  editor.estimate
    ? interpolate(strings.setup.start.estimate, { duration: editor.estimate.duration })
    : "",
);
const detail = $derived(
  editor.estimate
    ? interpolate(strings.setup.start.estimateDetail, {
        shots: editor.estimate.shots,
        rows: editor.game.rows,
        shotsPerRow: editor.game.shotsPerRow,
        pole: (editor.game.startPole === "nadir"
          ? strings.setup.grid.nadir
          : strings.setup.grid.zenith
        ).toLowerCase(),
        move: editor.game.waitAfterMoveMs,
        shot: editor.game.waitAfterShotMs,
      })
    : "",
);
const folder = $derived(`${sessionsFolder.replace(/\/$/, "")}/${editor.game.name}/`);
</script>

<section class="start">
  <div class="top">
    <div class="est">
      <p class="big">{big}</p>
      <p class="sub">{detail}</p>
    </div>
    <Button variant="primary" size="lg" disabled={disabled !== null} onclick={onstart}>
      <Icon name="play" size={13} />
      {strings.detail.startSession}
    </Button>
  </div>
  <p class="path" title={folder}>{folder}</p>
</section>

<style>
  .start {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 18px;
    border-radius: var(--r);
    background: var(--startcard);
    color: var(--startcard-fg);
  }

  .start :global(.btn.primary) {
    background: #fff;
    color: #4a3c96;
  }

  .top {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }

  .est {
    flex: 1;
    min-width: 0;
  }

  p {
    margin: 0;
  }

  .big {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .sub {
    margin-top: 2px;
    font-size: 12px;
    color: var(--startcard-sub);
  }

  .path {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11.5px;
    color: var(--startcard-sub);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
