<script lang="ts">
// One Capture zone picker window (spec §11.4, board B-ZonePicker): the
// dimmed display behind, a purple rectangle drawn with the mouse, corner
// handles, the size chip, the display chip and the instruction above, the
// bottom card with X / Y / W / H, Show frame on screen, Cancel / Save zone.
// Esc cancels. The zone is in display points: this window's CSS pixels.
import { onDestroy, onMount } from "svelte";
import Button from "../components/Button.svelte";
import Checkbox from "../components/Checkbox.svelte";
import { api } from "../lib/api";
import type { PickerSetup } from "../lib/generated/PickerSetup";
import type { Rect } from "../lib/generated/Rect";
import { interpolate, strings } from "../lib/strings";
import { applyAppearance } from "../lib/theme";
import {
  anchorOf,
  between,
  clampPoint,
  type Handle,
  isUsable,
  moved,
  type Point,
  withField,
} from "./geometry";

type Props = { label: string; mock?: boolean };

let { label, mock = false }: Props = $props();

type Drag =
  | { kind: "draw"; start: Point }
  | { kind: "move"; start: Point; from: Rect }
  | { kind: "resize"; anchor: Point };

let setup = $state<PickerSetup | null>(null);
let rect = $state<Rect | null>(null);
let drag = $state<Drag | null>(null);
/** Off: the dim and the rectangle go, so the game shows as it is; the
 * bottom card stays to bring them back. */
let showFrame = $state(true);
let error = $state<string | null>(null);

const display = $derived(setup ? { w: setup.display.width, h: setup.display.height } : null);
const displayText = $derived(
  setup
    ? interpolate(strings.zonePicker.display, {
        n: setup.displayIndex,
        w: setup.display.width,
        h: setup.display.height,
        s: setup.display.scale,
      })
    : "",
);
const canSave = $derived(isUsable(rect) && drag === null);

// One zone across the displays: a rectangle started in another picker
// window clears this one.
let unlisten: Promise<() => void> | null = null;
onDestroy(() => {
  void unlisten?.then((f) => f());
});

onMount(() => {
  // Esc and the fields need the page to hold the keyboard focus.
  window.focus();
  if (!mock) {
    unlisten = api.picker.onClear(() => {
      rect = null;
      drag = null;
    });
  }
  if (mock) {
    setup = {
      display: { id: "mock", width: 1440, height: 900, scale: 2, hdr: false },
      displayIndex: 1,
      zone: { x: 240, y: 160, w: 960, h: 540 },
      appearance: "auto",
    };
    rect = setup.zone;
    return;
  }
  api.picker
    .setup(label)
    .then((s) => {
      setup = s;
      rect = s.zone;
      applyAppearance(s.appearance);
    })
    .catch((e) => {
      error = String(e?.message ?? e);
    });
});

function point(e: PointerEvent): Point {
  return display ? clampPoint({ x: e.clientX, y: e.clientY }, display) : { x: 0, y: 0 };
}

function startDraw(e: PointerEvent) {
  if (e.button !== 0 || !display) {
    return;
  }
  // No text selection while the rectangle is drawn over the chips.
  e.preventDefault();
  const start = point(e);
  drag = { kind: "draw", start };
  rect = between(start, start);
  if (!mock) {
    void api.picker.drawing(label);
  }
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function startMove(e: PointerEvent) {
  if (e.button !== 0 || !rect) {
    return;
  }
  e.stopPropagation();
  drag = { kind: "move", start: point(e), from: rect };
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function startResize(e: PointerEvent, handle: Handle) {
  if (e.button !== 0 || !rect) {
    return;
  }
  e.stopPropagation();
  drag = { kind: "resize", anchor: anchorOf(rect, handle) };
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function move(e: PointerEvent) {
  if (!drag || !display) {
    return;
  }
  const p = point(e);
  switch (drag.kind) {
    case "draw":
      rect = between(drag.start, p);
      break;
    case "resize":
      rect = between(drag.anchor, p);
      break;
    case "move":
      rect = moved(drag.from, p.x - drag.start.x, p.y - drag.start.y, display);
      break;
  }
}

function end() {
  if (!drag) {
    return;
  }
  drag = null;
  if (!isUsable(rect)) {
    rect = null;
  }
}

function edit(field: keyof Rect, input: HTMLInputElement) {
  if (rect && display) {
    rect = withField(rect, field, input.value, display);
    // Clamped or ignored values show as what was kept.
    input.value = String(rect[field]);
  }
}

function toggleFrame(on: boolean) {
  showFrame = on;
}

async function save() {
  if (!isUsable(rect) || mock) {
    return;
  }
  try {
    await api.picker.save(label, rect);
  } catch (e) {
    error = String((e as { message?: string })?.message ?? e);
  }
}

function cancel() {
  if (!mock) {
    void api.picker.cancel();
  }
}

function onkeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    cancel();
  } else if (e.key === "Enter" && canSave && !(e.target instanceof HTMLInputElement)) {
    void save();
  }
}
</script>

<svelte:window {onkeydown} />

<div
  class="stage"
  class:mock
  role="application"
  aria-label={strings.zonePicker.instruction}
  onpointerdown={startDraw}
  onpointermove={move}
  onpointerup={end}
  onpointercancel={end}
>
  {#if showFrame}
    <div class="dim"></div>
  {/if}

  <div class="top">
    <span class="chip display">{displayText}</span>
    <span class="chip instruction">{strings.zonePicker.instruction}</span>
  </div>

  {#if rect && showFrame}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="zone"
      class:dragging={drag !== null}
      style:left="{rect.x}px"
      style:top="{rect.y}px"
      style:width="{rect.w}px"
      style:height="{rect.h}px"
      onpointerdown={startMove}
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="handle nw" onpointerdown={(e) => startResize(e, "nw")}></span>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="handle ne" onpointerdown={(e) => startResize(e, "ne")}></span>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="handle sw" onpointerdown={(e) => startResize(e, "sw")}></span>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="handle se" onpointerdown={(e) => startResize(e, "se")}></span>
      <span class="chip size">{interpolate(strings.zonePicker.size, { w: rect.w, h: rect.h })}</span>
    </div>
  {/if}

  <p class="tip">{strings.zonePicker.tip}</p>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="card bottom" onpointerdown={(e) => e.stopPropagation()}>
    <div class="fields">
      {#each ["x", "y", "w", "h"] as const as field (field)}
        <label class="sub" for="z-{field}">{field.toUpperCase()}</label>
        <input
          id="z-{field}"
          type="number"
          min="0"
          disabled={!rect}
          value={rect ? rect[field] : ""}
          onchange={(e) => edit(field, e.currentTarget)}
        />
      {/each}
    </div>
    <Checkbox checked={showFrame} label={strings.zonePicker.showFrame} onchange={toggleFrame} />
    {#if error}
      <span class="error">{error}</span>
    {/if}
    <span class="spacer"></span>
    <Button onclick={cancel}>{strings.zonePicker.cancel}</Button>
    <Button variant="primary" disabled={!canSave} onclick={save}>{strings.zonePicker.save}</Button>
  </div>
</div>

<style>
  :global(html, body) {
    margin: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    /* The live display shows through; the dim layer sits on top of it. */
    background: transparent;
    -webkit-user-select: none;
    user-select: none;
  }

  :global(body *) {
    -webkit-user-select: none;
    user-select: none;
  }

  :global(input) {
    -webkit-user-select: text;
    user-select: text;
  }

  .stage {
    position: fixed;
    inset: 0;
    overflow: hidden;
    cursor: crosshair;
    color: var(--fg);
    font-family: var(--font);
    font-size: 13px;
  }

  /* Only for the ?mock page in a browser: a stand-in for the game. */
  .stage.mock {
    background:
      radial-gradient(circle at 30% 40%, #7a6aa0 0, transparent 45%),
      radial-gradient(circle at 70% 70%, #3f6a9a 0, transparent 50%),
      #4a4458;
  }

  .dim {
    position: absolute;
    inset: 0;
    background: var(--dim);
    pointer-events: none;
  }

  .top {
    position: absolute;
    left: 24px;
    top: 20px;
    display: flex;
    gap: 8px;
    pointer-events: none;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 1px 9px 0;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .chip.display {
    background: rgb(255 255 255 / 92%);
    color: #1b1740;
  }

  .chip.instruction {
    background: rgb(27 23 64 / 75%);
    color: #fff;
  }

  .zone {
    position: absolute;
    border: 2px solid #a596ea;
    background: rgb(255 255 255 / 6%);
    box-shadow: 0 0 0 9999px rgb(0 0 0 / 35%);
    box-sizing: border-box;
    cursor: move;
  }

  .zone.dragging {
    cursor: inherit;
  }

  .handle {
    position: absolute;
    width: 8px;
    height: 8px;
    background: #fff;
    border: 1px solid #6454b5;
    box-sizing: border-box;
  }

  .handle.nw {
    left: -5px;
    top: -5px;
    cursor: nwse-resize;
  }

  .handle.ne {
    right: -5px;
    top: -5px;
    cursor: nesw-resize;
  }

  .handle.sw {
    left: -5px;
    bottom: -5px;
    cursor: nesw-resize;
  }

  .handle.se {
    right: -5px;
    bottom: -5px;
    cursor: nwse-resize;
  }

  .chip.size {
    position: absolute;
    left: 50%;
    top: -30px;
    transform: translateX(-50%);
    background: #6454b5;
    color: #fff;
    pointer-events: none;
  }

  .tip {
    position: absolute;
    left: 24px;
    right: 24px;
    bottom: 78px;
    margin: 0;
    color: #fff;
    font-size: 12px;
    font-weight: 500;
    text-shadow: 0 1px 2px rgb(0 0 0 / 60%);
    pointer-events: none;
  }

  .bottom {
    position: absolute;
    left: 24px;
    right: 24px;
    bottom: 20px;
    padding: 10px 14px;
    display: flex;
    align-items: center;
    gap: 20px;
    cursor: default;
  }

  .fields {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  input {
    height: 28px;
    width: 72px;
    padding: 0 10px;
    border: 1px solid var(--line2);
    border-radius: var(--rs);
    background: var(--field);
    color: var(--fg);
    font: inherit;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  input:disabled {
    opacity: 0.5;
  }

  .error {
    font-size: 11.5px;
    color: var(--bad-fg);
  }

  .spacer {
    flex: 1;
  }
</style>
