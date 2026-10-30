<script lang="ts">
// App-shell bar at the bottom of every screen (spec §11.1): the update
// banner with a link to the release page and ✕ to dismiss for this launch.
// Hidden when up to date, offline, or the check is off.
import { api } from "../lib/api";
import { interpolate, strings } from "../lib/strings";
import { update } from "../lib/update.svelte";
</script>

{#if update.banner}
  <div class="bar" role="status">
    <a
      href={update.banner.url}
      onclick={(e) => {
        e.preventDefault();
        if (update.banner) {
          void api.onboarding.openUrl(update.banner.url);
        }
      }}
    >
      {interpolate(strings.update.banner, { version: update.banner.version })}
    </a>
    <button type="button" aria-label="Dismiss" onclick={() => update.dismiss()}>✕</button>
  </div>
{/if}

<style>
  .bar {
    grid-area: bottom;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 24px;
    background: var(--lav);
    color: var(--lav-fg);
    border-top: 1px solid var(--line);
    font-size: 12px;
    font-weight: 500;
  }

  a {
    color: inherit;
  }

  button {
    border: 0;
    background: transparent;
    color: var(--lav-fg);
    font-size: 12px;
    padding: 2px 6px;
    border-radius: 4px;
  }

  button:hover {
    background: var(--hover);
  }
</style>
