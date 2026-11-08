<script lang="ts">
// Session end (spec §11.7, board B-SessionEnd): the state chip and the
// Game's name in the toolbar with *Back to Game*; the summary card with
// the actions row (*Open in…* per Stitcher set in Settings, completed
// Sessions only, the first one primary; *Zip Session*, primary when no
// Stitcher is set; *Reveal*), sized like the Game header's Duplicate and
// Share; the summary card with the title, the date line and the counts;
// the Shots card with the folder and the full grid, which fills the
// window when there is room and never shrinks below its grid. Shown for
// the run that just ended and for any Session opened from a table; no
// preview.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Icon from "../components/Icon.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import Tip from "../components/Tip.svelte";
import { api, isAppError, type Stitcher } from "../lib/api";
import { formatBytes, formatDuration, formatWhen, shotLabel } from "../lib/format";
import { nav } from "../lib/nav.svelte";
import { reasonFragment, stateTone } from "../lib/sessionText";
import type { SessionView } from "../lib/sessionView";
import { settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
import { interpolate, strings } from "../lib/strings";

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
  <div class="who">
    <Chip tone={stateTone(view.status)}>
      {#if view.status === "completed"}<Icon name="check" size={13} />{/if}
      {chip}
    </Chip>
    <span class="name">{game.name}</span>
  </div>
  <Button size="lg" onclick={back}>
    <Icon name="back" />
    {strings.session.end.backToGame}
  </Button>
</div>

<div class="end">
  <div class="actions">
    {#each stitchers as stitcher, i (stitcher)}
      <Button variant={i === 0 ? "primary" : "secondary"} disabled={!folder} onclick={() => openIn(stitcher)}>
        <Icon name={stitcher === "ptgui" ? "ptgui" : "autopano"} size={14} />
        {interpolate(strings.session.end.openIn, { stitcher: strings.settings.stitcher[stitcher] })}
      </Button>
    {/each}
    <Button variant={stitchers.length === 0 ? "primary" : "secondary"} disabled={!folder} onclick={zip}>
      <Icon name="zip" size={14} />
      {strings.session.end.zip}
    </Button>
    <Button disabled={!folder} onclick={reveal}>
      <Icon name="folder" size={14} />
      {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
    </Button>
  </div>
  {#if zipped || actionError}
    <p class="note" class:error={Boolean(actionError)}>{actionError ?? zipped}</p>
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
      {#if view.status === "completed"}
        <span class="t-tip next">{strings.session.end.next}</span>
      {/if}
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
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
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

  .actions {
    display: flex;
    gap: 8px;
  }

  .note {
    margin: -8px 0 0;
    font-size: 11.5px;
    color: var(--muted);
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

  .next {
    margin-left: auto;
    white-space: nowrap;
  }

  .thumbs {
    display: grid;
    gap: 6px;
  }
</style>
