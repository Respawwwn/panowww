<script lang="ts">
// Capture zone · this machine (spec §11.2): status line with a dot, the
// amber boxes, *Set zone…* / *Change zone…*, *Show zone* / *Hide zone*.
import Button from "../../components/Button.svelte";
import Icon from "../../components/Icon.svelte";
import StatusBox from "../../components/StatusBox.svelte";
import Tip from "../../components/Tip.svelte";
import type { ZoneState } from "../../lib/api";
import { interpolate, strings } from "../../lib/strings";

type Props = {
  state: ZoneState | null;
  shown: boolean;
  onpick: () => void;
  onshow: (on: boolean) => void;
  error?: string | null;
};

let { state, shown, onpick, onshow, error = null }: Props = $props();

const values = $derived(
  state?.zone
    ? {
        n: state.displayIndex ?? "?",
        w: state.zone.w,
        h: state.zone.h,
        x: state.zone.x,
        y: state.zone.y,
      }
    : null,
);
</script>

<section class="card zone">
  <h2 class="t-section">{strings.setup.zone.title}</h2>
  {#if values}
    <StatusBox tone={state?.status === "ready" ? "ok" : "bad"}>
      {interpolate(strings.setup.zone.set, values)}
    </StatusBox>
    {#if state?.status === "displayGone" || state?.status === "displayChanged"}
      <Tip kind="warn" text={strings.setup.zone.displayChanged} values={{ n: values.n }} />
    {/if}
  {:else}
    <StatusBox tone="bad">{strings.setup.zone.notSet}</StatusBox>
  {/if}
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <div class="actions">
    <Button onclick={onpick}>
      <Icon name="crop" size={13} />
      {values ? strings.setup.zone.changeZone : strings.setup.zone.setZone}
    </Button>
    <Button disabled={!values} onclick={() => onshow(!shown)}>
      <Icon name="eye" size={13} />
      {shown ? strings.setup.zone.hideZone : strings.setup.zone.showZone}
    </Button>
  </div>
</section>

<style>
  .zone {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  h2 {
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 6px;
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
