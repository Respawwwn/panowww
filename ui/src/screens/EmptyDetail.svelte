<script lang="ts">
// Centred empty state when no Game exists (spec §11.1). The buttons act
// once issue 15 brings the Game commands.
import { strings } from "../lib/strings";
</script>

<section class="empty">
  <h1>{strings.empty.title}</h1>
  <p>{strings.empty.body}</p>
  <p>{strings.empty.import}</p>
  <div class="actions">
    <button type="button" class="primary" disabled>{strings.empty.newButton}</button>
    <button type="button" class="secondary" disabled>{strings.empty.importButton}</button>
  </div>
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    gap: 8px;
  }

  h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
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

  button {
    height: var(--control-height);
    padding: 0 14px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
  }

  .primary {
    border: 0;
    background: var(--accent);
    color: #fff;
  }

  .secondary {
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--lavender-text);
  }

  button:disabled {
    opacity: 0.45;
  }
</style>
