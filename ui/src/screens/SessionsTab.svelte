<script lang="ts">
// The Sessions tab of a Game (spec §11.9): its table and the footer line
// naming the Sessions folder, linking to Settings.
import SessionsTable from "../components/SessionsTable.svelte";
import { games } from "../lib/games.svelte";
import { nav } from "../lib/nav.svelte";
import { settings } from "../lib/settings.svelte";
import { strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

type Props = {
  gameId: string;
  /** The count and total size, for the header beside the Game's name. */
  onloaded?: (n: number, bytes: number) => void;
};

let { gameId, onloaded }: Props = $props();

/** `<Sessions folder>/<Game>/`, as the Session folders are laid out. */
const folder = $derived(
  `${settings.value.sessionsFolder.replace(/\/$/, "")}/${games.find(gameId)?.name ?? ""}/`,
);
</script>

<div class="tab">
  <SessionsTable
    {gameId}
    onloaded={(rows, bytes) => onloaded?.(rows.length, bytes)}
    onopen={(id, folder) => nav.showSession(id, folder)}
  />
  <p class="t-tip footer">
    {strings.sessions.folder} <code>{folder}</code> ·
    <button type="button" class="link" onclick={() => windows.show("settings")}>
      {strings.sessions.changeFolder}
    </button>
  </p>
</div>

<style>
  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
  }

  .footer {
    margin: auto 0 0;
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--lav);
    color: var(--lav-fg);
  }

  .link {
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--lav-fg);
    font: inherit;
    text-align: left;
  }

  .link:hover {
    text-decoration: underline;
  }
</style>
