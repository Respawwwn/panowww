// Which screen a run puts the window on (spec §11.5 to §11.7): the
// countdown for every run kind, the running view, the end screen for a
// Session, back to the Game for everything else. Pure, so it is tested
// with a scripted event stream.
import type { Screen } from "./nav.svelte";
import type { CurrentRun, LastRun } from "./run.svelte";

/** The screen to show for the run state, or null to leave it as is. */
export function screenForRun(
  current: CurrentRun | null,
  last: LastRun | null,
  screen: Screen,
): Screen | null {
  if (current) {
    const gameId = current.gameId;
    if (current.phase === "countdown") {
      return screen.kind === "countdown" && screen.gameId === gameId
        ? null
        : { kind: "countdown", gameId };
    }
    return screen.kind === "running" && screen.gameId === gameId
      ? null
      : { kind: "running", gameId };
  }
  if (!last) {
    return null;
  }
  if (last.ended.by === "cannotStart") {
    // The countdown screen shows the error state until Back, wherever
    // the request came from.
    return screen.kind === "countdown" ? null : { kind: "countdown", gameId: last.gameId };
  }
  const onRunScreen = screen.kind === "countdown" || screen.kind === "running";
  if (!onRunScreen) {
    return null;
  }
  switch (last.ended.by) {
    case "cancelled":
      return { kind: "game", id: last.gameId, tab: "setup" };
    case "finished":
      return last.kind.kind === "session"
        ? { kind: "ended", gameId: last.gameId, folder: last.ended.folder }
        : { kind: "game", id: last.gameId, tab: "setup" };
  }
}
