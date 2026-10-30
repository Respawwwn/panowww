<script lang="ts">
// The window shell (spec §11.1): sidebar, detail, bottom bar. Navigation
// is state, not routes; later issues fill each slot.
import BottomBar from "./components/BottomBar.svelte";
import Sidebar from "./components/Sidebar.svelte";
import EmptyDetail from "./screens/EmptyDetail.svelte";

type Screen = { kind: "empty" } | { kind: "game"; id: string };

let screen = $state<Screen>({ kind: "empty" });
</script>

<div class="shell">
  <Sidebar />
  <main class="detail">
    {#if screen.kind === "empty"}
      <EmptyDetail />
    {:else}
      <!-- Game Setup and Sessions arrive with issues 15 to 18. -->
    {/if}
  </main>
  <BottomBar />
</div>

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
    min-width: 0;
    overflow: auto;
    padding: 24px;
  }
</style>
