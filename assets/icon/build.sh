#!/usr/bin/env bash
# Renders the app icon set from panowww.svg (spec §12, issue 20): the macOS
# squircle set (icns, PNG files) from macos.svg and the Windows ICO from the full
# bleed square. Needs rsvg-convert (brew install librsvg) and the Tauri CLI.
# Run from the repository root: assets/icon/build.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# macos.svg takes the glyph of panowww.svg between its markers.
python3 - "$here" <<'PY'
import pathlib, re, sys
here = pathlib.Path(sys.argv[1])
src = (here / "panowww.svg").read_text()
glyph = re.search(r"<!-- glyph -->\n(.*?)\n  <!-- /glyph -->", src, re.S).group(1)
mac = here / "macos.svg"
text = mac.read_text()
text = re.sub(r"(<!-- glyph -->\n).*?(\n  <!-- /glyph -->)", lambda m: m.group(1) + glyph + m.group(2), text, flags=re.S)
mac.write_text(text)
PY

rsvg-convert -w 1024 -h 1024 "$here/macos.svg" -o "$work/macos-1024.png"
rsvg-convert -w 1024 -h 1024 "$here/panowww.svg" -o "$work/square-1024.png"

# The CLI writes every platform's icons; only the desktop ones are kept.
(cd "$root" && pnpm tauri icon "$work/macos-1024.png" -o "$work/macos" >/dev/null 2>&1)
(cd "$root" && pnpm tauri icon "$work/square-1024.png" -o "$work/square" >/dev/null 2>&1)

dest="$root/app/icons"
mkdir -p "$dest"
for f in 32x32.png 64x64.png 128x128.png 128x128@2x.png icon.png icon.icns; do
  cp "$work/macos/$f" "$dest/$f"
done
cp "$work/square/icon.ico" "$dest/icon.ico"
echo "app/icons updated from assets/icon/panowww.svg"
