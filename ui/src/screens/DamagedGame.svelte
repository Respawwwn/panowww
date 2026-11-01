<script lang="ts">
// A damaged Game file (spec §11.3, board B-Damaged): the toolbar with a
// disabled Start, the title row with Reveal / Delete, the amber-edged card
// with the reason, the validation errors and the advice. Re-read when the
// file changes on disk.
import Button from "../components/Button.svelte";
import DetailToolbar from "../components/DetailToolbar.svelte";
import HeaderRow from "../components/HeaderRow.svelte";
import Icon from "../components/Icon.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import { api } from "../lib/api";
import { games } from "../lib/games.svelte";
import type { GameEntry } from "../lib/generated/GameEntry";
import { interpolate, strings } from "../lib/strings";

type Props = { entry: Extract<GameEntry, { kind: "damaged" }> };

let { entry }: Props = $props();

let confirming = $state(false);
const fileName = $derived(entry.path.split("/").pop() ?? entry.path);
const title = $derived(fileName.replace(/\.panowww\.json$/, ""));
</script>

<DetailToolbar tab="setup" disabled onchange={() => {}} onstart={() => {}} startDisabled note={strings.damaged.fix} />
<HeaderRow {title}>
  {#snippet actions()}
    <InlineConfirm
      {confirming}
      question={strings.detail.deleteConfirm}
      confirmLabel={strings.detail.delete}
      cancelLabel={strings.sessions.cancel}
      onconfirm={() => games.deleteFile(entry.path)}
      oncancel={() => (confirming = false)}
    >
      <Button onclick={() => api.games.revealFile(entry.path)}>
        <Icon name="folder" size={14} />
        {strings.detail.reveal}
      </Button>
      <span class="delete">
        <Button variant="ghost" onclick={() => (confirming = true)}>{strings.detail.delete}</Button>
      </span>
    </InlineConfirm>
  {/snippet}
</HeaderRow>

<section class="body">
  <div class="card warn">
    <span class="sign"><Icon name="warn" size={22} /></span>
    <div class="text">
      <div>
        <h2>{strings.damaged.title}</h2>
        <p class="sub">{interpolate(strings.damaged.where, { file: fileName })}</p>
      </div>
      <ul class="errors">
        {#if entry.errors.length > 0}
          {#each entry.errors as error (error.field)}
            <li><b>{error.field}</b> {error.message}</li>
          {/each}
        {:else}
          <li>{entry.message}</li>
        {/if}
      </ul>
      <p class="sub">{strings.damaged.body}</p>
    </div>
  </div>
</section>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px 24px 20px;
  }

  .delete :global(.btn) {
    color: var(--bad);
    font-weight: 600;
  }

  .warn {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    padding: 18px 20px;
    border-color: var(--warn);
  }

  .sign {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--warn-soft);
    color: var(--warn-fg);
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
  }

  .sign :global(.ic) {
    stroke-width: 2;
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }

  h2,
  p {
    margin: 0;
  }

  h2 {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  .errors {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }

  .errors b {
    font-weight: 600;
  }
</style>
