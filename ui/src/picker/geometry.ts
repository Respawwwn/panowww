// The rectangle maths of the Capture zone picker (spec §11.4), in display
// points: drawing from a corner, moving, resizing by a handle, clamping to
// the display, the X / Y / W / H fields.
import type { Rect } from "../lib/generated/Rect";

export type Point = { x: number; y: number };
export type Size = { w: number; h: number };
export type Handle = "nw" | "ne" | "sw" | "se";

/** Smallest zone that still makes sense; below it a drag is a click. */
export const MIN_SIDE = 8;

/** The rectangle between two corners, integer, never negative. */
export function between(a: Point, b: Point): Rect {
  const x1 = Math.round(Math.min(a.x, b.x));
  const y1 = Math.round(Math.min(a.y, b.y));
  const x2 = Math.round(Math.max(a.x, b.x));
  const y2 = Math.round(Math.max(a.y, b.y));
  return { x: x1, y: y1, w: x2 - x1, h: y2 - y1 };
}

/** Keeps the point on the display. */
export function clampPoint(p: Point, display: Size): Point {
  return {
    x: Math.min(Math.max(0, p.x), display.w),
    y: Math.min(Math.max(0, p.y), display.h),
  };
}

/** Keeps the rectangle inside the display, shrinking it when it is larger. */
export function clampRect(r: Rect, display: Size): Rect {
  const w = Math.min(Math.max(0, Math.round(r.w)), display.w);
  const h = Math.min(Math.max(0, Math.round(r.h)), display.h);
  const x = Math.min(Math.max(0, Math.round(r.x)), display.w - w);
  const y = Math.min(Math.max(0, Math.round(r.y)), display.h - h);
  return { x, y, w, h };
}

/** The corner opposite `handle`: what stays put while the handle moves. */
export function anchorOf(r: Rect, handle: Handle): Point {
  switch (handle) {
    case "nw":
      return { x: r.x + r.w, y: r.y + r.h };
    case "ne":
      return { x: r.x, y: r.y + r.h };
    case "sw":
      return { x: r.x + r.w, y: r.y };
    case "se":
      return { x: r.x, y: r.y };
  }
}

export function moved(r: Rect, dx: number, dy: number, display: Size): Rect {
  return clampRect({ ...r, x: r.x + dx, y: r.y + dy }, display);
}

/** Whether the rectangle is worth keeping after a drag. */
export function isUsable(r: Rect | null): r is Rect {
  return r !== null && r.w >= MIN_SIDE && r.h >= MIN_SIDE;
}

/** One field edited by hand: the others stay, the result stays on screen. */
export function withField(r: Rect, field: keyof Rect, raw: string, display: Size): Rect {
  const n = Number(raw);
  if (!Number.isFinite(n)) {
    return r;
  }
  const next = { ...r, [field]: Math.round(n) };
  if (field === "w" || field === "h") {
    next[field] = Math.max(MIN_SIDE, next[field]);
  }
  return clampRect(next, display);
}
