<script lang="ts">
// The Panowww mark (spec §12): the app icon's glyph, "Lens + orbit", as
// the sidebar's mark and About's icon. The same vector as
// assets/icon/panowww.svg; keep both in step.
type Props = {
  /** Side in CSS pixels. */
  size?: number;
  /** Corner radius in CSS pixels. */
  radius?: number;
};

let { size = 22, radius = 6 }: Props = $props();
</script>

<span class="mark" style:width="{size}px" style:height="{size}px" style:border-radius="{radius}px" aria-hidden="true">
  <svg viewBox="0 0 256 256" width={size} height={size}>
    <rect width="256" height="256" fill="#6454b5" />
    <path d="M63.7 176A84 84 0 1 1 192.3 176" fill="none" stroke="#000" stroke-width="40" stroke-linecap="round" />
    <path
      d="M210.7 191.4L173.9 160.6L169.2 203.6z"
      fill="#fff"
      stroke="#000"
      stroke-width="16"
      stroke-linejoin="round"
      paint-order="stroke"
    />
    <path d="M63.7 176A84 84 0 1 1 192.3 176" fill="none" stroke="#fff" stroke-width="20" stroke-linecap="round" />
    <path d="M210.7 191.4L173.9 160.6L169.2 203.6z" fill="#fff" />
    <circle cx="128" cy="122" r="36" fill="#fff" stroke="#000" stroke-width="16" paint-order="stroke" />
    <circle cx="128" cy="122" r="16" fill="#000" />
    <circle cx="134" cy="116" r="5" fill="#fff" />
  </svg>
</span>

<style>
  .mark {
    display: inline-flex;
    flex: none;
    overflow: hidden;
    background: #6454b5;
  }

  svg {
    display: block;
  }
</style>
