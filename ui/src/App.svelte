<script lang="ts">
// The window shell (spec §11.1): sidebar, detail, bottom bar. Navigation is
// state (`nav`), not routes; `?gallery` shows the component gallery.
import BottomBar from "./components/BottomBar.svelte";
import DetailToolbar from "./components/DetailToolbar.svelte";
import HeaderRow from "./components/HeaderRow.svelte";
import Sidebar from "./components/Sidebar.svelte";
import { nav } from "./lib/nav.svelte";
import { strings } from "./lib/strings";
import EmptyDetail from "./screens/EmptyDetail.svelte";
import Gallery from "./screens/Gallery.svelte";
</script>

{#if nav.screen.kind === "gallery"}
  <Gallery />
{:else}
  <div class="shell">
    <Sidebar />
    <main class="detail">
      {#if nav.screen.kind === "empty"}
        <div class="pane">
          <EmptyDetail />
        </div>
      {:else if nav.screen.kind === "game"}
        <DetailToolbar
          tab={nav.screen.tab}
          onchange={(tab) => {
            if (nav.screen.kind === "game") {
              nav.showGame(nav.screen.id, tab);
            }
          }}
        />
        <HeaderRow title={strings.app.name} />
        <!-- Game Setup and the Sessions tab arrive with issues 15 to 18. -->
      {:else if nav.screen.kind === "damaged"}
        <HeaderRow title={strings.damaged.title} />
      {:else}
        <!-- Countdown, running and end screens arrive with issue 17. -->
      {/if}
    </main>
    <BottomBar />
  </div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-rows: 1fr auto;
    grid-template-areas:
      "sidebar detail"
      "bottom bottom";
    height: 100%;
  }

  .detail {
    grid-area: detail;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .pane {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 24px;
  }
</style>
