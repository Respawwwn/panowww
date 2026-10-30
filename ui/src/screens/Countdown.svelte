<script lang="ts">
// Countdown (spec §11.5): the digit in a ring, the title, the body for a
// Session or a tool, the Display preview card, the stop line, Cancel. The
// same screen shows the error state when the run could not start.
import Button from "../components/Button.svelte";
import PreviewCard from "../components/PreviewCard.svelte";
import RichLine from "../components/RichLine.svelte";
import Tip from "../components/Tip.svelte";
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
      nav.showGame(game.id);
      return;
    case "captureUnavailable":
      await run.start(game.id, kind);
      return;
  }
}
</script>

<div class="countdown">
  <div class="main">
    <p class="t-overline">{game.name}</p>
    {#if failure}
      {@const f = failureText(failure)}
      <h1 class="t-title">{strings.countdown.error.title}</h1>
      <Tip kind="warn" text={f.text} values={f.values} />
      <div class="actions">
        {#if f.action}
          <Button variant="primary" onclick={() => act(failure)}>{f.action}</Button>
        {/if}
        <Button variant="ghost" onclick={() => nav.showGame(game.id)}>{strings.countdown.error.back}</Button>
      </div>
    {:else}
      <div class="ring" aria-live="polite">
        <svg viewBox="0 0 120 120" aria-hidden="true">
          <circle class="track" cx="60" cy="60" r="54" />
          <circle class="fill" cx="60" cy="60" r="54" style:stroke-dashoffset={339 * (1 - Math.min(1, (current?.secondsLeft ?? 0) / 5))} />
        </svg>
        <span class="digit t-counter">{current?.secondsLeft ?? ""}</span>
      </div>
      <h1 class="t-title">{strings.countdown.title}</h1>
      {#if kind.kind === "session"}
        <p class="body">
          {interpolate(strings.countdown.body, {
            n: estimate?.shots ?? game.shotsPerRow * game.rows,
            duration: estimate?.duration ?? "",
          })}
        </p>
      {:else}
        <p class="body">{interpolate(strings.countdown.bodyTool, { tool: toolName(kind) })}</p>
      {/if}
      <p class="t-tip"><RichLine text={strings.countdown.stop} /></p>
      <Button onclick={() => run.stop()}>{strings.countdown.cancel}</Button>
    {/if}
  </div>
  <aside class="side">
    <PreviewCard
      src={current?.preview?.image ?? null}
      displayW={current?.preview?.displayW ?? 16}
      displayH={current?.preview?.displayH ?? 9}
      zone={current?.preview?.zone ?? null}
      on={previewOn && !failure}
      ontoggle={togglePreview}
    />
  </aside>
</div>

<style>
  .countdown {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(280px, 2fr);
    gap: 24px;
    padding: 24px;
    min-height: 0;
    overflow: auto;
  }

  .main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 12px;
  }

  .main h1,
  .main p {
    margin: 0;
  }

  .body {
    max-width: 420px;
    color: var(--muted);
  }

  .ring {
    position: relative;
    width: 160px;
    height: 160px;
  }

  .ring svg {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }

  .track,
  .fill {
    fill: none;
    stroke-width: 6;
    stroke-linecap: round;
  }

  .track {
    stroke: var(--lav);
  }

  .fill {
    stroke: var(--accent);
    stroke-dasharray: 339;
    transition: stroke-dashoffset 0.9s linear;
  }

  .digit {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 56px;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .side {
    display: flex;
    align-items: flex-start;
  }
</style>
