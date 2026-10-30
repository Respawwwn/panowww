<script lang="ts">
// Calibrate (spec §11.2, §7.5): the four numbered tools with their adjust
// rows, Step once, the result line under the tool that ran, the amber
// line and the auto-opened checklist when the screen did not change, the
// permanent Nothing moved? link.
import Badge from "../../components/Badge.svelte";
import Button from "../../components/Button.svelte";
import NothingMovedPopover from "../../components/NothingMovedPopover.svelte";
import Tip from "../../components/Tip.svelte";
import { api } from "../../lib/api";
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
    adjustError = error instanceof Error ? error.message : String(error);
  }
}

const locked = $derived(disabled !== null || run.busy);
</script>

<section class="card calibrate">
  <h2 class="t-section">{s.title}</h2>
  <Tip text={s.countdown} />

  {#if running}
    <div class="running">
      <span>{interpolate(s.running, { tool: runningTool(running.kind) })}</span>
      <Button size="sm" variant="danger" onclick={() => run.stop()}>{s.stop}</Button>
    </div>
  {/if}

  {#snippet adjustRow(axis: "turn" | "pitch")}
    <div class="adjust">
      <span class="t-label">{s.overshot}</span>
      <Button size="sm" disabled={locked} onclick={() => adjust(axis, "overshot", "coarse")}>−10 %</Button>
      <Button size="sm" disabled={locked} onclick={() => adjust(axis, "overshot", "fine")}>−2 %</Button>
      <span class="gap"></span>
      <Button size="sm" disabled={locked} onclick={() => adjust(axis, "undershot", "fine")}>+2 %</Button>
      <Button size="sm" disabled={locked} onclick={() => adjust(axis, "undershot", "coarse")}>+10 %</Button>
      <span class="t-label">{s.undershot}</span>
    </div>
  {/snippet}

  {#snippet result(tool: Tool)}
    {#if resultLine?.tool === tool}
      <p class="result" class:amber={resultLine.amber}>
        {resultLine.text}
        <button type="button" class="link" onclick={() => (popover = true)}>{s.nothingMoved.link}</button>
      </p>
    {/if}
  {/snippet}

  <ol class="tools">
    <li>
      <div class="tool">
        <Badge n={1} />
        <b>{s.turnTestTitle}</b>
        <Button size="sm" disabled={locked} onclick={() => start({ kind: "turnTest" })}>{s.run}</Button>
      </div>
      {@render adjustRow("turn")}
      <Tip text={s.turnTest} />
      {@render result("turnTest")}
    </li>
    <li>
      <div class="tool">
        <Badge n={2} />
        <b>{s.overlapTurnTitle}</b>
        <Button size="sm" disabled={locked} onclick={() => onopenoverlap("turn")}>{s.open}</Button>
      </div>
      <Tip text={s.overlapTurn} />
    </li>
    <li>
      <div class="tool">
        <Badge n={3} />
        <b>{s.pitchTestTitle}</b>
        <Button size="sm" disabled={locked} onclick={() => start({ kind: "pitchTest" })}>{s.run}</Button>
      </div>
      {@render adjustRow("pitch")}
      <Tip text={game.startPole === "nadir" ? s.pitchTest.nadir : s.pitchTest.zenith} />
      {@render result("pitchTest")}
    </li>
    <li>
      <div class="tool">
        <Badge n={4} />
        <b>{s.overlapPitchTitle}</b>
        <Button size="sm" disabled={locked} onclick={() => onopenoverlap("pitch")}>{s.open}</Button>
      </div>
      <Tip text={s.overlapPitch} />
    </li>
  </ol>

  <div class="once">
    <b>{s.stepOnceTitle}</b>
    <Button size="sm" disabled={locked} onclick={() => start({ kind: "stepOnce", axis: "turn" })}>{s.turn}</Button>
    <Button size="sm" disabled={locked} onclick={() => start({ kind: "stepOnce", axis: "pitch" })}>{s.pitch}</Button>
  </div>
  <Tip text={s.stepOnce} />
  {@render result("stepOnce")}

  {#if disabled}
    <p class="t-tip">{strings.setup.start.disabled[disabled]}</p>
  {/if}
  {#if adjustError}
    <p class="error">{adjustError}</p>
  {/if}

  <div class="foot">
    <NothingMovedPopover open={popover} onclose={() => (popover = false)} device={game.input.device} />
    <button type="button" class="link" onclick={() => (popover = !popover)}>{s.nothingMoved.link}</button>
  </div>
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

  .tools {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .tools li {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .tool {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .tool b,
  .once b {
    flex: 1;
    font-weight: 600;
  }

  .adjust {
    display: flex;
    align-items: center;
    gap: 4px;
    padding-left: 34px;
  }

  .adjust .gap {
    width: 8px;
  }

  .once {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 4px;
    border-top: 1px solid var(--line);
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

  .foot {
    position: relative;
    margin-top: auto;
    padding-top: 4px;
  }

  .foot .link {
    margin: 0;
  }
</style>
