<script lang="ts">
// The Stitch panel (spec §16.3, boards StitchIdle, StitchRunning,
// StitchWarnings, StitchFailed, StitchFailedNoStitcher): under the
// Session header with *Back to Session*, the top card with the title, the
// chips and the actions of the state; on the left the six stages with
// their times, the card *Measured from the Shots*, the green file box or
// the running tips or the failed hints; on the right the Shots grid with
// the Row being aligned framed, pending Shots dimmed, amber for Shots
// placed from the grid, red for Shots without control points. The state
// comes from the stitch store (the Stitch may already be running or have
// ended when the panel opens); the Panorama on disk counts as finished.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Icon from "../components/Icon.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import ProgressBar from "../components/ProgressBar.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import Tip from "../components/Tip.svelte";
import { api, isAppError, type Stitcher } from "../lib/api";
import { formatBytes, formatSeconds, formatWhen, shotLabel } from "../lib/format";
import { nav } from "../lib/nav.svelte";
import { run } from "../lib/run.svelte";
import { stateTone } from "../lib/sessionText";
import { fetchSessionView, type SessionView } from "../lib/sessionView";
import { settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
import {
  percent,
  remainingMs,
  STAGES,
  type StageName,
  stageMs,
  stageState,
  stitch,
} from "../lib/stitch.svelte";
import {
  type CellStates,
  cellState,
  deg,
  expectedSize,
  fileStem,
  gridCaption,
  measuredOfEntry,
  measuredRows,
  nameList,
  panelMode,
  refLabel,
} from "../lib/stitchView";
import { interpolate, splitPlaceholders, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

type Props = { gameId: string; folder: string };

let { gameId, folder }: Props = $props();

const platform = detectPlatform();
const s = strings.stitch;

let view = $state<SessionView | null>(null);
let loadError = $state<string | null>(null);
let actionError = $state<string | null>(null);
let zipped = $state<string | null>(null);
let replacing = $state(false);

/** History thumbnails, generated on demand by file name. */
let thumbs = $state<Record<string, string>>({});
/** The clock, ticking while the Stitch runs, for the elapsed times. */
let now = $state(Date.now());

$effect(() => {
  const id = gameId;
  const f = folder;
  view = null;
  loadError = null;
  fetchSessionView(id, f)
    .then((found) => {
      if (found) {
        view = found;
      } else {
        loadError = strings.sessions.gone;
      }
    })
    .catch((e) => {
      loadError = isAppError(e) ? e.message : String(e);
    });
});

$effect(() => {
  const f = folder;
  const shots = view?.shots ?? [];
  if (shots.length === 0) {
    return;
  }
  let cancelled = false;
  thumbs = {};
  (async () => {
    for (const shot of shots) {
      if (cancelled) {
        return;
      }
      try {
        const src = await api.sessions.thumbnail(f, shot.file);
        thumbs = { ...thumbs, [shot.file]: src };
      } catch {
        // A missing file stays a plain cell.
      }
    }
  })();
  return () => {
    cancelled = true;
  };
});

const running = $derived(stitch.isStitching(folder) ? stitch.running : null);
const last = $derived(stitch.lastFor(folder));
const entry = $derived(stitch.entryFor(folder) ?? view?.panorama ?? null);
const mode = $derived(panelMode(running, last, entry));
const failure = $derived(last?.closing.event === "failed" ? last.closing.error : null);
const cancelledBy = $derived(last?.closing.event === "cancelled" ? last.closing.reason : null);

$effect(() => {
  if (mode !== "running") {
    return;
  }
  now = Date.now();
  const timer = setInterval(() => {
    now = Date.now();
  }, 250);
  return () => clearInterval(timer);
});

const game = $derived(view?.game ?? null);
const rows = $derived(game?.rows ?? 0);
const shotsPerRow = $derived(game?.shotsPerRow ?? 0);
const total = $derived(rows * shotsPerRow);
const progress = $derived(running?.progress ?? null);
const pct = $derived(percent(progress));
const elapsedMs = $derived(running ? now - running.startedAt : 0);
const leftMs = $derived(running ? remainingMs(elapsedMs, progress?.overall ?? 0) : null);

/** The Stitchers set in Settings, in display order. */
const stitchers = $derived(
  (["ptgui", "autopanoGiga"] as const).filter((st) => settings.value.stitchers[st] !== null),
);
/** *Stitch* waits for the Runner and for a Stitch of another Session. */
const stitchDisabled = $derived(run.busy || (stitch.running !== null && !running));

const title = $derived.by(() => {
  switch (mode) {
    case "idle":
      return interpolate(s.title.idle, { n: total });
    case "running":
      return interpolate(s.title.running, { n: total });
    case "finished":
      return interpolate(s.title.finished, {
        duration: formatSeconds(entry?.durationMs ?? (last ? last.endedAt - last.startedAt : 0)),
      });
    case "failed":
      return s.title.failed;
  }
});

const subtitle = $derived.by(() => {
  if (!view || !game) {
    return "";
  }
  const date = view.startedAt ? formatWhen(view.startedAt, strings.sessions) : "";
  // Once a Panorama exists the line names its size, not the Shots'.
  const parts = [
    mode === "finished" && entry
      ? interpolate(s.subtitleFinished, {
          date,
          rows: game.rows,
          shotsPerRow: game.shotsPerRow,
          w: entry.width,
          h: entry.height,
        })
      : interpolate(s.subtitle, {
          date,
          rows: game.rows,
          shotsPerRow: game.shotsPerRow,
          w: view.shotSize?.w ?? "?",
          h: view.shotSize?.h ?? "?",
        }),
  ];
  if (mode === "idle" && view.shotSize) {
    const size = expectedSize(view.shotSize.w);
    parts.push(interpolate(s.willWrite, { w: size.w, h: size.h }));
  }
  return parts.join(" · ");
});

/** The stage in progress, or the one that failed. */
const currentStage = $derived.by((): StageName | null => {
  if (mode === "running") {
    return progress?.stage.stage ?? "read";
  }
  if (mode === "failed") {
    return last?.progress?.stage.stage ?? "read";
  }
  return null;
});

/** The stage times: this Stitch's, or none for a Panorama found on disk. */
const spans = $derived(running?.stages ?? (last?.folder === folder ? last.stages : {}));

const stageRows = $derived(
  STAGES.map((name, i) => {
    const state =
      mode === "finished" && currentStage === null
        ? ("done" as const)
        : stageState(name, currentStage, mode === "failed");
    const stage = progress?.stage;
    const label =
      name === "align" && state === "now" && stage?.stage === "align"
        ? interpolate(s.stage.alignRow, { row: stage.row, rows: stage.rows })
        : s.stage[name];
    const ms = stageMs(spans[name], now);
    return { name, num: i + 1, state, label, time: ms === null ? "" : formatSeconds(ms) };
  }),
);

/** The card's rows: live while running, the Panorama's once written. */
const measured = $derived.by(() => {
  if (!game) {
    return null;
  }
  if (mode === "running" && running?.measured) {
    return measuredRows(running.measured, game.startPole);
  }
  if (mode === "finished" && entry) {
    return measuredRows(measuredOfEntry(entry), game.startPole);
  }
  return null;
});
const panoramaFile = $derived(entry?.file ?? "panorama.jpg");
const doneSegments = $derived(splitPlaceholders(s.done.file, ["file"]));
const replaceSegments = $derived(splitPlaceholders(s.actions.replace, ["file"]));
const unalignedSegments = $derived(splitPlaceholders(s.unaligned.box, ["list"]));
const droppedSegments = $derived(splitPlaceholders(s.dropped.box, ["pairs"]));

const unalignedRefs = $derived(
  (entry?.unaligned ?? []).map((file) => {
    const stem = fileStem(file);
    return { row: Number(stem.slice(1, 3)), col: Number(stem.slice(5, 7)) };
  }),
);
const unlinked = $derived(failure?.kind === "tooFewLinked" ? failure.unlinked : []);
const cellStates = $derived<CellStates>({ mode, progress, unaligned: unalignedRefs, unlinked });

const cells = $derived.by(() => {
  if (!view) {
    return [];
  }
  const byKey = new Map(
    view.shots.map((sh) => [`${sh.row}-${sh.col}`, sh.src ?? thumbs[sh.file] ?? null]),
  );
  const list: { label: string; src: string | null; state: ReturnType<typeof cellState> }[] = [];
  for (let row = 1; row <= rows; row += 1) {
    for (let col = 1; col <= shotsPerRow; col += 1) {
      list.push({
        label: shotLabel(row, col),
        src: byKey.get(`${row}-${col}`) ?? null,
        state: cellState(row, col, cellStates),
      });
    }
  }
  return list;
});

const caption = $derived(
  gridCaption(
    cellStates,
    rows,
    shotsPerRow,
    failure?.kind === "tooFewLinked" ? { linked: failure.linked, total: failure.total } : null,
  ),
);

const poleName = $derived(
  game?.startPole === "zenith" ? strings.setup.grid.zenith : strings.setup.grid.nadir,
);
const droppedPairs = $derived(entry?.droppedSteps ?? []);

const sep = platform === "windows" ? "\\" : "/";

function back() {
  nav.showSession(gameId, folder);
}

async function start() {
  replacing = false;
  actionError = null;
  zipped = null;
  try {
    await stitch.start(folder);
  } catch (error) {
    actionError = isAppError(error) ? error.message : String(error);
  }
}

function cancel() {
  void stitch.cancel().catch(() => {});
}

/** *Stitch again* asks only when there is a file to replace (spec §16.3). */
function askAgain() {
  if (entry) {
    replacing = true;
  } else {
    void start();
  }
}

/** Selects `panorama.jpg` in Finder or Explorer. */
function revealPanorama() {
  void api.sessions.reveal(`${folder}${sep}${entry?.file ?? "panorama.jpg"}`).catch(() => {});
}

/** The preview window comes with the next issue; the file is a click away. */
function previewPanorama() {
  revealPanorama();
}

async function zip() {
  actionError = null;
  try {
    zipped = await api.sessions.zip(folder);
  } catch (error) {
    actionError = isAppError(error) ? error.message : String(error);
  }
}

async function openIn(stitcher: Stitcher) {
  actionError = null;
  try {
    await api.sessions.openIn(folder, stitcher);
  } catch (error) {
    actionError = isAppError(error) ? error.message : String(error);
  }
}
</script>

<div class="toolbar" data-tauri-drag-region>
  <div class="who">
    {#if view}
      <Chip tone={stateTone(view.status)}>
        {#if view.status === "completed"}<Icon name="check" size={13} />{/if}
        {strings.session.end.chip[view.status]}
      </Chip>
      <span class="name">{view.game.name}</span>
    {/if}
  </div>
  <Button size="lg" onclick={back}>
    <Icon name="back" />
    {s.actions.backToSession}
  </Button>
</div>

{#if loadError}
  <div class="pane"><p class="error">{loadError}</p></div>
{:else if view && game}
  <div class="panel">
    {#snippet replaceQuestion()}
      {#each replaceSegments as segment, i (i)}
        {#if segment.kind === "slot"}<code>{panoramaFile}</code>{:else}{segment.text}{/if}
      {/each}
    {/snippet}

    {#snippet openInButtons(firstPrimary: boolean)}
      {#each stitchers as stitcher, i (stitcher)}
        <Button variant={firstPrimary && i === 0 ? "primary" : "secondary"} onclick={() => openIn(stitcher)}>
          <Icon name={stitcher === "ptgui" ? "ptgui" : "autopano"} size={14} />
          {interpolate(strings.session.end.openIn, { stitcher: strings.settings.stitcher[stitcher] })}
        </Button>
      {/each}
    {/snippet}

    <div class="actions">
    {#if mode === "idle"}
      <div class="acts">
        <Button variant="primary" disabled={stitchDisabled} onclick={start}>
          <Icon name="stitch" size={14} />
          {strings.session.end.stitch}
        </Button>
        {@render openInButtons(false)}
      </div>
    {:else if mode === "running"}
      <!-- The progress stays in the card; the row is for the buttons. -->
    {:else if mode === "finished"}
      <InlineConfirm
        confirming={replacing}
        question={interpolate(s.actions.replace, { file: panoramaFile })}
        questionContent={replaceQuestion}
        confirmLabel={strings.session.end.stitchAgain}
        cancelLabel={s.actions.cancel}
        onconfirm={start}
        oncancel={() => (replacing = false)}
      >
        <Button variant="primary" onclick={previewPanorama}>
          <Icon name="pano" size={14} />
          {strings.session.end.previewPanorama}
        </Button>
        <Button disabled={stitchDisabled} onclick={askAgain}>
          <Icon name="stitch" size={14} />
          {strings.session.end.stitchAgain}
        </Button>
        {@render openInButtons(false)}
        <Button onclick={revealPanorama}>
          <Icon name="folder" size={14} />
          {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
        </Button>
      </InlineConfirm>
    {:else}
      <div class="acts">
        {@render openInButtons(true)}
        {#if stitchers.length === 0}
          <Button variant="primary" onclick={zip}>
            <Icon name="zip" size={14} />
            {strings.session.end.zip}
          </Button>
        {/if}
        <Button disabled={stitchDisabled} onclick={start}>
          <Icon name="stitch" size={14} />
          {s.actions.tryAgain}
        </Button>
      </div>
    {/if}
    </div>

    <section class="card top">
      <div class="titles">
        <div class="titleRow">
          <h2>{title}</h2>
          {#if mode === "finished" && unalignedRefs.length > 0}
            <Chip tone="warn"><Icon name="warn" size={13} />{interpolate(s.unaligned.chip, { n: unalignedRefs.length })}</Chip>
          {/if}
          {#if mode === "finished" && droppedPairs.length > 0}
            <Chip tone="warn"><Icon name="warn" size={13} />{interpolate(s.dropped.chip, { n: droppedPairs.length })}</Chip>
          {/if}
          {#if mode === "failed"}
            <Chip tone="bad">{s.title.failedChip}</Chip>
          {/if}
        </div>
        <p class="sub">{subtitle}</p>
        {#if mode === "idle" && cancelledBy}
          <p class="sub">{cancelledBy === "sessionStarted" ? s.cancelled.session : s.cancelled.byYou}</p>
        {/if}
        {#if actionError || zipped}
          <p class="sub" class:bad={Boolean(actionError)}>{actionError ?? zipped}</p>
        {/if}
      </div>
      {#if mode === "finished" && entry}
        <div class="facts">
          <div class="fact">
            <div class="big">{formatBytes(entry.bytes)}</div>
            <div class="sub">{strings.session.end.onDisk}</div>
          </div>
        </div>
      {/if}
      {#if mode === "running"}
        <div class="progress">
          <div class="bar"><ProgressBar value={progress?.overall ?? 0} /></div>
          <span class="pct">{pct} %</span>
          <Button onclick={cancel}>{s.actions.cancel}</Button>
        </div>
      {/if}
    </section>

    <div class="columns">
      <div class="left">
        <section class="card stages">
          {#each stageRows as row (row.name)}
            <div class="stage {row.state}">
              <span class="n">
                {#if row.state === "done"}<Icon name="check" size={12} />{:else}{row.num}{/if}
              </span>
              <span class="t">{row.label}</span>
              <span class="d">{row.time}</span>
            </div>
          {/each}
        </section>

        {#if mode !== "failed"}
          <section class="card measuredCard">
            <b>{s.measured.title}</b>
            {#if measured}
              <div class="meas">
                {#each measured as [key, value] (key)}
                  <span class="k">{key}</span><span class="v">{value}</span>
                {/each}
              </div>
            {:else}
              <Tip text={s.measured.pending} />
            {/if}
          </section>
        {/if}

        {#if mode === "finished" && entry}
          <div class="okbox">
            <span class="grow">
              {#each doneSegments as segment, i (i)}
                {#if segment.kind === "slot"}<code>{entry.file}</code>{:else}{interpolate(segment.text, { duration: formatSeconds(entry.durationMs) })}{/if}
              {/each}
            </span>
            <Button size="sm" onclick={revealPanorama}>
              <Icon name="folder" size={13} />
              {strings.settings.reveal}
            </Button>
          </div>
        {:else if mode === "running"}
          <div class="tips">
            {#if leftMs !== null}
              <Tip text={interpolate(s.progress.time, { elapsed: formatSeconds(elapsedMs), left: formatSeconds(leftMs) })} />
            {/if}
            <Tip text={s.progress.cores} />
          </div>
        {:else if mode === "failed"}
          <div class="tipbox hint">
            {#if stitchers.length > 0}
              <span>{s.failed.stitcherHint}</span>
              <div class="acts">
                {#each stitchers as stitcher (stitcher)}
                  <Button size="sm" onclick={() => openIn(stitcher)}>
                    <Icon name={stitcher === "ptgui" ? "ptgui" : "autopano"} size={13} />
                    {interpolate(strings.session.end.openIn, { stitcher: strings.settings.stitcher[stitcher] })}
                  </Button>
                {/each}
              </div>
            {:else}
              <span>{s.failed.noStitcher}</span>
              <div class="acts">
                <Button size="sm" onclick={() => windows.show("settings")}>
                  <Icon name="gear" size={13} />
                  {s.failed.setOne}
                </Button>
              </div>
            {/if}
          </div>
          <Tip text={s.failed.fixGame} />
        {/if}
      </div>

      <section class="card shots">
        <div class="head">
          <b>{strings.session.end.shotsLabel}</b>
          <span class="sub caption">{caption}</span>
        </div>
        <div class="thumbs" style:grid-template-columns="repeat({shotsPerRow}, minmax(0, 1fr))">
          {#each cells as cell (cell.label)}
            <Thumbnail label={cell.label} src={cell.src} state={cell.state} />
          {/each}
        </div>
        {#if mode === "failed" && failure}
          <div class="tipbox bad">
            {#if failure.kind === "tooFewLinked"}
              {interpolate(s.failed.tooFew, { k: failure.total - failure.linked, n: failure.total })}
              {s.shots.legendFailed}
            {:else if failure.kind === "notCompleted"}
              {interpolate(s.failed.other, { message: strings.session.end.interrupted.note })}
            {:else}
              {interpolate(s.failed.other, { message: failure.message })}
            {/if}
          </div>
        {:else if mode === "finished"}
          {#if unalignedRefs.length > 0}
            <div class="tipbox warn">
              {#each unalignedSegments as segment, i (i)}
                {#if segment.kind === "slot"}{#each unalignedRefs as ref, j (refLabel(ref))}{#if j > 0}{j === unalignedRefs.length - 1 ? ` ${s.shots.and} ` : ", "}{/if}<code>{refLabel(ref)}</code>{/each}{:else}{segment.text}{/if}
              {/each}
            </div>
            <Tip text={s.shots.legendUnaligned} />
          {/if}
          {#if droppedPairs.length > 0 && entry}
            <div class="tipbox warn">
              {#each droppedSegments as segment, i (i)}
                {#if segment.kind === "slot"}{#each droppedPairs as [a, b], j (a)}{#if j > 0}{j === droppedPairs.length - 1 ? ` ${s.shots.and} ` : ", "}{/if}<code>{fileStem(a)}</code> {s.shots.and} <code>{fileStem(b)}</code>{/each}{:else}{interpolate(segment.text, { deg: deg(entry.turnStepDeg) })}{/if}
              {/each}
            </div>
          {/if}
        {:else}
          <Tip text={interpolate(s.shots.legend, { pole: poleName })} />
        {/if}
      </section>
    </div>
  </div>
{/if}

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

  .who {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .name {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pane {
    padding: 24px;
  }

  .error {
    margin: 0;
    font-size: 12.5px;
    color: var(--bad-fg);
  }

  .panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  /* The actions row sits above the card, as on the Session end. */
  /* As tall as its buttons, so the inline question of Stitch again, drawn
     with smaller buttons, does not move the cards below. */
  .actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    min-height: 28px;
  }

  .actions:empty {
    display: none;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 14px 18px;
  }

  .titles {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .facts {
    display: flex;
    gap: 16px;
  }

  .fact {
    text-align: center;
  }

  .big {
    font-size: 22px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .titleRow {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  h2,
  p {
    margin: 0;
  }

  h2 {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  .sub.bad {
    color: var(--bad-fg);
  }

  .acts {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 340px;
  }

  .bar {
    flex: 1;
  }

  .pct {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    width: 44px;
    text-align: right;
  }

  .columns {
    flex: 1;
    display: flex;
    gap: 16px;
    min-height: 0;
    align-items: flex-start;
  }

  .left {
    width: 340px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .stages {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stage {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 30px;
    font-size: 13px;
  }

  .stage .n {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
    font-size: 11px;
    font-weight: 700;
    background: var(--lav);
    color: var(--lav-fg);
  }

  .stage .t {
    flex: 1;
  }

  .stage .d {
    font-size: 11.5px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .stage.done .n {
    background: var(--ok-soft);
    color: var(--ok-fg);
  }

  .stage.now .n {
    background: var(--accent);
    color: var(--accent-fg);
  }

  .stage.now .t {
    font-weight: 600;
  }

  .stage.bad .n {
    background: var(--bad-soft);
    color: var(--bad-fg);
  }

  .stage.bad .t {
    font-weight: 600;
    color: var(--bad-fg);
  }

  .stage.todo {
    color: var(--faint);
  }

  .measuredCard {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .measuredCard b {
    font-weight: 600;
    font-size: 13px;
  }

  .meas {
    display: grid;
    grid-template-columns: auto 1fr;
    column-gap: 12px;
    row-gap: 4px;
    font-size: 12px;
    align-items: baseline;
  }

  .meas .k {
    color: var(--muted);
  }

  .meas .v {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .okbox {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--rs);
    background: var(--ok-soft);
    color: var(--ok-fg);
    font-size: 12.5px;
  }

  .grow {
    flex: 1;
  }

  .tips {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .tipbox {
    padding: 12px 14px;
    border-radius: var(--rs);
    background: var(--lav);
    color: var(--lav-fg);
    font-size: 12.5px;
    line-height: 1.5;
  }

  .tipbox.hint {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .tipbox.warn {
    background: var(--warn-soft);
    color: var(--warn-fg);
  }

  .tipbox.bad {
    background: var(--bad-soft);
    color: var(--bad-fg);
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11.5px;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgb(0 0 0 / 8%);
  }

  :global([data-theme="dark"]) code {
    background: rgb(255 255 255 / 10%);
  }

  .shots {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .head b {
    font-weight: 600;
    font-size: 14px;
  }

  .caption {
    font-variant-numeric: tabular-nums;
  }

  .thumbs {
    display: grid;
    gap: 5px;
  }
</style>
