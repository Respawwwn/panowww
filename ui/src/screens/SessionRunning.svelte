<script lang="ts">
// The running view (spec §11.6): for a Session the head card (Row r of
// R, progress, Shots, remaining time, hint, Stop), the amber identical
// line, the Display preview, the thumbnails grid filling live; for a
// Test or Step once the minimal state.
import Button from "../components/Button.svelte";
import PreviewCard from "../components/PreviewCard.svelte";
import ProgressBar from "../components/ProgressBar.svelte";
import RichLine from "../components/RichLine.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import { api } from "../lib/api";
import { formatDuration, shotLabel } from "../lib/format";
import type { Game } from "../lib/generated/Game";
import type { RunKindDto } from "../lib/generated/RunKindDto";
import { run } from "../lib/run.svelte";
import { settings } from "../lib/settings.svelte";
import { interpolate, strings } from "../lib/strings";

type Props = { game: Game };

let { game }: Props = $props();

const current = $derived(run.current);
const isSession = $derived(current?.kind.kind === "session");

function toolName(k: RunKindDto): string {
  const c = strings.setup.calibrate;
  switch (k.kind) {
    case "turnTest":
      return c.turnTestTitle;
    case "pitchTest":
      return c.pitchTestTitle;
    case "stepOnce":
      return c.stepOnceTitle;
    case "checkOverlap":
      return k.axis === "turn" ? c.overlapTurnTitle : c.overlapPitchTitle;
    case "session":
      return strings.detail.startSession;
  }
}

/** Remaining time from the measured per-Shot average (spec §7.6). */
const remaining = $derived.by(() => {
  if (!current || current.shots.length < 2 || !current.startedAt) {
    return null;
  }
  const elapsed = Date.now() - current.startedAt;
  const perShot = elapsed / current.shots.length;
  return formatDuration(perShot * Math.max(0, current.expected - current.shots.length));
});

const taken = $derived(current?.shots.length ?? 0);
const expected = $derived(current?.expected ?? 0);

/** The grid cells in Row-major order with their state. */
const cells = $derived.by(() => {
  const done = new Set(current?.shots.map((s) => `${s.row}-${s.col}`));
  const thumbs = new Map(current?.shots.map((s) => [`${s.row}-${s.col}`, s.thumbnail]));
  const next = current?.shots.length ?? 0;
  const list: { label: string; state: "done" | "now" | "todo"; src: string | null }[] = [];
  for (let row = 1; row <= game.rows; row += 1) {
    for (let col = 1; col <= game.shotsPerRow; col += 1) {
      const key = `${row}-${col}`;
      const index = (row - 1) * game.shotsPerRow + col - 1;
      list.push({
        label: shotLabel(row, col),
        state: done.has(key) ? "done" : index === next ? "now" : "todo",
        src: thumbs.get(key) ?? null,
      });
    }
  }
  return list;
});

const previewOn = $derived(settings.value.displayPreview);

async function togglePreview(on: boolean) {
  settings.value = { ...settings.value, displayPreview: on };
  await api.run.setPreview(on).catch(() => {});
}
</script>

<div class="running">
  <div class="main">
    <section class="card head">
      <div class="titles">
        <p class="t-overline">{game.name}</p>
        {#if isSession}
          <h1 class="t-window">{interpolate(strings.session.running.row, { row: current?.row ?? 1, rows: game.rows })}</h1>
        {:else if current}
          <h1 class="t-window">{toolName(current.kind)}</h1>
        {/if}
      </div>
      {#if isSession}
        <ProgressBar value={taken} max={expected} />
        <div class="counts">
          <span class="t-counter">{interpolate(strings.session.running.shots, { taken, expected })}</span>
          {#if remaining}
            <span class="t-label">{interpolate(strings.session.running.remaining, { duration: remaining })}</span>
          {/if}
        </div>
        <p class="t-tip"><RichLine text={strings.session.running.hint} /></p>
        {#if current?.screenUnchanged}
          <p class="identical">{strings.session.running.identical}</p>
        {/if}
      {:else}
        <p class="t-tip"><RichLine text={strings.test.running.hint} /></p>
      {/if}
      <div class="actions">
        <Button variant="danger" size="lg" onclick={() => run.stop()}>{strings.session.running.stop}</Button>
      </div>
    </section>

    {#if isSession}
      <div class="thumbs" style:grid-template-columns="repeat({game.shotsPerRow}, minmax(0, 1fr))">
        {#each cells as cell (cell.label)}
          <Thumbnail label={cell.label} src={cell.src} state={cell.state} />
        {/each}
      </div>
      <p class="t-tip folder">{settings.value.sessionsFolder}</p>
    {/if}
  </div>
  <aside class="side">
    <PreviewCard
      src={current?.preview?.image ?? null}
      displayW={current?.preview?.displayW ?? 16}
      displayH={current?.preview?.displayH ?? 9}
      zone={current?.preview?.zone ?? null}
      on={previewOn}
      ontoggle={togglePreview}
    />
  </aside>
</div>

<style>
  .running {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(280px, 2fr);
    gap: 20px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .main {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }

  .head {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  .titles p,
  .titles h1,
  .head p {
    margin: 0;
  }

  .counts {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .identical {
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--warn-soft);
    color: var(--warn-fg);
    font-size: 12.5px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }

  .thumbs {
    display: grid;
    gap: 6px;
  }

  .folder {
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .side {
    display: flex;
    align-items: flex-start;
  }
</style>
