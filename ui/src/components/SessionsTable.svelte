<script lang="ts">
// The Sessions table (spec §11.9): When · (Game) · State · Shots · Size ·
// Zip / Reveal / Delete, newest first; a failed row's `?` opens the
// popover; Delete swaps the row's actions for the inline confirmation.
import { api, isAppError } from "../lib/api";
import { RowConfirm } from "../lib/confirm.svelte";
import { formatBytes, formatDate } from "../lib/format";
import type { SessionSummary } from "../lib/generated/SessionSummary";
import { failedSessionText, stateTone } from "../lib/sessionText";
import { strings } from "../lib/strings";
import Button from "./Button.svelte";
import Chip from "./Chip.svelte";
import HelpButton from "./HelpButton.svelte";
import InlineConfirm from "./InlineConfirm.svelte";
import Popover from "./Popover.svelte";

type Props = {
  gameId?: string;
  showGame?: boolean;
  /** Called with the rows once loaded, for a header count. */
  onloaded?: (rows: SessionSummary[], totalBytes: number) => void;
};

let { gameId, showGame = false, onloaded }: Props = $props();

let rows = $state<SessionSummary[]>([]);
let sizes = $state<Record<string, number>>({});
let loadError = $state<string | null>(null);
let whyOpen = $state<string | null>(null);
let zipped = $state<Record<string, string>>({});
const confirm = new RowConfirm();

async function load() {
  loadError = null;
  try {
    rows = await api.sessions.list(gameId);
    const entries = await Promise.all(
      rows.map(async (r) => [r.folder, await api.sessions.size(r.folder).catch(() => 0)] as const),
    );
    sizes = Object.fromEntries(entries);
    onloaded?.(
      rows,
      entries.reduce((sum, [, b]) => sum + b, 0),
    );
  } catch (error) {
    loadError = isAppError(error) ? error.message : String(error);
  }
}

$effect(() => {
  void gameId;
  void load();
});

async function zip(folder: string) {
  try {
    zipped = { ...zipped, [folder]: await api.sessions.zip(folder) };
  } catch (error) {
    loadError = isAppError(error) ? error.message : String(error);
  }
}

async function remove(folder: string) {
  await confirm.confirm(async () => {
    await api.sessions.delete(folder);
    await load();
  });
}
</script>

{#if rows.length === 0 && !loadError}
  <p class="t-tip empty">{strings.sessions.empty}</p>
{:else}
  <table class="tbl">
    <thead>
      <tr>
        <th>{strings.sessions.columns.when}</th>
        {#if showGame}<th>{strings.sessions.columns.game}</th>{/if}
        <th>{strings.sessions.columns.state}</th>
        <th>{strings.sessions.columns.shots}</th>
        <th>{strings.sessions.columns.size}</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each rows as row (row.folder)}
        {@const r = row.record}
        <tr>
          <td>{formatDate(r.startedAt)}</td>
          {#if showGame}<td>{r.game.name}</td>{/if}
          <td>
            <span class="state">
              <Chip tone={stateTone(r.status)}>{strings.sessions.state[r.status]}</Chip>
              {#if r.status === "failed"}
                <HelpButton
                  label={strings.sessions.failed.why}
                  on={whyOpen === row.folder}
                  onclick={() => (whyOpen = whyOpen === row.folder ? null : row.folder)}
                />
                <Popover open={whyOpen === row.folder} onclose={() => (whyOpen = null)}>
                  {failedSessionText(r)}
                </Popover>
              {/if}
            </span>
          </td>
          <td class="num">{r.shotsTaken} / {r.shotsExpected}</td>
          <td class="num">{sizes[row.folder] === undefined ? "…" : formatBytes(sizes[row.folder] ?? 0)}</td>
          <td>
            <InlineConfirm
              confirming={confirm.confirming === row.folder}
              question={strings.sessions.deleteConfirm}
              confirmLabel={strings.sessions.delete}
              cancelLabel={strings.sessions.cancel}
              onconfirm={() => remove(row.folder)}
              oncancel={() => confirm.cancel()}
            >
              <Button size="sm" onclick={() => zip(row.folder)}>
                {zipped[row.folder] ? strings.sessions.zipped : strings.sessions.zip}
              </Button>
              <Button size="sm" onclick={() => api.sessions.reveal(row.folder)}>{strings.sessions.reveal}</Button>
              <Button size="sm" variant="danger" onclick={() => confirm.ask(row.folder)}>{strings.sessions.delete}</Button>
            </InlineConfirm>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
{#if loadError || confirm.error}
  <p class="error">{loadError ?? confirm.error}</p>
{/if}

<style>
  .empty,
  .error {
    margin: 0;
  }

  .error {
    font-size: 11.5px;
    color: var(--bad-fg);
  }

  .state {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .tbl :global(.chip),
  .tbl :global(.help) {
    height: 24px;
  }
</style>
