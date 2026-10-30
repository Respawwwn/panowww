<script lang="ts">
// The overlay drawing (spec §10, §12): everything inside the zone frame.
// Frame #a596ea 2 px just outside the rectangle; countdown digit centred;
// during a run a 2 px progress bar along the top inner edge and the
// `17 / 78` pill top right; the end pill for ≈ 2 s.
import { onMount } from "svelte";
import { api } from "../lib/api";
import type { OverlayState } from "../lib/generated/OverlayState";
import { counterText, endText, progressPercent } from "./text";

type Props = { mock?: boolean };

let { mock = false }: Props = $props();

let state = $state<OverlayState | null>(null);

const view = $derived(state?.view ?? null);

onMount(() => {
  if (mock) {
    return cycleMock();
  }
  let unlisten: (() => void) | null = null;
  void api.overlay
    .onState((next) => {
      state = next;
    })
    .then((stop) => {
      unlisten = stop;
    });
  // A state sent before this page listened.
  void api.overlay
    .state()
    .then((current) => {
      if (current && !state) {
        state = current;
      }
    })
    .catch(() => {});
  return () => unlisten?.();
});

function cycleMock(): () => void {
  const zone = { x: 160, y: 120, w: 960, h: 540 };
  const views: OverlayState["view"][] = [
    { kind: "frame" },
    { kind: "countdown", secondsLeft: 3 },
    { kind: "countdown", secondsLeft: 2 },
    { kind: "countdown", secondsLeft: 1 },
    { kind: "progress", taken: 0, expected: 78 },
    { kind: "progress", taken: 17, expected: 78 },
    { kind: "progress", taken: 52, expected: 78 },
    { kind: "end", stopped: false, taken: 78, expected: 78 },
    { kind: "end", stopped: true, taken: 47, expected: 78 },
  ];
  let i = 0;
  state = { zone, view: views[0] ?? { kind: "frame" } };
  const timer = setInterval(() => {
    i = (i + 1) % views.length;
    state = { zone, view: views[i] ?? { kind: "frame" } };
  }, 1200);
  return () => clearInterval(timer);
}
</script>

<div class="stage" class:mock>
  {#if state}
    <div
      class="frame"
      style:left="{state.zone.x - 2}px"
      style:top="{state.zone.y - 2}px"
      style:width="{state.zone.w}px"
      style:height="{state.zone.h}px"
    >
      {#if view?.kind === "countdown"}
        <div class="digit">{view.secondsLeft}</div>
      {:else if view?.kind === "progress" && view.expected > 0}
        <div class="bar" style:width="{progressPercent(view.taken, view.expected)}%"></div>
        <div class="pill counter">{counterText(view.taken, view.expected)}</div>
      {:else if view?.kind === "end"}
        <div class="pill end" class:stopped={view.stopped}>{endText(view)}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent;
    user-select: none;
    cursor: none;
  }

  .stage {
    position: fixed;
    inset: 0;
    font-family:
      "Inter Variable", Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui,
      sans-serif;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }

  /* Only for the ?mock page in a browser: a stand-in for the game. */
  .stage.mock {
    background:
      radial-gradient(circle at 30% 40%, #3b3360 0, transparent 45%),
      radial-gradient(circle at 70% 70%, #22406a 0, transparent 50%), #15122b;
  }

  .frame {
    position: absolute;
    box-sizing: content-box;
    border: 2px solid #a596ea;
    overflow: hidden;
  }

  .digit {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: clamp(96px, 30vmin, 320px);
    font-weight: 700;
    line-height: 1;
    color: #fff;
    text-shadow:
      0 0 24px rgb(0 0 0 / 80%),
      0 0 4px rgb(0 0 0 / 90%);
  }

  .bar {
    position: absolute;
    top: 0;
    left: 0;
    height: 2px;
    background: #a596ea;
    transition: width 200ms ease-out;
  }

  .pill {
    position: absolute;
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 1px 12px 0;
    border-radius: 999px;
    background: #a596ea;
    color: #15122b;
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    box-shadow: 0 1px 2px rgb(0 0 0 / 30%);
  }

  .counter {
    top: 12px;
    right: 12px;
  }

  .end {
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    height: 40px;
    padding: 1px 18px 0;
    font-size: 17px;
  }

  .end.stopped {
    background: #f0b544;
  }
</style>
