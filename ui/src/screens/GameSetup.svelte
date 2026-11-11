<script lang="ts">
// Game Setup (spec §11.2): left pane with the three cards (one open at a
// time), right pane with the Capture zone and Calibrate cards. The Game
// autosaves on every valid change. Check overlap opens over the app.
import { onDestroy, untrack } from "svelte";
import RichLine from "../components/RichLine.svelte";
import { api, isAppError, type ZoneState } from "../lib/api";
import { GameEditor } from "../lib/editor.svelte";
import { adjustCount, disabledReason } from "../lib/game";
import { games } from "../lib/games.svelte";
import type { Game } from "../lib/generated/Game";
import { run } from "../lib/run.svelte";
import { settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";
import { zones } from "../lib/zone.svelte";
import CheckOverlap from "./CheckOverlap.svelte";
import CalibrateCard from "./setup/CalibrateCard.svelte";
import GameInputCard from "./setup/GameInputCard.svelte";
import GridCard from "./setup/GridCard.svelte";
import StartCard from "./setup/StartCard.svelte";
import ZoneCard from "./setup/ZoneCard.svelte";

type Props = { game: Game };

let { game }: Props = $props();

const platform = detectPlatform();
// The editor starts from the Game given at mount; the effect below swaps it
// when another Game is shown.
let editor = $state(
  new GameEditor(
    untrack(() => game),
    (saved) => games.replace(saved),
  ),
);
let openCard = $state<1 | 2 | 0>(1);
let zone = $state<ZoneState | null>(null);
let zoneShown = $state(false);
let zoneError = $state<string | null>(null);
let startError = $state<string | null>(null);
let overlap = $state<"turn" | "pitch" | null>(null);
/** The Check overlap run whose window was closed, so it does not reopen. */
let overlapClosed = $state<number | null>(null);

// A Check overlap run goes through the countdown screen, which unmounts
// this screen: the window reopens from the finished run (spec §11.8).
$effect(() => {
  const l = run.last;
  if (
    l &&
    l.gameId === game.id &&
    l.kind.kind === "checkOverlap" &&
    l.ended.by === "finished" &&
    l.startedAt !== overlapClosed
  ) {
    overlap = l.kind.axis;
  }
});

function closeOverlap() {
  overlapClosed = run.last?.startedAt ?? null;
  overlap = null;
}

// A new Game in the detail area gets a fresh editor; the old one saves
// what it still holds. Result lines belong to the Game that ran.
$effect(() => {
  const next = game;
  if (next.id !== editor.game.id) {
    // A Game that is gone from the folder is not written back.
    if (games.find(editor.game.id)) {
      void editor.flush();
    }
    editor.dispose();
    editor = new GameEditor(next, (saved) => games.replace(saved));
    zoneShown = false;
    overlap = null;
    run.clearLast();
  }
});

$effect(() => {
  const id = game.id;
  void zones.load(id).then((state) => {
    zone = state;
  });
});

onDestroy(() => {
  if (games.find(editor.game.id)) {
    void editor.flush();
  }
  editor.dispose();
  // The frame shown without a run goes with the screen; a run that just
  // started owns the overlay and Rust ignores this.
  if (zoneShown) {
    void api.zone.show(editor.game.id, false).catch(() => {});
  }
});

async function pickZone() {
  zoneError = null;
  try {
    zone = await api.zone.pick(editor.game.id);
    zones.set(editor.game.id, zone);
  } catch (error) {
    zoneError = isAppError(error) ? error.message : String(error);
  }
}

async function showZone(on: boolean) {
  zoneShown = on;
  try {
    await api.zone.show(editor.game.id, on);
  } catch {
    zoneShown = false;
  }
}

async function startSession() {
  startError = null;
  try {
    await editor.flush();
    await run.start(editor.game.id, { kind: "session" });
  } catch (error) {
    startError = isAppError(error) ? error.message : String(error);
  }
}

async function openOverlap(axis: "turn" | "pitch") {
  overlap = axis;
  await editor.flush();
  await run.start(editor.game.id, { kind: "checkOverlap", axis });
}

const disabled = $derived(disabledReason(editor.game, zone?.status === "ready", platform));
</script>

<div class="panes">
  <div class="cols">
  <div class="left">
    <GameInputCard {editor} open={openCard === 1} ontoggle={() => (openCard = openCard === 1 ? 0 : 1)} {platform} />
    <GridCard {editor} open={openCard === 2} ontoggle={() => (openCard = openCard === 2 ? 0 : 2)} />
    <StartCard {editor} sessionsFolder={settings.value.sessionsFolder} {disabled} onstart={startSession} />
    {#if startError}
      <p class="error">{startError}</p>
    {/if}
    {#if editor.saveError}
      <p class="error">{editor.saveError}</p>
    {/if}
  </div>
  <div class="right">
    <ZoneCard state={zone} shown={zoneShown} onpick={pickZone} onshow={showZone} error={zoneError} />
    <CalibrateCard
      game={editor.game}
      {disabled}
      onadjusted={(adjusted) => {
        editor.replace(adjusted);
        games.replace(adjusted);
      }}
      onadjustcount={(axis, by) => editor.update(adjustCount(editor.game, axis, by))}
      onopenoverlap={openOverlap}
    />
  </div>
  </div>
  <div class="hints">
    {#if disabled}
      <p class="t-tip">{strings.setup.start.disabled[disabled]}</p>
    {:else}
      <p class="t-tip"><RichLine text={strings.setup.start.shortcutStart} /></p>
      <p class="t-tip"><RichLine text={strings.setup.start.shortcutStop} /></p>
    {/if}
  </div>
</div>

{#if overlap}
  <CheckOverlap
    game={editor.game}
    axis={overlap}
    onretake={() => overlap && openOverlap(overlap)}
    onclose={closeOverlap}
  />
{/if}

<style>
  .panes {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 372px;
    gap: 20px;
    align-items: start;
  }

  .hints {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  .hints p {
    margin: 0;
  }

  .left,
  .right {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
