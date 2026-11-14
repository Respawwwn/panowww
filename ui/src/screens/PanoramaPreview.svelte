<script lang="ts">
// The Panorama preview window (spec §16.4, board StitchDone): Photo
// Sphere Viewer over the dimmed app, fed by `panorama_preview` as a data
// URL; drag to look around, scroll to zoom, arrow keys, a button back to
// *Fill the window*; under the viewer the path in code and the field of
// view; *Reveal* and *Done* in the footer.
// Without WebGL 2 the viewer throws at start: the flat preview image is
// shown with a line saying so.
import { Viewer } from "@photo-sphere-viewer/core";
import "@photo-sphere-viewer/core/index.css";
import Button from "../components/Button.svelte";
import Chip from "../components/Chip.svelte";
import Icon from "../components/Icon.svelte";
import WindowFrame from "../components/WindowFrame.svelte";
import { api, isAppError } from "../lib/api";
import { formatWhen } from "../lib/format";
import { preview } from "../lib/preview.svelte";
import { detectPlatform } from "../lib/shortcut";
import { interpolate, strings } from "../lib/strings";
import { windows } from "../lib/windows.svelte";

const platform = detectPlatform();
const s = strings.stitch;
const sep = platform === "windows" ? "\\" : "/";

const subject = $derived(preview.subject);
let image = $state<string | null>(null);
let loadError = $state<string | null>(null);
let container = $state<HTMLDivElement | null>(null);
let viewer: Viewer | null = null;

// The preview image, read once per Panorama.
$effect(() => {
  const folder = subject?.folder;
  image = null;
  loadError = null;
  if (!folder) {
    return;
  }
  api.stitch
    .preview(folder)
    .then((url) => {
      image = url;
    })
    .catch((e) => {
      loadError = isAppError(e) ? e.message : String(e);
    });
});

// The viewer lives as long as the container and the image do: the
// fallback swaps the container out, which ends it; *Fill the window* only
// resizes it. Starting it without WebGL 2 throws: the window falls back to
// the flat image.
$effect(() => {
  const el = container;
  const url = image;
  if (!el || !url) {
    return;
  }
  try {
    viewer = new Viewer({
      container: el,
      panorama: url,
      navbar: false,
      keyboard: "always",
      mousewheel: true,
      moveInertia: true,
      defaultYaw: 0,
      defaultPitch: 0,
      loadingTxt: "",
      canvasBackground: "#0d0b1c",
      // The data URL carries no XMP; the image is the full sphere.
      panoData: (img) => ({
        fullWidth: img.width,
        fullHeight: img.height,
        croppedWidth: img.width,
        croppedHeight: img.height,
        croppedX: 0,
        croppedY: 0,
      }),
    });
    viewer.addEventListener("panorama-error", (event) => {
      console.warn("Panorama preview: the viewer could not load the image", event.error);
      preview.viewerFailed();
    });
  } catch (error) {
    console.warn("Panorama preview: the viewer could not start", error);
    viewer = null;
    preview.viewerFailed();
  }
  return () => {
    viewer?.destroy();
    viewer = null;
  };
});

function close() {
  preview.close();
  windows.close();
}

function reveal() {
  if (subject) {
    void api.sessions.reveal(`${subject.folder}${sep}${subject.entry.file}`).catch(() => {});
  }
}

const path = $derived(subject ? `${subject.folder}${sep}${subject.entry.file}` : "");
const fov = $derived(
  subject ? interpolate(s.preview.fov, { deg: subject.entry.fieldOfViewDeg.toFixed(1) }) : "",
);
const when = $derived(subject?.startedAt ? formatWhen(subject.startedAt, strings.sessions) : "");
</script>

{#if subject}
  <WindowFrame title={s.preview.title} width={1040} height={680} fill={preview.fill} onclose={close}>
    {#snippet aside()}
      <span class="who">{subject.gameName}</span>
      <Chip tone="ok"><Icon name="check" size={13} />{interpolate(s.preview.chip, { when: formatWhen(subject.entry.stitchedAt, strings.sessions) })}</Chip>
    {/snippet}
    <div class="viewer" class:flat={preview.fallback}>
      {#if preview.fallback && image}
        <img class="fallback" src={image} alt={s.preview.title} />
        <p class="nowebgl">{s.preview.noWebgl}</p>
      {:else if loadError}
        <p class="nowebgl">{loadError}</p>
      {:else}
        <div class="psv" bind:this={container}></div>
      {/if}
      <div class="hud">
        <button type="button" class="hudbtn" onclick={() => preview.toggleFill()}>
          <Icon name="crop" size={14} />
          {s.actions.fill}
        </button>
      </div>
      {#if !preview.fallback}
        <div class="hint">{s.preview.hint}</div>
      {/if}
    </div>
    <div class="meta">
      <code title={path}>{path}</code>
      <span class="sub fov">{fov}</span>
    </div>
    {#snippet footer()}
      <Button size="lg" onclick={reveal}>
        <Icon name="folder" />
        {platform === "macos" ? strings.detail.revealInFinder : strings.detail.revealInExplorer}
      </Button>
      <span class="spacer"></span>
      <Button size="lg" variant="primary" onclick={close}>{s.actions.done}</Button>
    {/snippet}
  </WindowFrame>
{/if}

<style>
  .who {
    font-size: 12px;
    font-weight: 500;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .viewer {
    position: relative;
    flex: 1;
    min-height: 0;
    border-radius: 12px;
    overflow: hidden;
    background: #0d0b1c;
    border: 1px solid var(--line);
  }

  .psv {
    position: absolute;
    inset: 0;
  }

  .fallback {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }

  .nowebgl {
    position: absolute;
    left: 12px;
    right: 12px;
    top: 12px;
    margin: 0;
    padding: 8px 10px;
    border-radius: 999px;
    background: rgb(13 11 28 / 72%);
    color: rgb(255 255 255 / 85%);
    font-size: 11.5px;
    text-align: center;
    backdrop-filter: blur(6px);
  }

  .hud {
    position: absolute;
    left: 12px;
    bottom: 12px;
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .hudbtn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid rgb(255 255 255 / 18%);
    background: rgb(13 11 28 / 72%);
    color: #fff;
    font: inherit;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1;
    backdrop-filter: blur(6px);
    cursor: pointer;
  }

  .hudbtn:hover {
    background: rgb(13 11 28 / 88%);
  }

  .hint {
    position: absolute;
    right: 12px;
    bottom: 12px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    padding: 0 10px;
    border-radius: 999px;
    background: rgb(13 11 28 / 72%);
    color: rgb(255 255 255 / 85%);
    font-size: 11.5px;
    font-weight: 500;
    backdrop-filter: blur(6px);
    pointer-events: none;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12.5px;
    min-width: 0;
    flex: none;
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11.5px;
    background: var(--lav);
    color: var(--lav-fg);
    padding: 3px 7px;
    border-radius: 5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 560px;
  }

  .sub {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .fov {
    margin-left: auto;
  }

  .spacer {
    flex: 1;
  }
</style>
