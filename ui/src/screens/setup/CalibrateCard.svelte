<script lang="ts">
// Calibrate (spec §11.2, §7.5): the four numbered tools with their adjust
// rows, Step once, the result line under the tool that ran, the amber
// line and the auto-opened checklist when the screen did not change, the
// permanent Nothing moved? link.
import Badge from "../../components/Badge.svelte";
import Button from "../../components/Button.svelte";
import HelpButton from "../../components/HelpButton.svelte";
import NothingMovedPopover from "../../components/NothingMovedPopover.svelte";
import Popover from "../../components/Popover.svelte";
import { api, isAppError } from "../../lib/api";
import type { DisabledReason } from "../../lib/game";
import type { Game } from "../../lib/generated/Game";
import type { RunKindDto } from "../../lib/generated/RunKindDto";
import { run } from "../../lib/run.svelte";
import { interpolate, strings } from "../../lib/strings";

type Props = {
  game: Game;
  disabled: DisabledReason | null;
  /** The Game after an adjust, to put in the editor. */
  onadjusted: (game: Game) => void;
  onopenoverlap: (axis: "turn" | "pitch") => void;
};

let { game, disabled, onadjusted, onopenoverlap }: Props = $props();

let popover = $state(false);
let adjustError = $state<string | null>(null);
/** Which step's `?` explanation is open. */
let help = $state<Tool | "overlapTurn" | "overlapPitch" | null>(null);

const s = strings.setup.calibrate;

type Tool = "turnTest" | "pitchTest" | "stepOnce";

/** Which tool the last run of this Game was, if it was a Test or Step once. */
const lastTool = $derived.by((): Tool | null => {
  const l = run.last;
  if (!l || l.gameId !== game.id) {
    return null;
  }
  return l.kind.kind === "turnTest" || l.kind.kind === "pitchTest" || l.kind.kind === "stepOnce"
    ? l.kind.kind
    : null;
});

/** The result line of the tool that ran (spec §11.2), or null. */
const resultLine = $derived.by((): { tool: Tool; text: string; amber: boolean } | null => {
  const l = run.last;
  const tool = lastTool;
  if (!l || !tool) {
    return null;
  }
  if (l.screenUnchanged) {
    return { tool, text: s.nothingMoved.detected, amber: true };
  }
  if (l.ended.by !== "finished" || l.ended.outcome.status === "interrupted") {
    return { tool, text: s.result.stopped, amber: false };
  }
  const text =
    tool === "turnTest"
      ? interpolate(s.result.turnTest, { n: l.steps })
      : tool === "pitchTest"
        ? interpolate(s.result.pitchTest, { n: l.steps })
        : s.result.stepOnce;
  return { tool, text, amber: false };
});

// The screen did not change: the checklist opens by itself.
$effect(() => {
  if (resultLine?.amber) {
    popover = true;
  }
});

const running = $derived(run.current?.gameId === game.id ? run.current : null);

function runningTool(kind: RunKindDto): string {
  switch (kind.kind) {
    case "turnTest":
      return s.turnTestTitle;
    case "pitchTest":
      return s.pitchTestTitle;
    case "stepOnce":
      return s.stepOnceTitle;
    case "checkOverlap":
      return kind.axis === "turn" ? s.overlapTurnTitle : s.overlapPitchTitle;
    case "session":
      return strings.detail.startSession;
  }
}

async function start(kind: RunKindDto) {
  await run.start(game.id, kind);
}

async function adjust(
  axis: "turn" | "pitch",
  observation: "overshot" | "undershot",
  magnitude: "fine" | "coarse",
) {
  adjustError = null;
  try {
    onadjusted(await api.games.adjustStep(game.id, axis, observation, magnitude));
  } catch (error) {
    adjustError = isAppError(error) || error instanceof Error ? error.message : String(error);
  }
}

const locked = $derived(disabled !== null || run.busy);
</script>

<section class="card calibrate">
  <div class="head">
    <h2 class="t-section">{s.title}</h2>
    <button type="button" class="link" onclick={() => (popover = !popover)}>{s.nothingMoved.link}</button>
    <NothingMovedPopover open={popover} onclose={() => (popover = false)} device={game.input.device} align="right" />
  </div>

  {#if running}
    <div class="running">
      <span>{interpolate(s.running, { tool: runningTool(running.kind) })}</span>
      <Button size="sm" variant="danger" onclick={() => run.stop()}>{s.stop}</Button>
    </div>
  {/if}

  {#snippet adjustRow(axis: "turn" | "pitch")}
    <div class="adjust">
      <span>{s.overshot}</span>
      <Button disabled={locked} onclick={() => adjust(axis, "overshot", "coarse")}>−10 %</Button>
      <Button disabled={locked} onclick={() => adjust(axis, "overshot", "fine")}>−2 %</Button>
      <Button disabled={locked} onclick={() => adjust(axis, "undershot", "fine")}>+2 %</Button>
      <Button disabled={locked} onclick={() => adjust(axis, "undershot", "coarse")}>+10 %</Button>
      <span>{s.undershot}</span>
    </div>
  {/snippet}

  {#snippet helpFor(tool: Tool | "overlapTurn" | "overlapPitch", text: string)}
    <span class="help">
      <HelpButton subtle label={text} on={help === tool} onclick={() => (help = help === tool ? null : tool)} />
      <Popover open={help === tool} onclose={() => (help = null)} align="right">
        <p class="helptext">{text}</p>
      </Popover>
    </span>
  {/snippet}

  {#snippet result(tool: Tool)}
    {#if resultLine?.tool === tool}
      <p class="result" class:amber={resultLine.amber}>
        {resultLine.text}
        <button type="button" class="link" onclick={() => (popover = true)}>{s.nothingMoved.link}</button>
      </p>
    {/if}
  {/snippet}

  <div class="step">
    <Badge n={1} />
    <div class="t">
      <div class="n">
        <b>{s.turnTestTitle}</b>
        {@render helpFor("turnTest", s.turnTest)}
        <Button size="sm" disabled={locked} onclick={() => start({ kind: "turnTest" })}>{s.run}</Button>
      </div>
      {@render adjustRow("turn")}
      {@render result("turnTest")}
    </div>
  </div>
  <div class="step">
    <Badge n={2} />
    <div class="t">
      <div class="n">
        <b>{s.overlapTurnTitle}</b>
        {@render helpFor("overlapTurn", strings.overlap.instruction)}
        <Button size="sm" disabled={locked} onclick={() => onopenoverlap("turn")}>{s.run}</Button>
      </div>
    </div>
  </div>
  <div class="sep"></div>
  <div class="step">
    <Badge n={3} />
    <div class="t">
      <div class="n">
        <b>{s.pitchTestTitle}</b>
        {@render helpFor("pitchTest", game.startPole === "nadir" ? s.pitchTest.nadir : s.pitchTest.zenith)}
        <Button size="sm" disabled={locked} onclick={() => start({ kind: "pitchTest" })}>{s.run}</Button>
      </div>
      {@render adjustRow("pitch")}
      {@render result("pitchTest")}
    </div>
  </div>
  <div class="step">
    <Badge n={4} />
    <div class="t">
      <div class="n">
        <b>{s.overlapPitchTitle}</b>
        {@render helpFor("overlapPitch", strings.overlap.instruction)}
        <Button size="sm" disabled={locked} onclick={() => onopenoverlap("pitch")}>{s.run}</Button>
      </div>
    </div>
  </div>
  <div class="sep"></div>
  <div class="once">
    <span>{s.stepOnceTitle}</span>
    <Button size="sm" disabled={locked} onclick={() => start({ kind: "stepOnce", axis: "turn" })}>{s.turn}</Button>
    <Button size="sm" disabled={locked} onclick={() => start({ kind: "stepOnce", axis: "pitch" })}>{s.pitch}</Button>
  </div>
  {@render result("stepOnce")}

  {#if disabled}
    <p class="t-tip">{strings.setup.start.disabled[disabled]}</p>
  {/if}
  {#if adjustError}
    <p class="error">{adjustError}</p>
  {/if}
</section>

<style>
  .calibrate {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  h2,
  p {
    margin: 0;
  }

  .running {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-radius: var(--rs);
    background: var(--lav);
    color: var(--lav-fg);
    font-size: 12.5px;
    font-weight: 500;
  }

  .step {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .t {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .n {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .n b {
    font-weight: 600;
  }

  .n .help {
    position: relative;
    display: inline-flex;
  }

  /* Run / Open sit at the right edge, with or without a `?`. */
  .n > :global(.btn) {
    margin-left: auto;
  }

  .helptext {
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
  }

  .adjust {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--muted);
  }

  .adjust :global(.btn) {
    height: 22px;
    min-width: 32px;
    padding: 1px 7px 0;
    border-radius: 6px;
    font-size: 11px;
  }

  .sep {
    height: 1px;
    background: var(--line);
  }

  .once {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .once span {
    color: var(--muted);
    font-size: 12px;
  }

  .result {
    font-size: 12px;
    color: var(--fg);
  }

  .result.amber {
    color: var(--warn-fg);
    background: var(--warn-soft);
    padding: 6px 10px;
    border-radius: var(--rs);
  }

  .link {
    border: 0;
    background: transparent;
    padding: 0;
    margin-left: 4px;
    color: var(--lav-fg);
    font-size: 12px;
    font-weight: 500;
  }

  .error {
    font-size: 11.5px;
    color: var(--bad-fg);
  }

  .head {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .head .link {
    margin: 0;
  }
</style>
