<script lang="ts">
// The running view (spec §11.6, board B-SessionRunning): the state chip,
// the Game's name and *Stop* in the toolbar; the head card with Row r of
// R, the Shots count, the remaining time, the progress bar and the hint;
// the Display preview beside it; the Shots card filling live; the two
// shortcut hints bottom right. A Test or Step once shows the minimal state.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Icon from "../components/Icon.svelte";
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
      return strings.session.running.title;
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

/** The Game's Sessions folder; the Session's own folder is known at the end. */
const folder = $derived(`${settings.value.sessionsFolder.replace(/\/$/, "")}/${game.name}/`);
</script>

<div class="toolbar" data-tauri-drag-region>
  <div class="who">
    <Chip tone="lav">
      <span class="dot"></span>
      {current ? toolName(current.kind) : strings.session.running.title}
    </Chip>
    <span class="name">{game.name}</span>
  </div>
  <Button variant="danger" size="lg" onclick={() => run.stop()}>
    <Icon name="stop" size={13} />
    {strings.session.running.stop}
  </Button>
</div>

<div class="running">
  <div class="top">
    <section class="card head">
      {#if isSession}
        <div class="line">
          <div class="row">
            {interpolate(strings.session.running.rowOnly, { row: current?.row ?? 1 })}
            <span class="of">{interpolate(strings.session.running.rowOf, { rows: game.rows })}</span>
          </div>
          <div class="count">{interpolate(strings.session.running.shots, { taken, expected })}</div>
          {#if remaining}
            <div class="sub remaining">{interpolate(strings.session.running.remaining, { duration: remaining })}</div>
          {/if}
        </div>
        <ProgressBar value={taken} max={expected} />
        <p class="sub"><RichLine text={strings.session.running.hint} /></p>
        {#if current?.screenUnchanged}
          <p class="identical">{strings.session.running.identical}</p>
        {/if}
      {:else}
        <div class="line">
          <div class="row">{current ? toolName(current.kind) : ""}</div>
        </div>
        <p class="sub"><RichLine text={strings.test.running.hint} /></p>
      {/if}
    </section>
    <div class="preview">
      <PreviewCard
        src={current?.preview?.image ?? null}
        displayW={current?.preview?.displayW ?? 16}
        displayH={current?.preview?.displayH ?? 9}
        zone={current?.preview?.zone ?? null}
        on={previewOn}
        ontoggle={togglePreview}
      />
    </div>
  </div>

  {#if isSession}
    <section class="card shots">
      <div class="shead">
        <b>{strings.session.end.shotsLabel}</b>
        <span class="sub">
          {strings.session.running.live} · {interpolate(strings.session.end.grid, { rows: game.rows, shotsPerRow: game.shotsPerRow })} ·
          <code>{folder}</code>
        </span>
      </div>
      <div class="thumbs" style:grid-template-columns="repeat({game.shotsPerRow}, minmax(0, 1fr))">
        {#each cells as cell (cell.label)}
          <Thumbnail label={cell.label} src={cell.src} state={cell.state} />
        {/each}
      </div>
    </section>
  {/if}

  <div class="hints">
    <p class="t-tip"><RichLine text={strings.setup.start.shortcutStart} /></p>
    <p class="t-tip"><RichLine text={strings.setup.start.shortcutStop} /></p>
  </div>
</div>

<style>
  .toolbar {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .toolbar :global(.btn.danger) {
    background: var(--bad);
    border-color: var(--bad);
    color: #fff;
    font-weight: 600;
  }

  .who {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
  }

  .name {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .running {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .top {
    display: flex;
    gap: 16px;
    align-items: stretch;
  }

  .head {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 18px;
  }

  .preview {
    width: 300px;
    flex: none;
  }

  .line {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }

  .row {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }

  .of {
    color: var(--muted);
    font-weight: 500;
    font-size: 16px;
  }

  .count {
    font-size: 15px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .remaining {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }

  p {
    margin: 0;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  .identical {
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--warn-soft);
    color: var(--warn-fg);
    font-size: 12.5px;
  }

  .shots {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    min-width: 0;
  }

  .shead {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .shead b {
    font-weight: 600;
    font-size: 14px;
  }

  .shead .sub {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--lav);
    color: var(--lav-fg);
  }

  .thumbs {
    display: grid;
    gap: 6px;
  }

  .hints {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }
</style>
