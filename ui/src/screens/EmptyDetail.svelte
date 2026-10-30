<script lang="ts">
// Centred empty state when no Game exists (spec §11.1, §13.5). The buttons
// act once issue 15 brings the Game commands.
import Button from "../components/Button.svelte";
import { strings } from "../lib/strings";

type Props = {
  onnew?: () => void;
  onimport?: () => void;
};

let { onnew, onimport }: Props = $props();
</script>

<section class="empty">
  <h1 class="t-title">{strings.empty.title}</h1>
  <p>{strings.empty.body}</p>
  <p>{strings.empty.import}</p>
  <div class="actions">
    <Button variant="primary" size="lg" disabled={!onnew} onclick={onnew}>{strings.empty.newButton}</Button>
    <Button size="lg" disabled={!onimport} onclick={onimport}>{strings.empty.importButton}</Button>
  </div>
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    max-width: 440px;
    margin: 0 auto;
    text-align: center;
    gap: 8px;
  }

  h1 {
    margin: 0;
  }

  p {
    margin: 0;
    color: var(--muted);
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
</style>
