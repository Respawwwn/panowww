// The overlay window's page (spec §10): transparent, drawn over the game,
// fed by the `overlay` event. Nothing here is clickable or focusable.
import "@fontsource-variable/inter";
import { mount } from "svelte";
import Overlay from "./Overlay.svelte";

const target = document.getElementById("overlay");
if (!target) {
  throw new Error("overlay.html has no #overlay element");
}
// Outside Tauri, in dev, `?mock` cycles through the views over a dark
// stand-in for the game.
const mock = import.meta.env.DEV && new URLSearchParams(location.search).has("mock");
mount(Overlay, { target, props: { mock } });
