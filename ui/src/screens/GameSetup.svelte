<script lang="ts">
// Game Setup (spec §11.2): left pane with the three cards (one open at a
// time), right pane with the Capture zone and Calibrate cards. The Game
// autosaves on every valid change.
import { onDestroy, untrack } from "svelte";
import { api, isAppError, type ZoneState } from "../lib/api";
import { GameEditor } from "../lib/editor.svelte";
import { disabledReason } from "../lib/game";
import { games } from "../lib/games.svelte";
import type { Game } from "../lib/generated/Game";
import { settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
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

// A new Game in the detail area gets a fresh editor; the old one saves
// what it still holds.
$effect(() => {
  const next = game;
  if (next.id !== editor.game.id) {
    void editor.flush();
    editor.dispose();
    editor = new GameEditor(next, (saved) => games.replace(saved));
    zoneShown = false;
  }
});

$effect(() => {
  const id = game.id;
  api.zone
    .get(id)
    .then((state) => {
      zone = state;
    })
    .catch(() => {
      zone = null;
    });
});

onDestroy(() => {
  void editor.flush();
  editor.dispose();
});

async function pickZone() {
  zoneError = null;
  try {
    zone = await api.zone.pick(editor.game.id);
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
    await api.run.start(editor.game.id, { kind: "session" });
  } catch (error) {
    startError = isAppError(error) ? error.message : String(error);
  }
}

const disabled = $derived(disabledReason(editor.game, zone?.status === "ready", platform));
</script>

<div class="panes">
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
    <CalibrateCard />
  </div>
</div>

<style>
  .panes {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(280px, 2fr);
    gap: 20px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
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
