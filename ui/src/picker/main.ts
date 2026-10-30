// A Capture zone picker window's page (spec §11.4). Rust injects the
// window's label before the page loads; the page asks for its setup.
import "@fontsource-variable/inter";
import "../app.css";
import { mount } from "svelte";
import Picker from "./Picker.svelte";

declare global {
  interface Window {
    __PANOWWW_PICKER__?: string;
  }
}

const target = document.getElementById("picker");
if (!target) {
  throw new Error("picker.html has no #picker element");
}
// Outside Tauri, in dev, `?mock` shows the picker over a stand-in display.
const mock = import.meta.env.DEV && new URLSearchParams(location.search).has("mock");
mount(Picker, { target, props: { label: window.__PANOWWW_PICKER__ ?? "picker-1", mock } });
