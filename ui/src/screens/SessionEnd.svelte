<script lang="ts">
// Session end (spec §11.7): the summary card per end state, Zip Session,
// Reveal, Back to Game, the full Shots grid. No preview.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import Tip from "../components/Tip.svelte";
import { api, isAppError } from "../lib/api";
import { formatBytes, formatDate, formatDuration, shotLabel } from "../lib/format";
import { games } from "../lib/games.svelte";
import type { Game } from "../lib/generated/Game";
import type { RunErrorKind } from "../lib/generated/RunErrorKind";
import { run } from "../lib/run.svelte";
import { detectPlatform } from "../lib/shortcut";
import { interpolate, strings } from "../lib/strings";

type Props = { game: Game; folder: string | null };

let { game, folder }: Props = $props();

const platform = detectPlatform();
const last = $derived(run.last);
const outcome = $derived(last?.ended.by === "finished" ? last.ended.outcome : null);
const shots = $derived(last?.shots ?? []);

let size = $state<number | null>(null);
let zipped = $state<string | null>(null);
let actionError = $state<string | null>(null);

$effect(() => {
  const f = folder;
  if (!f) {
    size = null;
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

const title = $derived.by(() => {
  const e = strings.session.end;
  const n = shots.length;
  switch (outcome?.status) {
    case "completed":
      return interpolate(e.completed.title, { n });
    case "interrupted":
      return interpolate(e.interrupted.title, { n });
    case "failed":
      return interpolate(e.failed.title, { n: n + 1 });
    default:
      return "";
  }
});

const line = $derived.by(() => {
  const started = last?.startedAt ? formatDate(last.startedAt) : "";
  const duration =
    last?.startedAt && last.endedAt ? formatDuration(last.endedAt - last.startedAt) : "";
  return [started, duration, `${game.rows} × ${game.shotsPerRow}`, "PNG"]
    .filter(Boolean)
    .join(" · ");
});

function reasonFragment(kind: RunErrorKind, message: string): string {
  const r = strings.sessions.failed.reason;
  switch (kind) {
    case "captureUnavailable":
      return r.captureUnavailable;
    case "displayGone":
      return interpolate(r.displayGone, { d: "?" });
    case "permissionRevoked":
      return r.permissionRevoked;
    case "shotFailed":
      return r.shotFailed;
    default:
      return message;
  }
}

const failedText = $derived.by(() => {
  if (outcome?.status !== "failed") {
    return null;
  }
  return interpolate(strings.sessions.failed.template, {
    n: shots.length + 1,
    reason: reasonFragment(outcome.error.kind, outcome.error.message),
    taken: shots.length,
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

const cells = $derived.by(() => {
  const byKey = new Map(shots.map((s) => [`${s.row}-${s.col}`, s.thumbnail]));
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
</script>

<div class="end">
  <section class="card summary">
    <div class="titles">
      <p class="t-overline">{game.name}</p>
      <h1 class="t-title">{title}</h1>
      <p class="t-label">{line}</p>
    </div>
    <div class="facts">
      {#if outcome?.status === "interrupted"}
        <Chip tone="warn">{strings.sessions.state.interrupted}</Chip>
      {:else if outcome?.status === "failed"}
        <Chip tone="bad">{strings.sessions.state.failed}</Chip>
      {:else if outcome}
        <Chip tone="ok">{strings.sessions.state.completed}</Chip>
      {/if}
      <span class="t-label">{interpolate(strings.session.end.shots, { n: shots.length })}</span>
      {#if size !== null}
        <span class="t-label">{formatBytes(size)}</span>
      {/if}
    </div>
    {#if outcome?.status === "completed"}
      <Tip kind="lav" text={strings.session.end.next} />
    {:else if outcome?.status === "interrupted"}
      <Tip text={strings.session.end.interrupted.note} />
    {:else if failedText}
      <Tip kind="warn" text={failedText} />
    {/if}
    <div class="actions">
      <Button variant="primary" disabled={!folder} onclick={zip}>{strings.session.end.zip}</Button>
      <Button disabled={!folder} onclick={reveal}>
        {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
      </Button>
      <span class="spacer"></span>
      <Button variant="ghost" onclick={() => games.select(game.id)}>{strings.session.end.backToGame}</Button>
    </div>
    {#if zipped}
      <p class="t-tip">{zipped}</p>
    {/if}
    {#if actionError}
      <p class="error">{actionError}</p>
    {/if}
  </section>
  <div class="thumbs" style:grid-template-columns="repeat({game.shotsPerRow}, minmax(0, 1fr))">
    {#each cells as cell (cell.label)}
      <Thumbnail label={cell.label} src={cell.src} state={cell.state} />
    {/each}
  </div>
</div>

<style>
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
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .titles p,
  .titles h1,
  .summary p {
    margin: 0;
  }

  .facts {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spacer {
    flex: 1;
  }

  .thumbs {
    display: grid;
    gap: 6px;
  }

  .error {
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
