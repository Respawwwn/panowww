# README images

Images the root `README.md` shows. WebP, at most 1760 px wide and 500 KB each:

```sh
magick shot.png -resize 1760x\> -strip tmp.png && cwebp -q 84 -alpha_q 100 -m 6 tmp.png -o name.webp
```

| File | What | Source |
|---|---|---|
| `hero.svg` | Animated hero: Capture zone, Session, stitched panorama | built locally from the app captures and the Session's Shots |
| `download-macos.svg`, `download-windows.svg` | Download buttons under the header | drawn by hand |
| `setup.webp` | Game Setup, World of Warcraft, zone set | app, CleanShot window capture |
| `capture-zone.webp` | Capture zone picker over World of Warcraft | app, full-screen capture |
| `session.webp` | Session running | design board `B-SessionRunning` with the real Shots |
| `session-end.webp` | Session completed | app, CleanShot window capture |

