<script lang="ts">
// The detail area of a valid Game (spec §11.1): toolbar with Setup |
// Sessions, the header row with Duplicate / Share / Delete, then the tab.
import Button from "../components/Button.svelte";
import DetailToolbar from "../components/DetailToolbar.svelte";
import HeaderRow from "../components/HeaderRow.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import { games } from "../lib/games.svelte";
import type { Game } from "../lib/generated/Game";
import { type DetailTab, nav } from "../lib/nav.svelte";
import { strings } from "../lib/strings";
import GameSetup from "./GameSetup.svelte";
import SessionsTab from "./SessionsTab.svelte";

type Props = { game: Game; tab: DetailTab };

let { game, tab }: Props = $props();

let confirming = $state(false);
</script>

<DetailToolbar {tab} disabled={nav.busy} onchange={(next) => nav.showGame(game.id, next)} />
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
      <Button variant="ghost" onclick={() => games.duplicate(game.id)}>{strings.detail.duplicate}</Button>
      <Button variant="ghost" onclick={() => games.share(game.id)}>{strings.detail.share}</Button>
      <Button variant="ghost" onclick={() => (confirming = true)}>{strings.detail.delete}</Button>
    </InlineConfirm>
  {/snippet}
</HeaderRow>
{#if tab === "setup"}
  <GameSetup {game} />
{:else}
  <SessionsTab gameId={game.id} />
{/if}
