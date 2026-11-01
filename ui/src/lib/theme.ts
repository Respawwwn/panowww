// The window's appearance (spec §12): the OS appearance by default, or the
// light / dark tokens forced through `data-theme` on the document.
import type { Appearance } from "./generated/Appearance";

export function applyAppearance(
  appearance: Appearance,
  root: HTMLElement = document.documentElement,
): void {
  if (appearance === "auto") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", appearance);
  }
}
