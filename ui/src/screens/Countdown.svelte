<script lang="ts">
// Countdown (spec §11.5): the digit in a ring, the title, the body for a
// Session or a tool, the Display preview card, the stop line, Cancel. The
// same screen shows the error state when the run could not start.
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import PreviewCard from "../components/PreviewCard.svelte";
import RichLine from "../components/RichLine.svelte";
import { api, type Estimate } from "../lib/api";
import type { Game } from "../lib/generated/Game";
import type { RunKindDto } from "../lib/generated/RunKindDto";
import type { StartFailure } from "../lib/generated/StartFailure";
import { nav } from "../lib/nav.svelte";
import { run } from "../lib/run.svelte";
import { settings } from "../lib/settings.svelte";
import { interpolate, strings } from "../lib/strings";

type Props = { game: Game };

let { game }: Props = $props();

let estimate = $state<Estimate | null>(null);

$effect(() => {
  const snapshot = $state.snapshot(game) as Game;
  api.games
    .estimate(snapshot)
    .then((e) => {
      estimate = e;
    })
    .catch(() => {
      estimate = null;
    });
});

const current = $derived(run.current);
const failure = $derived.by((): StartFailure | null => {
  const l = run.last;
  return !current && l && l.gameId === game.id && l.ended.by === "cannotStart"
    ? l.ended.failure
    : null;
});
const kind = $derived<RunKindDto>(current?.kind ?? run.last?.kind ?? { kind: "session" });

function toolName(k: RunKindDto): string {
  const c = strings.setup.calibrate;
  switch (k.kind) {
    case "turnTest":
      return c.turnTestTitle;
    case "pitchTest":
      return c.pitchTestTitle;
    case "stepOnce":
      return c.stepOnceTitle;
    case "checkOverlap":
      return k.axis === "turn" ? c.overlapTurnTitle : c.overlapPitchTitle;
    case "session":
      return strings.detail.startSession;
  }
}

const previewOn = $derived(settings.value.displayPreview);

async function togglePreview(on: boolean) {
  settings.value = { ...settings.value, displayPreview: on };
  await api.run.setPreview(on).catch(() => {});
}

function failureText(f: StartFailure): {
  text: string;
  values?: Record<string, string>;
  action: string | null;
} {
  const e = strings.countdown.error;
  switch (f.reason) {
    case "accessibility":
      return { text: e.accessibility, action: e.openSystemSettings };
    case "screenRecording":
      return { text: e.screenRecording, action: e.openSystemSettings };
    case "vigem":
      return { text: e.vigem, action: e.installVigem };
    case "displayGone":
      return { text: e.displayGone, values: { n: f.display }, action: e.changeZone };
    case "captureUnavailable":
      return { text: e.captureUnavailable, action: e.tryAgain };
    case "noZone":
      return { text: strings.setup.start.disabled.noZone, action: e.changeZone };
  }
}

/** Back forgets the failed start first; the flow keeps this screen while it stands. */
function back() {
  run.clearLast();
  nav.showGame(game.id);
}

async function act(f: StartFailure) {
  switch (f.reason) {
    case "accessibility":
      await api.onboarding.openSystemSettings("accessibility");
      return;
    case "screenRecording":
      await api.onboarding.openSystemSettings("screenRecording");
      return;
    case "vigem":
      await api.onboarding.openUrl("https://github.com/nefarius/ViGEmBus/releases/tag/v1.22.0");
      return;
    case "displayGone":
    case "noZone":
      back();
      return;
    case "captureUnavailable":
      await run.start(game.id, kind);
      return;
  }
}
</script>

<div class="toolbar" data-tauri-drag-region>
  <div class="who">
    <Chip tone={failure ? "bad" : "lav"}>{failure ? strings.countdown.error.title : strings.countdown.starting}</Chip>
    <span class="name">{game.name}</span>
  </div>
  {#if failure}
    <Button size="lg" onclick={back}>{strings.countdown.error.back}</Button>
  {:else}
    <Button size="lg" onclick={() => run.stop()}>{strings.countdown.cancel}</Button>
  {/if}
</div>

<div class="countdown">
  {#if failure}
    {@const f = failureText(failure)}
    {@const [head, ...rest] = interpolate(f.text, f.values ?? {}).split(/(?<=\.)\s+/)}
    <h1>{strings.countdown.error.title}</h1>
    <div class="box">
      <b>{head}</b>
      {#if rest.length > 0}<span>{rest.join(" ")}</span>{/if}
    </div>
    {#if f.action}
      <div class="actions">
        <Button variant="primary" onclick={() => act(failure)}>{f.action}</Button>
      </div>
    {/if}
  {:else}
    <div class="ring" aria-live="polite">
      <svg viewBox="0 0 180 180" aria-hidden="true">
        <circle class="fill" cx="90" cy="90" r="86" style:stroke-dashoffset={540 * (1 - Math.min(1, (current?.secondsLeft ?? 0) / 5))} />
      </svg>
      <span class="digit">{current?.secondsLeft ?? ""}</span>
    </div>
    <h1>{strings.countdown.title}</h1>
    {#if kind.kind === "session"}
      <p class="sub">
        {interpolate(strings.countdown.body, {
          n: estimate?.shots ?? game.shotsPerRow * game.rows,
          duration: estimate?.duration ?? "",
        })}
      </p>
    {:else}
      <p class="sub">{interpolate(strings.countdown.bodyTool, { tool: toolName(kind) })}</p>
    {/if}
    <p class="sub stop"><RichLine text={strings.countdown.stop} /></p>
    <div class="preview">
      <PreviewCard
        src={current?.preview?.image ?? null}
        displayW={current?.preview?.displayW ?? 16}
        displayH={current?.preview?.displayH ?? 9}
        zone={current?.preview?.zone ?? null}
        on={previewOn}
        ontoggle={togglePreview}
      />
    </div>
  {/if}
</div>

<style>
  .toolbar {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .who {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .name {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .countdown {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 16px 24px 20px;
    min-height: 0;
    overflow: auto;
    text-align: center;
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    margin-top: 10px;
    font-size: 24px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .sub {
    max-width: 420px;
    color: var(--muted);
    font-size: 12.5px;
  }

  .stop {
    margin-top: 6px;
  }

  /* The amber box of the error state: the first sentence on its own line,
     the rest under it, everything centred like the title. */
  .box {
    width: min(420px, 100%);
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    border-radius: var(--rs);
    background: var(--warn-soft);
    color: var(--warn-fg);
    font-size: 12.5px;
    text-align: center;
  }

  .box b {
    font-weight: 600;
  }

  .ring {
    position: relative;
    width: 180px;
    height: 180px;
    border-radius: 50%;
    background: var(--lav);
    color: var(--lav-fg);
  }

  .ring svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }

  .fill {
    fill: none;
    stroke: var(--accent);
    stroke-width: 5;
    stroke-linecap: round;
    stroke-dasharray: 540;
    transition: stroke-dashoffset 0.9s linear;
  }

  .digit {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 96px;
    font-weight: 700;
    line-height: 1;
    /* Proportional digits: a tabular "1" sits off-centre in its box. */
    font-variant-numeric: proportional-nums;
    letter-spacing: 0;
  }

  .preview {
    width: 320px;
    margin-top: 18px;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
</style>
