<script lang="ts">
// Check overlap (spec §11.8): Turn | Pitch, the two Samples onion-skinned
// (second at 55 %), the dashed band with the Overlap chip, the alignment
// slider, the verdict box, Retake Samples / Done. Drawn as a window over
// the app; the Samples stay in memory.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Tip from "../components/Tip.svelte";
import type { Game } from "../lib/generated/Game";
import { overlapPercent, verdict, verdictText } from "../lib/overlap";
import { run } from "../lib/run.svelte";
import { interpolate, strings } from "../lib/strings";

type Props = {
  game: Game;
  axis: "turn" | "pitch";
  onretake: () => void;
  onclose: () => void;
};

let { game, axis, onretake, onclose }: Props = $props();

/** The Samples of the run in progress, or of the last one of this Game. */
const samples = $derived.by(() => {
  const c = run.current;
  if (c && c.gameId === game.id && c.kind.kind === "checkOverlap") {
    return c.samples;
  }
  const l = run.last;
  if (l && l.gameId === game.id && l.kind.kind === "checkOverlap") {
    return l.samples;
  }
  return { first: null, second: null };
});

let offset = $state(0);
let size = $state(0);
let naturalW = $state(16);
let naturalH = $state(9);

function onFirstLoaded(event: Event) {
  const img = event.currentTarget as HTMLImageElement;
  naturalW = img.naturalWidth || 16;
  naturalH = img.naturalHeight || 9;
  size = axis === "turn" ? naturalW : naturalH;
  offset = Math.round(size * 0.7);
}

$effect(() => {
  // Switching axis changes the extent the slider spans.
  size = axis === "turn" ? naturalW : naturalH;
  offset = Math.round(size * 0.7);
});

const percent = $derived(overlapPercent(offset, size));
const p = $derived(Math.round(percent));
const v = $derived(verdict(percent));

/** The second Sample's shift, as a share of its own extent. */
const shift = $derived(size > 0 ? (offset / size) * 100 : 0);

/** The overlapping band, as shares of the extent. */
const band = $derived.by(() => {
  const share = Math.abs(shift);
  const from = shift >= 0 ? share : 0;
  return { from, length: Math.max(0, 100 - share) };
});

const ready = $derived(Boolean(samples.first && samples.second));
</script>

<div class="scrim" role="presentation">
  <div class="win" role="dialog" aria-labelledby="overlap-title">
    <header class="win-t">
      <h2 id="overlap-title">{interpolate(strings.overlap.title, { game: game.name })}</h2>
      <Chip>{axis === "turn" ? strings.setup.calibrate.turn : strings.setup.calibrate.pitch}</Chip>
    </header>
    <div class="win-b">
      <div class="sub">
        <span>{strings.overlap.instruction}</span>
        <Chip>{strings.overlap.target}</Chip>
      </div>
      <div class="stage" style:aspect-ratio="{naturalW} / {naturalH}">
        {#if ready}
          <img class="first" src={samples.first} alt={strings.overlap.sample1} onload={onFirstLoaded} />
          <img
            class="second"
            src={samples.second}
            alt={strings.overlap.sample2}
            style:transform={axis === "turn" ? `translateX(${shift}%)` : `translateY(${shift}%)`}
          />
          <span
            class="band"
            style:left={axis === "turn" ? `${band.from}%` : "0"}
            style:width={axis === "turn" ? `${band.length}%` : "100%"}
            style:top={axis === "pitch" ? `${band.from}%` : "0"}
            style:height={axis === "pitch" ? `${band.length}%` : "100%"}
          >
            <Chip tone="lav">{interpolate(strings.overlap.chip, { p })}</Chip>
          </span>
          <span class="pill one">{strings.overlap.sample1}</span>
          <span class="pill two">{strings.overlap.sample2}</span>
        {:else}
          <span class="waiting">{run.busy ? strings.overlap.waiting : strings.overlap.none}</span>
        {/if}
      </div>
      <label class="slider">
        <span class="t-label">{strings.overlap.align}</span>
        <input type="range" min={-size} max={size} step="1" bind:value={offset} disabled={!ready} />
        <span class="t-label px">{offset > 0 ? "−" : offset < 0 ? "+" : ""}{Math.abs(offset)} px</span>
      </label>
      {#if ready}
        <div class="verdict" class:ok={v === "ok"}>
          <Chip tone={v === "ok" ? "ok" : "warn"}>{strings.overlap.label[v]}</Chip>
          <span>{interpolate(verdictText(v, axis), { p })}</span>
        </div>
      {/if}
      <Tip kind="lav" text={strings.overlap.tip} />
    </div>
    <footer class="win-f">
      <Button disabled={run.busy} onclick={onretake}>{strings.overlap.retake}</Button>
      <span class="spacer"></span>
      <Button variant="primary" onclick={onclose}>{strings.overlap.done}</Button>
    </footer>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--dim);
  }

  .win {
    width: min(920px, 92vw);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    background: var(--content);
    border-radius: var(--r);
    box-shadow: var(--pop);
    overflow: hidden;
  }

  .win-t {
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 14px;
    background: var(--sidebar);
    flex: none;
  }

  h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .win-b {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .sub {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    color: var(--muted);
    font-size: 12.5px;
  }

  .stage {
    position: relative;
    width: 100%;
    max-height: 56vh;
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--line);
    background: #1b1740;
  }

  .stage img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: fill;
    display: block;
  }

  .second {
    opacity: 0.55;
  }

  .band {
    position: absolute;
    box-sizing: border-box;
    border: 2px dashed #a596ea;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 8px;
    pointer-events: none;
  }

  .waiting {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #a8a4c6;
    font-size: 13px;
  }

  .slider {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .slider .px {
    width: 60px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .pill {
    position: absolute;
    bottom: 10px;
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 1px 9px 0;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1;
  }

  .pill.one {
    left: 10px;
    background: rgb(255 255 255 / 90%);
    color: #1b1740;
  }

  .pill.two {
    right: 10px;
    background: rgb(27 23 64 / 85%);
    color: #fff;
  }

  .verdict {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--rs);
    background: var(--warn-soft);
    color: var(--warn-fg);
    font-size: 12.5px;
  }

  .verdict.ok {
    background: var(--ok-soft);
    color: var(--ok-fg);
  }

  .verdict :global(.chip) {
    background: var(--card);
  }

  .slider input {
    flex: 1;
    accent-color: var(--accent);
  }

  .win-f {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--line);
    flex: none;
  }

  .spacer {
    flex: 1;
  }
</style>
