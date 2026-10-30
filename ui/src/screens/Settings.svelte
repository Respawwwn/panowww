<script lang="ts">
// Settings (spec §11.12): Sessions folder, the two shortcuts with their
// recorders, New Game defaults, the Display preview switch, General.
// Every change is saved at once; a shortcut the OS refuses shows its
// error next to the recorder.
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
let saveError = $state<string | null>(null);

const s = $derived(settings.value);

async function save(patch: Partial<Settings>) {
  const next = { ...settings.value, ...patch };
  const shortcutsChanged = patch.shortcuts !== undefined;
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
  <section>
    <h3 class="t-overline">{strings.settings.sessionsTitle}</h3>
    <Field label={strings.settings.sessionsFolder} tip={strings.settings.folder}>
      <span class="path" title={s.sessionsFolder}>{s.sessionsFolder}</span>
      <Button size="sm" onclick={chooseFolder}>{strings.settings.change}</Button>
      <Button size="sm" variant="ghost" onclick={() => api.settings.revealPath(s.sessionsFolder)}>{strings.settings.reveal}</Button>
    </Field>
  </section>

  <section>
    <h3 class="t-overline">{strings.settings.shortcutsTitle}</h3>
    <Field label={strings.settings.start}>
      <ShortcutRecorder
        shortcut={s.shortcuts.start}
        defaultShortcut={defaultSettings.shortcuts.start}
        onchange={(start) => save({ shortcuts: { ...s.shortcuts, start } })}
      />
    </Field>
    <Field label={strings.settings.stop} tip={strings.settings.shortcuts}>
      <ShortcutRecorder
        shortcut={s.shortcuts.stop}
        defaultShortcut={defaultSettings.shortcuts.stop}
        onchange={(stop) => save({ shortcuts: { ...s.shortcuts, stop } })}
        error={shortcutError}
      />
    </Field>
    {#if platform === "macos"}
      <Tip text={strings.settings.shortcutsMac} />
    {/if}
  </section>

  <section>
    <h3 class="t-overline">{strings.settings.defaultsTitle}</h3>
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

  <section>
    <h3 class="t-overline">{strings.settings.previewTitle}</h3>
    <Switch
      checked={s.displayPreview}
      label={strings.settings.displayPreview.label}
      onchange={(on) => save({ displayPreview: on })}
    />
    <Tip text={strings.settings.displayPreview.hint} />
  </section>

  <section>
    <h3 class="t-overline">{strings.settings.general}</h3>
    <Switch
      checked={s.checkForUpdates}
      label={strings.settings.checkForUpdates}
      onchange={(on) => save({ checkForUpdates: on })}
    />
    <div class="actions">
      <Button size="sm" onclick={() => windows.show("onboarding")}>{strings.settings.reviewPermissions}</Button>
      <Button size="sm" variant="ghost" onclick={() => api.settings.revealLogs()}>{strings.settings.revealLogs}</Button>
    </div>
  </section>

  {#if saveError}
    <p class="error">{saveError}</p>
  {/if}
</WindowFrame>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--line);
  }

  section:last-of-type {
    border-bottom: 0;
  }

  h3 {
    margin: 0;
  }

  .path {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    display: flex;
    gap: 6px;
  }

  .error {
    margin: 0;
    font-size: 11.5px;
    color: var(--bad-fg);
  }
</style>
