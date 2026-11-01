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

<WindowFrame title={strings.about.title} width={420} height={560} onclose={() => windows.close()}>
  <div class="about">
    <Mark size={72} radius={18} />
    <div class="name">
      <h2 class="t-title">{strings.about.title}</h2>
      <p class="sub">{interpolate(strings.about.version, { version })}</p>
    </div>
    <p class="tagline">{strings.about.tagline}</p>
    <p class="by">
      <img class="mark" src={respawwwnMark} alt="" />
      <span>{strings.about.by}</span>
      <button type="button" class="link brand" onclick={() => api.onboarding.openUrl("https://respawwwn.com")}>
        {strings.about.brand}
      </button>
    </p>
    <nav class="links">
      {#each links as link (link.url)}
        <button type="button" class="link" onclick={() => api.onboarding.openUrl(link.url)}>{link.label}</button>
      {/each}
    </nav>
    <p class="t-tip licence">{strings.about.licence}</p>
    <div class="status">
      {#if update.banner}
        <button type="button" class="link" onclick={() => update.banner && api.onboarding.openUrl(update.banner.url)}>
          <Chip tone="lav">{interpolate(strings.update.banner, { version: update.banner.version })}</Chip>
        </button>
      {:else}
        <Chip tone="ok">{strings.about.upToDate}</Chip>
      {/if}
    </div>
  </div>
</WindowFrame>

<style>
  .about {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 10px;
    padding-top: 6px;
  }

  h2,
  p {
    margin: 0;
  }

  .name {
    margin-top: 6px;
  }

  .t-title {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .sub {
    color: var(--muted);
    font-size: 12.5px;
  }

  .tagline {
    font-size: 13px;
    max-width: 300px;
  }

  .licence {
    max-width: 320px;
    margin-top: 8px;
  }

  .status {
    margin-top: auto;
    padding-top: 10px;
  }

  .by {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
    font-weight: 600;
  }

  .by .mark {
    margin-right: 2px;
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
    margin-top: 6px;
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

  .link:hover {
    text-decoration: underline;
    cursor: pointer;
  }

  .link.brand {
    color: inherit;
    font-size: inherit;
    font-weight: inherit;
  }
</style>
