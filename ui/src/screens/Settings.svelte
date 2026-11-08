<script lang="ts">
// Settings (spec §11.12): Sessions folder, the two shortcuts with their
// recorders, New Game defaults, the Display preview switch, General.
// Every change is saved at once; a shortcut the OS refuses shows its
// error next to the recorder.
import { onMount } from "svelte";
import Button from "../components/Button.svelte";
import Field from "../components/Field.svelte";
import Segmented from "../components/Segmented.svelte";
import ShortcutRecorder from "../components/ShortcutRecorder.svelte";
import Switch from "../components/Switch.svelte";
import Tip from "../components/Tip.svelte";
import WindowFrame from "../components/WindowFrame.svelte";
import { api, isAppError } from "../lib/api";
import type { Settings } from "../lib/generated/Settings";
import { defaultSettings, settings } from "../lib/settings.svelte";
import { detectPlatform } from "../lib/shortcut";
import { strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

const platform = detectPlatform();
let shortcutError = $state<string | null>(null);
/** Under which recorder the error sits: the one that was changed. */
let shortcutErrorFor = $state<"start" | "stop">("stop");
let saveError = $state<string | null>(null);
/** Resolved default Sessions folder; *Reset* shows when the setting differs. */
let defaultFolder = $state<string | null>(null);

// No Session starts from the combo being recorded (spec §10).
onMount(() => {
  void api.settings
    .defaultSessionsFolder()
    .then((folder) => {
      defaultFolder = folder;
    })
    .catch(() => {});
  void api.settings.setShortcutsEnabled(false).catch(() => {});
  return () => {
    void api.settings.setShortcutsEnabled(true).catch(() => {});
  };
});

const s = $derived(settings.value);
const folderOverridden = $derived(defaultFolder !== null && s.sessionsFolder !== defaultFolder);

async function save(patch: Partial<Settings>, which: "start" | "stop" = "stop") {
  const next = { ...settings.value, ...patch };
  const shortcutsChanged = patch.shortcuts !== undefined;
  if (shortcutsChanged) {
    shortcutErrorFor = which;
  }
  try {
    await settings.save(next);
    saveError = null;
    if (shortcutsChanged) {
      shortcutError = null;
    }
  } catch (error) {
    const message = isAppError(error) ? error.message : String(error);
    if (shortcutsChanged) {
      shortcutError = message;
    } else {
      saveError = message;
    }
  }
}

async function chooseFolder() {
  const folder = await api.settings.chooseSessionsFolder().catch(() => null);
  if (folder) {
    await save({ sessionsFolder: folder });
  }
}

function number(value: string): number {
  const n = Number(value);
  return Number.isFinite(n) ? n : 0;
}
</script>

<WindowFrame title={strings.settings.title} width={680} onclose={() => windows.close()}>
  <section class="card short">
    <h3>{strings.settings.general}</h3>
    <Field label={strings.settings.appearance.label}>
      <Segmented
        items={[
          { id: "auto", label: strings.settings.appearance.auto },
          { id: "light", label: strings.settings.appearance.light },
          { id: "dark", label: strings.settings.appearance.dark },
        ]}
        value={s.appearance}
        onchange={(id) => save({ appearance: id as Settings["appearance"] })}
      />
    </Field>
    <Field label={strings.settings.folderLabel} id="s-folder" tip={strings.settings.folder}>
      <input
        id="s-folder"
        class="wide"
        type="text"
        readonly
        value={s.sessionsFolder}
        title={s.sessionsFolder}
        ondblclick={chooseFolder}
      />
      <Button size="sm" onclick={chooseFolder}>{strings.settings.change}</Button>
      <Button size="sm" onclick={() => api.settings.revealPath(s.sessionsFolder)}>{strings.settings.reveal}</Button>
      {#if folderOverridden && defaultFolder}
        {@const folder = defaultFolder}
        <Button size="sm" variant="ghost" onclick={() => save({ sessionsFolder: folder })}>{strings.settings.reset}</Button>
      {/if}
    </Field>
    <Switch
      checked={s.displayPreview}
      label={strings.settings.displayPreview.label}
      onchange={(on) => save({ displayPreview: on })}
    />
    <Tip text={strings.settings.displayPreview.hint} />
  </section>

  <section class="card short">
    <h3>{strings.settings.shortcutsTitle}</h3>
    {#if platform === "macos"}
      <Tip text={strings.settings.shortcutsMac} />
    {/if}
    <Field label={strings.settings.start}>
      <ShortcutRecorder
        shortcut={s.shortcuts.start}
        defaultShortcut={defaultSettings.shortcuts.start}
        onchange={(start) => save({ shortcuts: { ...s.shortcuts, start } }, "start")}
        error={shortcutErrorFor === "start" ? shortcutError : null}
      />
    </Field>
    <Field label={strings.settings.stop} tip={strings.settings.shortcuts}>
      <ShortcutRecorder
        shortcut={s.shortcuts.stop}
        defaultShortcut={defaultSettings.shortcuts.stop}
        onchange={(stop) => save({ shortcuts: { ...s.shortcuts, stop } }, "stop")}
        error={shortcutErrorFor === "stop" ? shortcutError : null}
      />
    </Field>
  </section>

  <section class="card">
    <h3>{strings.settings.defaultsTitle}</h3>
    <Field label={strings.setup.grid.startPoleLabel}>
      <Segmented
        items={[
          { id: "nadir", label: strings.setup.grid.nadir },
          { id: "zenith", label: strings.setup.grid.zenith },
        ]}
        value={s.newGameDefaults.startPole}
        onchange={(id) => save({ newGameDefaults: { ...s.newGameDefaults, startPole: id as "nadir" | "zenith" } })}
      />
    </Field>
    <Field label={strings.setup.grid.waitAfterMoveLabel} id="d-wait-move">
      <input
        id="d-wait-move"
        type="number"
        min="0"
        max="10000"
        step="50"
        value={s.newGameDefaults.waitAfterMoveMs}
        onchange={(e) => save({ newGameDefaults: { ...s.newGameDefaults, waitAfterMoveMs: number(e.currentTarget.value) } })}
      />
      <span class="unit">ms</span>
    </Field>
    <Field label={strings.setup.grid.waitAfterShotLabel} id="d-wait-shot" tip={strings.settings.defaults}>
      <input
        id="d-wait-shot"
        type="number"
        min="0"
        max="10000"
        step="50"
        value={s.newGameDefaults.waitAfterShotMs}
        onchange={(e) => save({ newGameDefaults: { ...s.newGameDefaults, waitAfterShotMs: number(e.currentTarget.value) } })}
      />
      <span class="unit">ms</span>
    </Field>
  </section>

  <section class="card">
    <h3>{strings.settings.updatesTitle}</h3>
    <Switch
      checked={s.checkForUpdates}
      label={strings.settings.checkForUpdates}
      onchange={(on) => save({ checkForUpdates: on })}
    />
    <Switch
      checked={s.includePrereleases}
      label={strings.settings.includePrereleases.label}
      disabled={!s.checkForUpdates}
      onchange={(on) => save({ includePrereleases: on })}
    />
    <Tip text={strings.settings.includePrereleases.hint} />
  </section>

  <div class="actions">
    <Button size="sm" onclick={() => windows.show("onboarding")}>{strings.settings.reviewPermissions}</Button>
    <Button size="sm" variant="ghost" onclick={() => api.settings.revealLogs()}>{strings.settings.revealLogs}</Button>
  </div>

  {#if saveError}
    <p class="error">{saveError}</p>
  {/if}
</WindowFrame>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
  }

  /* Short labels (Appearance, Folder; Start, Stop): a narrower column
     leaves the folder line room for Change, Reveal and Reset. New Game
     defaults keep 124 px for their long labels. */
  .short :global(.row > label) {
    width: 84px;
  }

  h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 8px;
    padding-top: 2px;
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
