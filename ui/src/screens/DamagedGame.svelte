<script lang="ts">
// A damaged Game file (spec §11.3): the header with Reveal / Delete, the
// banner, the validation errors, the advice. Re-read when the file
// changes on disk.
import Button from "../components/Button.svelte";
import HeaderRow from "../components/HeaderRow.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import Tip from "../components/Tip.svelte";
import { api } from "../lib/api";
import { games } from "../lib/games.svelte";
import type { GameEntry } from "../lib/generated/GameEntry";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";

type Props = { entry: Extract<GameEntry, { kind: "damaged" }> };

let { entry }: Props = $props();

let confirming = $state(false);
const platform = detectPlatform();
const fileName = $derived(entry.path.split("/").pop() ?? entry.path);
</script>

<HeaderRow title={fileName}>
  {#snippet actions()}
    <InlineConfirm
      {confirming}
      question={strings.detail.deleteConfirm}
      confirmLabel={strings.detail.delete}
      cancelLabel={strings.sessions.cancel}
      onconfirm={() => games.deleteFile(entry.path)}
      oncancel={() => (confirming = false)}
    >
      <Button variant="ghost" onclick={() => api.games.revealFile(entry.path)}>
        {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
      </Button>
      <Button variant="ghost" onclick={() => (confirming = true)}>{strings.detail.delete}</Button>
    </InlineConfirm>
  {/snippet}
</HeaderRow>

<section class="body">
  <div class="banner">
    <h2 class="t-section">{strings.damaged.title}</h2>
    <p class="t-tip">{entry.message}</p>
  </div>
  {#if entry.errors.length > 0}
    <ul class="errors">
      {#each entry.errors as error (error.field)}
        <li><code>{error.field}</code> {error.message}</li>
      {/each}
    </ul>
  {/if}
  <Tip kind="warn" text={strings.damaged.body} />
</section>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px;
    max-width: 640px;
  }

  .banner {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    border-radius: var(--rs);
    background: var(--warn-soft);
    color: var(--warn-fg);
  }

  .banner h2,
  .banner p {
    margin: 0;
    color: inherit;
  }

  .errors {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }

  code {
    font-size: 12px;
    color: var(--muted);
    margin-right: 6px;
  }
</style>
