<script lang="ts">
// Onboarding (spec §11.11): the two macOS permissions with their status
// and *Open System Settings*, the stuck line, the macOS 26 picker card,
// the gamepad card; the Windows variant with the administrator card and
// the ViGEmBus row. Polls the permissions while open; Continue needs
// both green.
import { onDestroy } from "svelte";
import Button from "../components/Button.svelte";
import Tip from "../components/Tip.svelte";
import WindowFrame from "../components/WindowFrame.svelte";
import { api, type PermissionKind, type Permissions } from "../lib/api";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

const platform = detectPlatform();
let permissions = $state<Permissions | null>(null);

async function refresh() {
  permissions = await api.onboarding.permissions().catch(() => null);
}

const timer = setInterval(() => void refresh(), 1000);
void refresh();
onDestroy(() => clearInterval(timer));

async function grant(kind: PermissionKind) {
  const granted = await api.onboarding.request(kind).catch(() => false);
  if (!granted) {
    await api.onboarding.openSystemSettings(kind).catch(() => {});
  }
  await refresh();
}

const ready = $derived(
  platform === "macos" ? Boolean(permissions?.accessibility && permissions?.screenRecording) : true,
);
const o = strings.onboarding;
const rows: { kind: PermissionKind; row: { name: string; ok: string; missing: string } }[] = [
  { kind: "accessibility", row: o.mac.accessibility },
  { kind: "screenRecording", row: o.mac.screenRecording },
];
</script>

<WindowFrame title={o.title} width={560} onclose={() => windows.close()}>
  {#if platform === "macos"}
    <div class="intro">
      <h3 class="t-section">{o.mac.subtitle}</h3>
      <p class="t-tip">{o.mac.intro}</p>
    </div>
    {#each rows as { kind, row } (kind)}
      {@const granted = permissions?.[kind] ?? false}
      <div class="perm">
        <span class="dot" class:ok={granted} class:bad={!granted}></span>
        <div class="t">
          <b>{row.name}</b>
          <span class="s">{granted ? row.ok : row.missing}</span>
        </div>
        {#if !granted}
          <Button size="sm" onclick={() => grant(kind)}>{strings.countdown.error.openSystemSettings}</Button>
        {/if}
      </div>
    {/each}
    <Tip text={o.mac.stuck} />
    <Tip kind="lav" text={o.mac.picker} />
    <Tip kind="lav" text={o.mac.gamepad} />
  {:else}
    <div class="intro">
      <h3 class="t-section">{o.win.subtitle}</h3>
    </div>
    <Tip kind="lav" text={o.win.admin} />
    <div class="perm">
      <span class="dot" class:ok={permissions?.vigem} class:bad={!permissions?.vigem}></span>
      <div class="t">
        <b>ViGEmBus</b>
        <span class="s">{o.win.vigem}</span>
      </div>
      <Button size="sm" onclick={() => api.onboarding.openUrl("https://github.com/nefarius/ViGEmBus/releases/tag/v1.22.0")}>
        {o.win.installVigem}
      </Button>
    </div>
    <Tip text={o.win.vigemTip} />
  {/if}
  {#snippet footer()}
    <Button variant="ghost" onclick={() => windows.close()}>{o.later}</Button>
    <span class="spacer"></span>
    <Button variant="primary" disabled={!ready} onclick={() => windows.close()}>{o.continue}</Button>
  {/snippet}
</WindowFrame>

<style>
  .intro h3,
  .intro p {
    margin: 0;
  }

  .intro {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .perm {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: var(--rs);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }

  .dot.ok {
    background: var(--ok);
  }

  .dot.bad {
    background: var(--bad);
  }

  .t {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .t b {
    font-weight: 600;
  }

  .s {
    font-size: 12px;
    color: var(--muted);
  }

  .spacer {
    flex: 1;
  }
</style>
