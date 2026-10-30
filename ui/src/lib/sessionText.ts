// Text of a Session row (spec §13.4): the failed template with its reason
// fragment, the state chip tone.
import type { RunErrorKind } from "./generated/RunErrorKind";
import type { SessionRecord } from "./generated/SessionRecord";
import type { SessionStatus } from "./generated/SessionStatus";
import { interpolate, strings } from "./strings";

export function reasonFragment(kind: RunErrorKind, message: string, display: string): string {
  const r = strings.sessions.failed.reason;
  switch (kind) {
    case "captureUnavailable":
      return r.captureUnavailable;
    case "displayGone":
      return interpolate(r.displayGone, { d: display });
    case "permissionRevoked":
      return r.permissionRevoked;
    case "shotFailed":
      return r.shotFailed;
    default:
      return message;
  }
}

/** `Stopped at Shot {n}: {reason} The {n−1} Shots taken are kept in the folder.` */
export function failedSessionText(record: SessionRecord): string {
  const taken = record.shotsTaken;
  const reason = record.error
    ? reasonFragment(record.error.kind, record.error.message, record.zone.displayId)
    : "";
  return interpolate(strings.sessions.failed.template, { n: taken + 1, reason, taken });
}

export function stateTone(status: SessionStatus): "ok" | "warn" | "bad" | "lav" {
  switch (status) {
    case "completed":
      return "ok";
    case "interrupted":
      return "warn";
    case "failed":
      return "bad";
    case "running":
      return "lav";
  }
}
