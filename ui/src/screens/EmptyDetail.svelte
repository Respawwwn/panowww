<script lang="ts">
// Centred empty state when no Game exists (spec §11.1, §13.5, board
// B-Empty): the app icon, the title, the body, New Game / Import Game, the
// import tip under them.
import Button from "../components/Button.svelte";
import Icon from "../components/Icon.svelte";
import Mark from "../components/Mark.svelte";
import { games } from "../lib/games.svelte";
import { strings } from "../lib/strings";

type Props = {
  onnew?: () => void;
  onimport?: () => void;
};

let { onnew, onimport }: Props = $props();

/** No Game at all: the first-launch wording. Otherwise the home wording. */
const first = $derived(games.loaded && games.entries.length === 0);
</script>

<section class="empty">
  <Mark size={88} radius={20} />
  <h1 class="t-title">{first ? strings.empty.title : strings.empty.homeTitle}</h1>
  <p class="sub">{first ? strings.empty.body : strings.empty.homeBody}</p>
  <div class="actions">
    <Button variant="primary" size="lg" disabled={!onnew} onclick={onnew}>
      <Icon name="plus" size={15} />
      {strings.empty.newButton}
    </Button>
    <Button size="lg" disabled={!onimport} onclick={onimport}>
      <Icon name="import" />
      {strings.empty.importButton}
    </Button>
  </div>
  {#if first}
    <p class="t-tip">{strings.empty.import}</p>
  {/if}
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    max-width: 420px;
    margin: 0 auto;
    text-align: center;
    gap: 14px;
  }

  h1 {
    margin: 6px 0 0;
    font-size: 22px;
  }

  p {
    margin: 0;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-top: 6px;
  }
</style>
