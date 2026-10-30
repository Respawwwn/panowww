<script lang="ts">
// Toolbar of the detail area (spec §11.1): the segmented Setup | Sessions
// on the left; no second Start button (decided at assembly).
import type { DetailTab } from "../lib/nav.svelte";
import { strings } from "../lib/strings";
import PillTabs from "./PillTabs.svelte";

type Props = {
  tab: DetailTab;
  onchange: (tab: DetailTab) => void;
  disabled?: boolean;
};

let { tab, onchange, disabled = false }: Props = $props();

const items: { id: DetailTab; label: string }[] = [
  { id: "setup", label: strings.detail.setup },
  { id: "sessions", label: strings.detail.sessions },
];
</script>

<div class="toolbar" data-tauri-drag-region>
  <PillTabs {items} value={tab} {disabled} onchange={(id) => onchange(id as DetailTab)} />
</div>

<style>
  .toolbar {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }
</style>
