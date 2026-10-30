# App icon

Concept *Lens + orbit* on the Respawwwn recipe (spec §12): purple `#6454B5` ground, white glyph, thick black rounded outline.

- `panowww.svg`: the vector source, full bleed, 256 × 256. Edit this one.
- `macos.svg`: Apple's squircle (824 px on the 1024 canvas) with the glyph; generated from `panowww.svg` by `build.sh`.
- `build.sh` (or `just icons`): renders `app/icons/` — the macOS set (`icon.icns`, the PNG files) from the squircle, `icon.ico` from the square. Needs `rsvg-convert` (`brew install librsvg`).

The same glyph is drawn inline by `ui/src/components/Mark.svelte` for the sidebar and About; keep both in step.
