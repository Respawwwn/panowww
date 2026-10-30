import "@fontsource-variable/inter";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

async function start(target: HTMLElement) {
  // Outside Tauri, in dev, `?mock` stands in for the Rust side.
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("mock")) {
    const { installMock } = await import("./lib/mock");
    installMock();
  }
  mount(App, { target });
}

const target = document.getElementById("app");
if (!target) {
  throw new Error("index.html has no #app element");
}
void start(target);
