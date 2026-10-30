<script lang="ts">
// Component gallery (issue 14 acceptance): every component in both themes,
// side by side, at `?gallery`. Not part of the app's navigation.
import Badge from "../components/Badge.svelte";
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import CollapsibleCard from "../components/CollapsibleCard.svelte";
import DeviceIcon from "../components/DeviceIcon.svelte";
import Field from "../components/Field.svelte";
import HelpButton from "../components/HelpButton.svelte";
import InlineConfirm from "../components/InlineConfirm.svelte";
import KeyCaps from "../components/KeyCaps.svelte";
import PillTabs from "../components/PillTabs.svelte";
import Popover from "../components/Popover.svelte";
import PreviewCard from "../components/PreviewCard.svelte";
import ProgressBar from "../components/ProgressBar.svelte";
import Segmented from "../components/Segmented.svelte";
import StatusBox from "../components/StatusBox.svelte";
import Switch from "../components/Switch.svelte";
import Thumbnail from "../components/Thumbnail.svelte";
import Tip from "../components/Tip.svelte";
import { defaultSettings } from "../lib/settings.svelte";
import { strings } from "../lib/strings";

let tab = $state("setup");
let seg = $state("mouse");
let open = $state(1);
let checked = $state(true);
let confirming = $state(false);
let popover = $state(false);
let preview = $state(true);
let platform = $state<"macos" | "windows">("macos");

const themes = ["light", "dark"] as const;
</script>

<div class="gallery" data-platform={platform === "windows" ? "windows" : undefined}>
  <header class="top">
    <h1 class="t-window">Panowww components</h1>
    <Segmented
      items={[
        { id: "macos", label: "macOS radii" },
        { id: "windows", label: "Windows radii" },
      ]}
      value={platform}
      onchange={(id) => (platform = id as "macos" | "windows")}
    />
  </header>

  <div class="columns">
    {#each themes as theme (theme)}
      <section class="theme" data-theme={theme}>
        <h2 class="t-overline">{theme}</h2>

        <h3 class="t-section">Typography</h3>
        <div class="stack">
          <p class="t-title">Title 22 / 700</p>
          <p class="t-window">Window title 18 / 700</p>
          <p class="t-section">Section 14 / 600</p>
          <p class="t-body">Body 13 / 400 — the Shots of a 360° panorama, 78 Shots, 6 Rows × 13.</p>
          <p class="t-label">Label 12 / 500</p>
          <p class="t-tip">Tip 11.5 / 400</p>
          <p class="t-overline">Overline 11 / 600</p>
          <p class="t-counter">26 / 700 · 17 / 78</p>
        </div>

        <h3 class="t-section">Buttons</h3>
        <div class="line">
          <Button variant="primary" size="lg">{strings.detail.startSession}</Button>
          <Button variant="primary">Primary</Button>
          <Button>Secondary</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="danger">Danger</Button>
          <Button size="sm">Small</Button>
          <Button disabled>Disabled</Button>
          <HelpButton label="Why did it fail?" on={popover} onclick={() => (popover = !popover)} />
        </div>

        <h3 class="t-section">Pill tabs · segmented · switch</h3>
        <div class="line">
          <PillTabs
            items={[
              { id: "setup", label: strings.detail.setup },
              { id: "sessions", label: strings.detail.sessions },
            ]}
            value={tab}
            onchange={(id) => (tab = id)}
          />
          <Segmented
            items={[
              { id: "keyboard", label: "Keyboard" },
              { id: "mouse", label: "Mouse" },
              { id: "gamepad", label: "Gamepad" },
            ]}
            value={seg}
            onchange={(id) => (seg = id)}
          />
          <Switch {checked} label={strings.settings.checkForUpdates} onchange={(v) => (checked = v)} />
        </div>

        <h3 class="t-section">Chips · badges · key caps · icons</h3>
        <div class="line">
          <Chip tone="ok">{strings.sessions.state.completed}</Chip>
          <Chip tone="warn">{strings.sessions.state.interrupted}</Chip>
          <Chip tone="bad">{strings.sessions.state.failed}</Chip>
          <Chip>30 % Overlap</Chip>
          <Badge n={1} />
          <Badge n={2} />
          <KeyCaps shortcut={defaultSettings.shortcuts.start} {platform} />
          <KeyCaps shortcut={{ modifiers: ["meta", "alt"], code: "ArrowUp" }} {platform} />
          <span class="icons">
            <DeviceIcon device="keyboard" />
            <DeviceIcon device="mouse" />
            <DeviceIcon device="gamepad" />
          </span>
        </div>

        <h3 class="t-section">Status box · tips</h3>
        <div class="stack">
          <StatusBox tone="ok">Display 1 · 2560 × 1440 at (0, 0)</StatusBox>
          <StatusBox tone="bad">{strings.setup.zone.notSet}</StatusBox>
          <Tip text={strings.setup.grid.waitAfterMove} />
          <Tip text={strings.setup.start.shortcutStart} />
          <Tip kind="lav" text={strings.overlap.tip} />
          <Tip kind="warn" text={strings.setup.zone.displayChanged} values={{ n: 2 }} />
          <Tip kind="ok" text={strings.overlap.verdict.ok} values={{ p: 30 }} />
        </div>

        <h3 class="t-section">Collapsible card · field</h3>
        <div class="stack">
          <CollapsibleCard
            n={1}
            title={strings.setup.game.title}
            summary="Mouse · drag"
            open={open === 1}
            ontoggle={() => (open = open === 1 ? 0 : 1)}
          >
            <Field label={strings.setup.game.name} id="g-name">
              <input id="g-name" class="wide" value="Elden Ring – photo mode" />
            </Field>
            <Field label="Turn delta" id="g-turn" tip={strings.setup.input.mouse.crossOs}>
              <input id="g-turn" type="number" value="840" />
              <span class="unit">units</span>
            </Field>
            <Field label="Rows" id="g-rows" error="Rows must be between 2 and 36.">
              <input id="g-rows" type="number" value="1" />
            </Field>
          </CollapsibleCard>
          <CollapsibleCard
            n={2}
            title={strings.setup.grid.title}
            summary="13 × 6 · 300 / 300 ms"
            open={open === 2}
            ontoggle={() => (open = open === 2 ? 0 : 2)}
          >
            <Tip text={strings.setup.grid.shotsPerRow} />
          </CollapsibleCard>
        </div>

        <h3 class="t-section">Progress · thumbnails · preview</h3>
        <div class="stack">
          <ProgressBar value={17} max={78} />
          <div class="thumbs">
            <Thumbnail label="r01_c01" src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='9'%3E%3Crect width='16' height='9' fill='%236454b5'/%3E%3C/svg%3E" />
            <Thumbnail label="r01_c02" state="now" />
            <Thumbnail label="r01_c03" state="todo" />
            <Thumbnail label="r01_c04" state="todo" />
          </div>
          <div class="preview">
            <PreviewCard
              src={preview
                ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='9'%3E%3Crect width='16' height='9' fill='%231b1740'/%3E%3C/svg%3E"
                : null}
              displayW={2560}
              displayH={1440}
              zone={{ x: 320, y: 180, w: 1920, h: 1080 }}
              on={preview}
              ontoggle={(on) => (preview = on)}
            />
          </div>
        </div>

        <h3 class="t-section">Table · inline confirmation · popover</h3>
        <table class="tbl">
          <thead>
            <tr>
              <th>{strings.sessions.columns.when}</th>
              <th>{strings.sessions.columns.state}</th>
              <th>{strings.sessions.columns.shots}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>Today, 15:04</td>
              <td><Chip tone="ok">{strings.sessions.state.completed}</Chip></td>
              <td class="num">78 / 78</td>
              <td>
                <InlineConfirm
                  {confirming}
                  question={strings.sessions.deleteConfirm}
                  confirmLabel={strings.sessions.delete}
                  cancelLabel={strings.sessions.cancel}
                  onconfirm={() => (confirming = false)}
                  oncancel={() => (confirming = false)}
                >
                  <Button size="sm">{strings.sessions.zip}</Button>
                  <Button size="sm">{strings.sessions.reveal}</Button>
                  <Button size="sm" variant="danger" onclick={() => (confirming = true)}>
                    {strings.sessions.delete}
                  </Button>
                </InlineConfirm>
              </td>
            </tr>
            <tr>
              <td>Yesterday, 21:40</td>
              <td>
                <span class="anchor">
                  <Chip tone="bad">{strings.sessions.state.failed}</Chip>
                  <HelpButton label={strings.nothingMoved.title} on={popover} onclick={() => (popover = !popover)} />
                  <Popover open={popover} onclose={() => (popover = false)}>
                    <b>{strings.nothingMoved.title}</b>
                    <p>{strings.nothingMoved.intro}</p>
                    <ol>
                      <li>{strings.nothingMoved.fullscreen}</li>
                      <li>{strings.nothingMoved.focus}</li>
                      <li>{strings.nothingMoved.mouse}</li>
                      <li>{strings.nothingMoved.mac.controllerMouse}</li>
                      <li>{strings.nothingMoved.antiCheat}</li>
                    </ol>
                  </Popover>
                </span>
              </td>
              <td class="num">46 / 78</td>
              <td></td>
            </tr>
          </tbody>
        </table>
      </section>
    {/each}
  </div>
</div>

<style>
  .gallery {
    height: 100%;
    overflow: auto;
    background: var(--content);
  }

  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid var(--line);
  }

  h1,
  h2,
  h3,
  p {
    margin: 0;
  }

  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .theme {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 24px;
    background: var(--content);
    color: var(--fg);
  }

  .theme h3 {
    margin-top: 12px;
  }

  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .icons {
    display: inline-flex;
    gap: 8px;
    color: var(--muted);
  }

  .thumbs {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
  }

  .preview {
    max-width: 360px;
  }

  .anchor {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .anchor ol {
    margin: 6px 0 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
