<script lang="ts">
// The small line icons of the boards (b.css `.ic`): 24-unit paths, stroke
// in the current colour. Sidebar actions and footer, the empty state.
type Props = {
  name:
    | "plus"
    | "import"
    | "list"
    | "gear"
    | "help"
    | "duplicate"
    | "share"
    | "folder"
    | "play"
    | "crop"
    | "eye"
    | "zip"
    | "back"
    | "check"
    | "stop"
    | "warn"
    | "refresh"
    | "ptgui"
    | "autopano";
  size?: number;
};

let { name, size = 16 }: Props = $props();
</script>

<svg class="ic" class:filled={name === "play" || name === "stop"} viewBox="0 0 24 24" width={size} height={size} aria-hidden="true">
  {#if name === "plus"}
    <path d="M12 5v14M5 12h14" />
  {:else if name === "import"}
    <path d="M12 3v11M8 10l4 4 4-4M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" />
  {:else if name === "list"}
    <path d="M4 6h16M4 12h16M4 18h10" />
  {:else if name === "gear"}
    <circle cx="12" cy="12" r="3" />
    <path
      d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"
    />
  {:else if name === "help"}
    <circle cx="12" cy="12" r="9" />
    <path d="M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.3-1 .9-1 1.7M12 17h.01" />
  {:else if name === "duplicate"}
    <rect x="9" y="9" width="12" height="12" rx="2" />
    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
  {:else if name === "share"}
    <path d="M12 3v12M8 7l4-4 4 4M5 12v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6" />
  {:else if name === "folder"}
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
  {:else if name === "play"}
    <path d="M7 5v14l11-7z" />
  {:else if name === "crop"}
    <path d="M6 2v16a2 2 0 0 0 2 2h14M2 6h16a2 2 0 0 1 2 2v14" />
  {:else if name === "eye"}
    <path d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6S2 12 2 12z" />
    <circle cx="12" cy="12" r="3" />
  {:else if name === "zip"}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M3 9h18M10 4v5M10 13h4M10 16h4" />
  {:else if name === "back"}
    <path d="M19 12H5M11 6l-6 6 6 6" />
  {:else if name === "check"}
    <path d="M5 12.5l4.5 4.5L19 7.5" />
  {:else if name === "stop"}
    <rect x="6" y="6" width="12" height="12" rx="2" />
  {:else if name === "warn"}
    <path d="M12 3.5 2.8 19.5h18.4L12 3.5z" />
    <path d="M12 10v4M12 17h.01" />
  {:else if name === "refresh"}
    <path d="M20 12a8 8 0 1 1-2.4-5.7M20 4v5h-5" />
  {:else if name === "ptgui"}
    <!-- Stitched strips of a panorama. -->
    <rect x="2" y="7" width="20" height="10" rx="2" />
    <path d="M8.5 7l1 10M15.5 7l-1 10" />
  {:else if name === "autopano"}
    <!-- Detection: a lens with its points. -->
    <circle cx="12" cy="12" r="8" />
    <path d="M12 4v3M12 17v3M4 12h3M17 12h3M12 12h.01" />
  {/if}
</svg>

<style>
  .ic {
    flex: none;
    stroke: currentColor;
    fill: none;
    stroke-width: 1.75;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .filled {
    fill: currentColor;
    stroke: none;
  }
</style>
