# README images

Images the root `README.md` shows. WebP, at most 1760 px wide and 500 KB each:

```sh
magick shot.png -resize 1760x\> -strip tmp.png && cwebp -q 84 -alpha_q 100 -m 6 tmp.png -o name.webp
```

| File | What | Source |
|---|---|---|
| `download-macos.svg`, `download-windows.svg` | Download buttons under the header | drawn by hand |
| `setup.webp` | Game Setup, a mouse Game | app, CleanShot window capture |
| `capture-zone.webp` | Capture zone picker over World of Warcraft | app, full-screen capture |
| `session.webp` | Session running (also the hero until issue 27) | design board `B-SessionRunning` |
| `session-end.webp` | Session completed | design board `B-SessionEnd` |

The two Session images are the direction-B boards of `prototype/visual-design`, rendered in headless Chrome with the canvas runtime stubbed, the sample Games and Shots swapped for the human's Games and real World of Warcraft Shots. Retake them from the app once a full clean Session exists.
