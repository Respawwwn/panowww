<script lang="ts">
// Session end (spec §11.7, §16.2, boards B-SessionEnd and Main): the state
// chip and the Game's name in the toolbar with *Back to Game*; the actions
// row (`sessionActions`: *Stitch*, or *Preview Panorama* and a *Stitch*
// that opens the panel once a Panorama exists, when the Experimental
// switch is on;
// *Open in…* per Stitcher set in Settings, completed Sessions only; *Zip
// Session*; *Reveal*), sized like the Game header's Duplicate and Share;
// the summary card with the title, the date line, the *Next* line with
// its links or the Panorama box, and the counts; the Shots card with the
// folder and the full grid, which fills the window when there is room and
// never shrinks below its grid. Shown for the run that just ended and for
// any Session opened from a table; no preview.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Icon from "../components/Icon.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import { api, isAppError, type Stitcher } from "../lib/api";
import { formatBytes, formatDuration, formatWhen, shotLabel } from "../lib/format";
import { nav } from "../lib/nav.svelte";
import { preview } from "../lib/preview.svelte";
import { run } from "../lib/run.svelte";
import { nextVariant, sessionActions } from "../lib/sessionEnd";
import { reasonFragment, stateTone } from "../lib/sessionText";
import type { SessionView } from "../lib/sessionView";
import { settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
import { percent, stitch } from "../lib/stitch.svelte";
import { type InterpolationValues, interpolate, splitPlaceholders, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

type Props = { view: SessionView };

let { view }: Props = $props();

const platform = detectPlatform();
const game = $derived(view.game);
const folder = $derived(view.folder);
const taken = $derived(view.shotsTaken);

let size = $state<number | null>(null);
let zipped = $state<string | null>(null);
let actionError = $state<string | null>(null);
/** History thumbnails, generated on demand by file name. */
let thumbs = $state<Record<string, string>>({});

$effect(() => {
  const f = folder;
  size = null;
  if (!f) {
    return;
  }
  api.sessions
    .size(f)
    .then((bytes) => {
      size = bytes;
    })
    .catch(() => {
      size = null;
    });
});

// Shots without an in-memory thumbnail get one from the folder, a few at a
// time, newest view first.
$effect(() => {
  const f = folder;
  const missing = view.shots.filter((s) => !s.src);
  if (!f || missing.length === 0) {
    return;
  }
  let cancelled = false;
  thumbs = {};
  (async () => {
    for (const shot of missing) {
      if (cancelled) {
        return;
      }
      try {
        const src = await api.sessions.thumbnail(f, shot.file);
        thumbs = { ...thumbs, [shot.file]: src };
      } catch {
        // A missing file stays a dashed cell.
      }
    }
  })();
  return () => {
    cancelled = true;
  };
});

const chip = $derived.by(() => {
  const s = strings.session.end.chip;
  switch (view.status) {
    case "completed":
      return s.completed;
    case "interrupted":
      return s.interrupted;
    case "failed":
      return s.failed;
    case "running":
      return s.running;
  }
});

const title = $derived.by(() => {
  const e = strings.session.end;
  switch (view.status) {
    case "completed":
      return interpolate(e.completed.title, { n: taken });
    case "failed":
      return interpolate(e.failed.title, { n: taken + 1 });
    default:
      return interpolate(e.interrupted.title, { n: taken });
  }
});

const line = $derived.by(() => {
  const started = view.startedAt ? formatWhen(view.startedAt, strings.sessions) : "";
  const duration =
    view.startedAt && view.endedAt
      ? formatDuration(view.endedAt - view.startedAt).replace(/^≈ /, "")
      : "";
  return [
    started,
    duration,
    interpolate(strings.session.end.grid, { rows: game.rows, shotsPerRow: game.shotsPerRow }),
    strings.session.end.png,
  ]
    .filter(Boolean)
    .join(" · ");
});

const failedText = $derived.by(() => {
  if (view.status !== "failed" || !view.error) {
    return null;
  }
  return interpolate(strings.sessions.failed.template, {
    n: taken + 1,
    reason: reasonFragment(view.error.kind, view.error.message, view.displayId ?? "?"),
    taken,
  });
});

async function zip() {
  if (!folder) {
    return;
  }
  actionError = null;
  try {
    zipped = await api.sessions.zip(folder);
  } catch (error) {
    actionError = isAppError(error) ? error.message : String(error);
  }
}

async function reveal() {
  if (folder) {
    await api.sessions.reveal(folder).catch(() => {});
  }
}

/** The Stitchers set in Settings, in display order; completed Sessions only. */
const stitchers = $derived(
  view.status === "completed"
    ? (["ptgui", "autopanoGiga"] as const).filter((s) => settings.value.stitchers[s] !== null)
    : [],
);

async function openIn(stitcher: Stitcher) {
  if (!folder) {
    return;
  }
  actionError = null;
  try {
    await api.sessions.openIn(folder, stitcher);
  } catch (error) {
    actionError = isAppError(error) ? error.message : String(error);
  }
}

/** The Panorama in the folder: written during this window, or found on disk. */
const panorama = $derived(stitch.entryFor(folder) ?? view.panorama);
const stitching = $derived(stitch.isStitching(folder));
const stitchOn = $derived(settings.value.experimental.stitch);
const actions = $derived(
  sessionActions({
    status: view.status,
    stitchOn,
    stitchers,
    hasPanorama: panorama !== null,
    stitching,
  }),
);
/** *Stitch* waits for the Runner (spec §16.2) and for a Stitch elsewhere. */
const stitchDisabled = $derived(!folder || run.busy || (stitch.running !== null && !stitching));

const nextSegments = $derived(
  splitPlaceholders(strings.session.end.next[nextVariant(stitchOn, stitchers)], [
    "stitchHere",
    "stitchers",
  ]),
);

const panoramaSegments = $derived(splitPlaceholders(strings.session.end.panorama, ["file"]));
const panoramaValues = $derived.by(
  (): InterpolationValues =>
    panorama
      ? {
          when: formatWhen(panorama.stitchedAt, strings.sessions),
          width: panorama.width,
          height: panorama.height,
          size: formatBytes(panorama.bytes),
        }
      : {},
);

/** Opens the Stitch panel: the Stitch starts there, after its options. */
function openStitch() {
  if (folder) {
    nav.showStitch(game.id, folder);
  }
}

/** The preview window over the app (spec §16.4). */
function previewPanorama() {
  if (folder && panorama) {
    preview.show({
      folder,
      entry: panorama,
      gameName: game.name,
      startedAt: view.startedAt,
      startPole: game.startPole,
    });
    windows.show("panorama");
  }
}

const cells = $derived.by(() => {
  const byKey = new Map(
    view.shots.map((s) => [`${s.row}-${s.col}`, s.src ?? thumbs[s.file] ?? null]),
  );
  const list: { label: string; src: string | null; state: "done" | "todo" }[] = [];
  for (let row = 1; row <= game.rows; row += 1) {
    for (let col = 1; col <= game.shotsPerRow; col += 1) {
      const key = `${row}-${col}`;
      list.push({
        label: shotLabel(row, col),
        src: byKey.get(key) ?? null,
        state: byKey.has(key) ? "done" : "todo",
      });
    }
  }
  return list;
});

// Shown with a trailing separator of this OS: a folder, not a file.
const sep = detectPlatform() === "windows" ? "\\" : "/";
const folderText = $derived(folder ? `${folder.replace(/[\\/]$/, "")}${sep}` : "");

/** Back lands on the Game's Sessions tab, where this Session is listed. */
async function back() {
  await api.games.select(game.id).catch(() => {});
  nav.showGame(game.id, "sessions");
}
</script>

<div class="toolbar" data-tauri-drag-region>
  <Button onclick={back}>
    <Icon name="back" />
    {strings.session.end.backToGame}
  </Button>
</div>

<div class="end">
  <!-- The heading, as on Setup: the Game, the Session's chip, the actions. -->
  <div class="heading">
  <div class="who">
    <h1 class="t-title name">{game.name}</h1>
    <Chip tone={stateTone(view.status)}>
      {#if view.status === "completed"}<Icon name="check" size={13} />{/if}
      {chip}
    </Chip>
  </div>
  <div class="actions">
    {#each actions as action (action.id === "openIn" ? `openIn-${action.stitcher}` : action.id)}
      {#if action.id === "stitch"}
        <Button variant="primary" disabled={!folder} onclick={openStitch}>
          <Icon name="stitch" size={14} />
          {strings.session.end.stitch}
        </Button>
      {:else if action.id === "stitching"}
        <Button variant="primary" onclick={openStitch}>
          <Icon name="stitch" size={14} />
          {interpolate(strings.session.end.stitching, { pct: percent(stitch.running?.progress ?? null) })}
        </Button>
      {:else if action.id === "previewPanorama"}
        <Button variant="primary" disabled={!folder} onclick={previewPanorama}>
          <Icon name="pano" size={14} />
          {strings.session.end.previewPanorama}
        </Button>
      {:else if action.id === "stitchPanel"}
        <Button disabled={!folder} onclick={openStitch}>
          <Icon name="stitch" size={14} />
          {strings.session.end.stitch}
        </Button>
      {:else if action.id === "openIn"}
        <Button variant={action.primary ? "primary" : "secondary"} disabled={!folder} onclick={() => openIn(action.stitcher)}>
          <Icon name={action.stitcher === "ptgui" ? "ptgui" : "autopano"} size={14} />
          {interpolate(strings.session.end.openIn, { stitcher: strings.settings.stitcher[action.stitcher] })}
        </Button>
      {:else if action.id === "zip"}
        <Button variant={action.primary ? "primary" : "secondary"} disabled={!folder} onclick={zip}>
          <Icon name="zip" size={14} />
          {strings.session.end.zip}
        </Button>
      {:else}
        <Button disabled={!folder} onclick={reveal}>
          <Icon name="folder" size={14} />
          {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
        </Button>
      {/if}
    {/each}
  </div>
  </div>
  {#if zipped || actionError}
    <p class="note" class:error={Boolean(actionError)}>{#if actionError}{actionError}{:else}<code title={zipped}>{zipped}</code>{/if}</p>
  {/if}

  <section class="card summary">
    <div class="titles">
      <h2>{title}</h2>
      <p class="sub">{line}</p>
      {#if view.status === "interrupted"}
        <p class="sub">{strings.session.end.interrupted.note}</p>
      {:else if failedText}
        <p class="sub bad">{failedText}</p>
      {/if}
      {#if view.status === "completed" && panorama && stitchOn}
        <div class="tipbox done">
          <span>
            {#each panoramaSegments as segment, i (i)}
              {#if segment.kind === "slot"}<code class="file">{panorama.file}</code>{:else}{interpolate(segment.text, panoramaValues)}{/if}
            {/each}
          </span>
        </div>
      {:else if view.status === "completed"}
        <p class="tipbox next">
          {#each nextSegments as segment, i (i)}
            {#if segment.kind === "text"}{segment.text}{:else if segment.name === "stitchHere"}<button
                type="button"
                class="lnk"
                disabled={!folder}
                onclick={openStitch}>{strings.session.end.next.stitchHere}</button
              >{:else}{#each stitchers as stitcher, j (stitcher)}{#if j > 0}{strings.session.end.next.or}{/if}<button
                  type="button"
                  class="lnk"
                  disabled={!folder}
                  onclick={() => openIn(stitcher)}>{strings.settings.stitcher[stitcher]}</button
                >{/each}{/if}
          {/each}
        </p>
      {/if}
    </div>
    <div class="facts">
      <div class="fact">
        <div class="big">{taken} / {view.shotsExpected}</div>
        <div class="sub">{strings.session.end.shotsLabel}</div>
      </div>
      <div class="fact">
        <div class="big">{size === null ? "…" : formatBytes(size)}</div>
        <div class="sub">{strings.session.end.onDisk}</div>
      </div>
    </div>
  </section>

  <section class="card shots">
    <div class="head">
      <b>{strings.session.end.shotsLabel}</b>
      {#if folderText}<code>{folderText}</code>{/if}
    </div>
    <div class="thumbs" style:grid-template-columns="repeat({game.shotsPerRow}, minmax(0, 1fr))">
      {#each cells as cell (cell.label)}
        <Thumbnail label={cell.label} src={cell.src} state={cell.state} />
      {/each}
    </div>
  </section>
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

  .who {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .name {
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .end {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 16px 18px;
  }

  .titles {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
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

  .heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .note {
    margin: -8px 0 0;
    font-size: 11.5px;
    color: var(--muted);
    text-align: right;
  }

  .note.error {
    color: var(--bad-fg);
  }

  /* Grows to the bottom of the window when there is room, never shrinks
     below its grid: the background always covers every Shot. */
  .shots {
    flex: 1 0 auto;
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

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--lav);
    color: var(--lav-fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tipbox {
    align-self: flex-start;
    margin-top: 6px;
    font-size: 11.5px;
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--lav);
    color: var(--lav-fg);
  }

  code.file {
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgb(0 0 0 / 8%);
    color: inherit;
  }

  :global([data-theme="dark"]) code.file {
    background: rgb(255 255 255 / 10%);
  }

  .lnk {
    display: inline;
    padding: 0;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }

  .lnk:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .thumbs {
    display: grid;
    gap: 6px;
  }
</style>
