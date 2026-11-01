<script lang="ts">
// Onboarding (spec §11.11): the two macOS permissions with their status
// and *Open System Settings*, the stuck line, the macOS 26 picker card,
// the gamepad card; the Windows variant with the administrator card and
// the ViGEmBus row. Polls the permissions while open; Continue needs
// both green.
import { onDestroy } from "svelte";
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Mark from "../components/Mark.svelte";
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
    <div class="head">
      <Mark size={56} radius={13} />
      <div class="intro">
        <h2>{o.mac.subtitle}</h2>
        <p class="sub">{o.mac.intro}</p>
      </div>
    </div>
    {#each rows as { kind, row } (kind)}
      {@const granted = permissions?.[kind] ?? false}
      <div class="perm">
        <span class="dot" class:ok={granted} class:bad={!granted}></span>
        <div class="t">
          <b>{row.name}</b>
          <span class="s">{granted ? row.ok : row.missing}</span>
        </div>
        <Button size="sm" disabled={granted} onclick={() => grant(kind)}>
          {strings.countdown.error.openSystemSettings}
        </Button>
      </div>
    {/each}
    <Tip text={o.mac.stuck} />
    <div class="card hint">
      <Chip tone="lav">{o.mac.picker.chip}</Chip>
      <div>
        <b>{o.mac.picker.title}</b>
        <p class="sub">{o.mac.picker.body}</p>
      </div>
    </div>
    <div class="card hint">
      <Chip tone="warn">{o.mac.gamepad.chip}</Chip>
      <div>
        <b>{o.mac.gamepad.title}</b>
        <p class="sub">{o.mac.gamepad.body}</p>
      </div>
    </div>
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
  .head {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .head h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .sub {
    margin: 0;
    color: var(--muted);
    font-size: 12.5px;
  }

  .hint {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    font-size: 12.5px;
  }

  .hint :global(.chip) {
    flex: none;
  }

  .hint b {
    font-weight: 600;
  }

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
