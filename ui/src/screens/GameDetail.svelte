<script lang="ts">
// The detail area of a valid Game (spec §11.1): toolbar with Setup |
// Sessions, the header row with Duplicate / Share / Delete, then the tab.
import Button from "../components/Button.svelte";
import DetailToolbar from "../components/DetailToolbar.svelte";
import HeaderRow from "../components/HeaderRow.svelte";
import Icon from "../components/Icon.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import { api } from "../lib/api";
import { formatBytes } from "../lib/format";
import { disabledReason } from "../lib/game";
import { games } from "../lib/games.svelte";
import type { Game } from "../lib/generated/Game";
import { type DetailTab, nav } from "../lib/nav.svelte";
import { run } from "../lib/run.svelte";
import { detectPlatform } from "../lib/shortcut";
import { interpolate, strings } from "../lib/strings";
import { zones } from "../lib/zone.svelte";
import GameSetup from "./GameSetup.svelte";
import SessionsTab from "./SessionsTab.svelte";

type Props = { game: Game; tab: DetailTab };

let { game, tab }: Props = $props();

let confirming = $state(false);
/** The Sessions tab's count and total size, once its table loaded. */
let sessions = $state<{ n: number; bytes: number } | null>(null);
let confirmingAll = $state(false);
let deletingAll = $state(false);
/** Bumped to remount the table after Delete all. */
let sessionsKey = $state(0);

async function deleteAllSessions() {
  deletingAll = true;
  try {
    const rows = await api.sessions.list(game.id);
    for (const row of rows) {
      await api.sessions.delete(row.folder);
    }
  } finally {
    deletingAll = false;
    confirmingAll = false;
    sessionsKey += 1;
  }
}

const platform = detectPlatform();
$effect(() => {
  void zones.load(game.id);
});

/** Why a Session cannot start, for the line beside the toolbar's button. */
const startReason = $derived(disabledReason(game, zones.ready(game.id), platform));
const startDisabled = $derived(startReason !== null || run.busy);

/** The Game's file, for Reveal. */
const path = $derived(
  games.entries.find((e) => e.kind === "game" && e.game.id === game.id)?.path ?? null,
);
</script>

<DetailToolbar
  {tab}
  disabled={nav.busy}
  onchange={(next) => nav.showGame(game.id, next)}
  onstart={() => run.start(game.id, { kind: "session" })}
  {startDisabled}
  note={startReason ? strings.setup.start.disabled[startReason] : null}
/>
{#if tab === "setup"}
  <HeaderRow title={game.name}>
    {#snippet actions()}
      <InlineConfirm
        {confirming}
        question={strings.detail.deleteConfirm}
        confirmLabel={strings.detail.delete}
        cancelLabel={strings.sessions.cancel}
        onconfirm={() => games.delete(game.id)}
        oncancel={() => (confirming = false)}
      >
        <Button onclick={() => games.duplicate(game.id)}>
          <Icon name="duplicate" size={14} />
          {strings.detail.duplicate}
        </Button>
        <Button onclick={() => games.share(game.id)}>
          <Icon name="share" size={14} />
          {strings.detail.share}
        </Button>
        <Button disabled={!path} onclick={() => path && api.games.revealFile(path)}>
          <Icon name="folder" size={14} />
          {strings.detail.reveal}
        </Button>
        <span class="delete">
          <Button variant="ghost" onclick={() => (confirming = true)}>{strings.detail.delete}</Button>
        </span>
      </InlineConfirm>
    {/snippet}
  </HeaderRow>
{:else}
  <HeaderRow title={game.name}>
    {#snippet aside()}
      {#if sessions}
        {interpolate(strings.sessions.summary, { n: sessions.n, size: formatBytes(sessions.bytes) })}
      {/if}
    {/snippet}
    {#snippet actions()}
      <InlineConfirm
        confirming={confirmingAll}
        question={interpolate(strings.sessions.deleteAllConfirm, { n: sessions?.n ?? 0 })}
        confirmLabel={strings.sessions.delete}
        cancelLabel={strings.sessions.cancel}
        onconfirm={deleteAllSessions}
        oncancel={() => (confirmingAll = false)}
      >
        <span class="delete">
          <Button variant="ghost" disabled={!sessions?.n || deletingAll} onclick={() => (confirmingAll = true)}>
            {strings.sessions.deleteAll}
          </Button>
        </span>
      </InlineConfirm>
    {/snippet}
  </HeaderRow>
{/if}
{#if tab === "setup"}
  <GameSetup {game} />
{:else}
  {#key sessionsKey}
    <SessionsTab gameId={game.id} onloaded={(n, bytes) => (sessions = { n, bytes })} />
  {/key}
{/if}

<style>
  .delete :global(.btn) {
    color: var(--bad);
    font-weight: 600;
  }
</style>
