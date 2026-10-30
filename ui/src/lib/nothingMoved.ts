// The Nothing moved? checklist (spec §11.10, §13.6): five numbered items
// filtered by the Game's Input device and the OS.
import type { Device } from "./generated/Device";
import type { Platform } from "./shortcut";
import { strings } from "./strings";

export function nothingMovedItems(device: Device, platform: Platform): string[] {
  const n = strings.nothingMoved;
  const deviceLine = { keyboard: n.keyboard, mouse: n.mouse, gamepad: n.gamepad }[device];
  const osLine = platform === "macos" ? n.mac.controllerMouse : n.win.rawInput;
  return [n.fullscreen, n.focus, deviceLine, osLine, n.antiCheat];
}
