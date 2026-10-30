<script lang="ts">
// About (spec §11.13): icon, version, tagline, the Respawwwn line and
// links, the licence, the update line.
import respawwwnMark from "../../../assets/brand/respawwwn-mark.svg";
import Chip from "../components/Chip.svelte";
import Mark from "../components/Mark.svelte";
import WindowFrame from "../components/WindowFrame.svelte";
import { api } from "../lib/api";
import { interpolate, strings } from "../lib/strings";
import { update } from "../lib/update.svelte";
import { windows } from "../lib/windows.svelte";

let version = $state("");

$effect(() => {
  api
    .ping()
    .then((p) => {
      version = p.version;
    })
    .catch(() => {
      version = "";
    });
});

const links = [
  { label: strings.about.links.site, url: "https://respawwwn.com" },
  { label: strings.about.links.discord, url: "https://discord.gg/49tS4mVNbp" },
  { label: strings.about.links.kofi, url: "https://ko-fi.com/respawwwn" },
  { label: strings.about.links.github, url: "https://github.com/Respawwwn/panowww" },
];
</script>

<WindowFrame title={strings.sidebar.about} width={440} onclose={() => windows.close()}>
  <div class="about">
    <Mark size={72} radius={18} />
    <h2 class="t-window">{interpolate(strings.about.version, { version })}</h2>
    <p class="tagline">{strings.about.tagline}</p>
    <p class="by"><img class="mark" src={respawwwnMark} alt="Respawwwn" /> {strings.about.by}</p>
    <nav class="links">
      {#each links as link (link.url)}
        <button type="button" class="link" onclick={() => api.onboarding.openUrl(link.url)}>{link.label}</button>
      {/each}
    </nav>
    <p class="t-tip">{strings.about.licence}</p>
    {#if update.banner}
      <button type="button" class="link" onclick={() => update.banner && api.onboarding.openUrl(update.banner.url)}>
        <Chip tone="lav">{interpolate(strings.update.banner, { version: update.banner.version })}</Chip>
      </button>
    {:else}
      <Chip tone="ok">{strings.about.upToDate}</Chip>
    {/if}
  </div>
</WindowFrame>

<style>
  .about {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 10px;
    padding: 8px 0;
  }

  h2,
  p {
    margin: 0;
  }

  .tagline {
    color: var(--muted);
    max-width: 320px;
  }

  .by {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    display: block;
  }

  .links {
    display: flex;
    gap: 14px;
  }

  .link {
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--lav-fg);
    font: inherit;
    font-size: 12.5px;
    font-weight: 500;
  }
</style>
