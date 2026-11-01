# Respawwwn brand identity for Panowww

Research for ticket [004](../.wayfinder/tickets/004-respawwwn-brand.md). Question: what from the Respawwwn universe should Panowww reuse, and under what terms?

Sources (read on 2026-10-29):

- Local repo `~/www/gameguessr/` (Respawwwn, codename GameGuessr), branch `dev`, commit `654a29d` ("Merge branch 'release/1.3.1' into dev"). Paths below are relative to that repo root.
- https://respawwwn.com/ (live homepage).

No asset files were copied into this repo. Everything is referenced by path.

## Facts

### Licence of the GameGuessr repo

- **There is no LICENSE or COPYING file** anywhere in the repo (root, `gameguessr-website/`, `gameguessr-api/`, `gameguessr-doc/`).
- `gameguessr-api/composer.json`: `"license": "proprietary"`.
- `gameguessr-website/package.json`: `"private": true`, `"license": "ISC"`. That is the npm `init` default, not a deliberate licence. With `private: true` and no LICENSE file, it does not grant anything.
- Terms of use (`gameguessr-website/app/routes/terms-and-conditions.tsx`, around lines 140-155, "Intellectual property rights"): "All trademarks, service marks, graphics and logos used in connection with our Website or Services, are trademarks or registered trademarks of Website Operator [...] Your use of our Website and Services grants you no right or license to reproduce or otherwise use any Website Operator or third-party trademarks."
- So the code and the brand assets are **all rights reserved** by default. Copying any of them into an MIT repo needs explicit permission from the owners.
- Owners/team (`gameguessr-website/app/routes/gameguessr.about.tsx`, `humans.txt`, `gameguessr-website/public/humans.txt`): Kevin Wenger (developer, GitHub/X `@wengerk`), Pierre Georges (visual designer, X `@pierre__georges`), Sébastien Graf (developer, GitHub `@seb-graf`). **Pierre Georges designed the visual identity.** If he did the work as a freelancer, the rights may still be his, not Respawwwn's.

### Logo files and variants

All the logos are hand-drawn white letters with a black outline (`fill="white"` + `fill="black"`), made to sit on the purple background.

| Asset | Path | Format / size | Notes |
|---|---|---|---|
| Wordmark | `gameguessr-website/public/logo.svg` | SVG 132x28 | white + black, transparent background |
| Wordmark with "beta" | `gameguessr-website/public/logo-beta.svg` | SVG 132x35 | the version the site uses now (`app/components/Logo.tsx`) |
| Wordmark (raster) | `gameguessr-api/assets/images/logo.png` | PNG 132x28 | used by the API for share images |
| Favicon / app mark | `gameguessr-website/public/favicon.svg` | SVG 16x16 | rounded square `rx=4`, fill `#6454B5`, white "R" glyph with black outline |
| Favicons (raster) | `gameguessr-website/public/favicon.ico`, `favicon.png`, `favicon-96x96.png`, `apple-touch-icon.png` (180x180) | PNG/ICO | |
| PWA icons | `gameguessr-website/public/web-app-manifest-192x192.png`, `web-app-manifest-512x512.png` (maskable) | PNG | closest thing to an app icon |
| Panorama watermark | `gameguessr-api/assets/images/watermark-360.png` | PNG 500x500 | stamped over the **nadir** of every panorama by `gameguessr-api/src/MessageHandler/Asset/Panorama/PanoramaWatermarkMessageHandler.php` (EXIF tag `Watermark added by Respawwwn`) |
| Screenshot watermark | `gameguessr-api/assets/images/watermark.png` | PNG | |
| Background pattern | `gameguessr-website/app/images/pattern.png`, `pattern-transparent.png` | PNG | |

There is no monochrome / one-colour logo, no light-background logo, and no vector source file (Figma etc.) in the repo.

### Colour palette (design tokens)

Defined as Tailwind v4 `@theme` tokens in `gameguessr-website/app/styles/tailwind.css`:

| Token | Value | Role (from usage) |
|---|---|---|
| `--color-purple-200` | `#ada3d8` | light accent |
| `--color-purple-400` | `#6454b5` | **brand purple** (favicon background) |
| `--color-purple-500` | `#57489f` | |
| `--color-purple-700` | `#45378e` | |
| `--color-purple-800` | `#362977` | **page background**: `theme_color` and `background_color` in `public/site.webmanifest` |
| `--color-green-400` | `#4fba4d` | |
| `--color-green-500` | `#46a944` | main button (default `Button` variant is `green`) |
| `--color-blue-500` | `#3cb5e1` | "soon" badges |
| `--color-yellow-500` | `#f2c94c` | |
| `--color-red-400` | `#ff4970` | |
| `--color-red-500` | `#ec2e6e` | errors |
| `--color-gray-300` | `#00000026` | black at 15 % |
| `--color-gray-400` | `#00000040` | black at 25 % |
| `--color-gray-500` | `#000000b2` | black at 70 % |
| `--color-white` / `--color-black` | `#fff` / `#000` | |
| `--color-brand-discord` / `-twitch` / `-google` | `#5865f2` / `#9146ff` / `#ea4335` | third-party brand colours |

Other tokens in the same file: 2px black outline + hard bottom drop shadows (`--shadow-card: 0px 0px 0px 2px #000, 0px 6px 0px 0px #00000040`, `--drop-shadow-bottom-1..6`), purple vignettes (`.left-gradient`, `.circular-gradient` in `app/styles/global.css`, `rgb(54 40 119 / 70%)`). `html { color-scheme: dark; }`, so the site is dark only.

The style is "chunky arcade": thick black outlines, hard offset shadows, text with an outline (`stroke-text` utility built from `text-shadow` in `app/styles/typography.css`), and click sounds on buttons (`app/components/Button.tsx`).

### Typography and font licences

From `app/styles/tailwind.css`, `app/styles/typography.css` and the `@fontsource` packages in `gameguessr-website/package.json`:

| Font | Use | Package | Licence |
|---|---|---|---|
| **Teko** (variable, 700) | headings `h-4xl`..`h-lg`, uppercase, letter-spacing 0.6px, 2px outline | `@fontsource-variable/teko` | SIL Open Font License 1.1 (Google Fonts) |
| **Inter** (variable, 400/600) | body `p-base` (18/26), `p-sm`, small headings `h-base`..`h-xs` | `@fontsource-variable/inter` | SIL OFL 1.1 |
| **Epilogue** (variable, bold) | small badges ("soon") | `@fontsource-variable/epilogue` | SIL OFL 1.1 |
| **Nanum Pen Script** | handwritten accents | `@fontsource/nanum-pen-script` | SIL OFL 1.1 |

The API also keeps static TTFs for image generation: `gameguessr-api/assets/fonts/teko-700.ttf`, `inter-500.ttf`.

All four fonts are OFL 1.1. Panowww can bundle them in an MIT repo, as long as the OFL text ships with the font files and the fonts are not sold on their own. They should come from upstream (Google Fonts / Fontsource), not from gameguessr. This is the one thing that needs no permission.

### Tone of voice

- Short, playful, gamer slang, written as "we": "Guess a new game everyday & level'up your gaming culture." (`public/site.webmanifest` description); "Compete against the clock.", "Crossword, leveled up.", "Challenge your friends." (homepage menu on respawwwn.com).
- Plays with repeated letters, like the brand name itself: "I want mooooore" (section heading in about/contribute pages).
- Warm, community first: "Respawwwn thrives on the passion and dedication of our gaming community." "Respawwwn is entirely free and we want to keep it that way forever." (`app/routes/gameguessr.contribute.tsx`).
- Headings in uppercase Teko. UI strings are English.
- The name is always written **Respawwwn** (capital R, three w's). The TikTok/YouTube handle is spelled differently: `respawww_n`.

### How Respawwwn credits itself

- respawwwn.com has **no "By Respawwwn" line, no copyright line and no GitHub link** in the visible page. Footer links: Privacy policy, Terms and conditions, Copyright complaints.
- Page title format: `<Page> | Respawwwn - Game Guesser | Gaming Challenge`.
- Logo alt text: "Go to the Respawwwn.com homepage" (`app/components/Logo.tsx`).
- Hosting credited to Antistatique (https://antistatique.net/) in the contributors list (`app/routes/gameguessr.contribute.tsx`).
- Official links (`app/components/SocialLinks.tsx`, same on the live site):
  - Website: https://respawwwn.com
  - Discord: https://discord.gg/49tS4mVNbp
  - TikTok: https://www.tiktok.com/@respawww_n
  - YouTube: https://www.youtube.com/@respawww_n
  - Instagram: https://www.instagram.com/respawwwn
  - Ko-fi: https://ko-fi.com/respawwwn
  - No X/Twitter, Bluesky or GitHub account for the brand. The GitHub org `Respawwwn` exists (see the map: `github.com/Respawwwn/game-pano-capture`).
- Contribution flow: "Join our Discord" + "Submit new game" Google Form (https://forms.gle/xKz7BdKrwVqswvLD8) + tutorial video (https://www.youtube.com/embed/ZR5Tn9tmDEM).

### Shared UI components / code

The website is React (Remix / React Router) + Tailwind v4, under `gameguessr-website/app/components/`. Pieces that could matter for a Tauri webview:

- `Button.tsx`: chunky 3D button with variants green/purple/blue/red/yellow/orange, sizes small/large, all in Tailwind classes, plays sounds through `useSound`.
- `Card.tsx`, `Modal.tsx`, `Toast.tsx`, `Tooltip.tsx`, `Checkbox.tsx`, `RadioButton.tsx`, `TextInput.tsx`, `Chip.tsx`, `Badge.tsx`, `Icon.tsx`.
- `app/assets/icons/*.svg`: 45 custom 32x32 icons (discord, panorama, gear, skull, smiley, play, pause, download, refresh…), loaded by `Icon.tsx` through `?react` SVGR imports. They appear to be drawn in-house, so they fall under the same all-rights-reserved status.
- `app/styles/typography.css`: the `h-*` / `p-*` / `stroke-text` utilities. `app/styles/tailwind.css`: the token block.
- There is no Rust code. The backend is Symfony/PHP. Nothing in it can be reused directly. The panorama watermark (ImageMagick at the nadir) belongs to the GameGuessr pipeline, not to capture.

## Recommendation

1. **Visual direction: borrow the colours, not the arcade look.** The map wants a Sindre-Sorhus-style native macOS look (sober, system fonts, light + dark). The GameGuessr look (thick outlines, hard shadows, Teko all caps, click sounds, dark only) clashes with that. Use **`#6454b5` (purple-400) as Panowww's accent colour** and maybe `#362977` (purple-800) for the app icon background, so it feels like family. Keep system UI fonts (SF Pro / Segoe UI) for the app. Teko could appear only on the app icon or the README banner, if at all.
2. **Do not reuse GameGuessr components or CSS.** They are React + Tailwind, all rights reserved, and built for a game, not a utility. Copying tokens (hex values) into Panowww's own CSS is fine: colour values on their own are not protected by copyright. Copying the CSS files or components is not, unless the owners agree.
3. **Logo / credit: a "By Respawwwn" line with a link, plus an MIT carve-out for the marks.**
   - In the app: an About panel / footer with "By Respawwwn" linking to https://respawwwn.com, plus the Discord and Ko-fi links (these are what the site promotes for its community).
   - Panowww needs its **own** icon/logo. Reusing the Respawwwn `R` mark or wordmark as the app icon would make Panowww look like the game itself.
   - If the Respawwwn wordmark ships in the repo (e.g. `logo.svg` in the About panel), put it in a separate folder (e.g. `assets/brand/`) with a NOTICE: "The Respawwwn name and logos are trademarks of Respawwwn and are not covered by the MIT licence." Then add a matching sentence to the README / LICENSE section. Lots of MIT projects do this (code MIT, marks reserved).
4. **Fonts:** if a brand font is needed, take Teko / Inter from upstream (OFL 1.1) and ship `OFL.txt` next to them.
5. **Tone:** README and in-app help in English, "we" voice, short and friendly, a light gamer wink ("level up your panoramas") but no slang in technical instructions. Keep the "www" joke in the name, as Panowww already does.
6. **Link to the game:** Panowww's purpose is making panoramas for Respawwwn. The README can point to the contribution flow (Discord + "Submit new game" form) as "Send your panoramas to Respawwwn". The nadir watermark in GameGuessr covers the bottom pole, which matches the map's "nadir recommended by default".

## Open questions for the human

1. **Who owns the Respawwwn marks and visuals?** Respawwwn as an entity (association, company, or the three founders personally)? Did Pierre Georges assign his rights to the logo and icons? The answer decides who can grant permission and what the NOTICE should name.
2. **Permission to ship `logo.svg` / `favicon.svg` in the public MIT repo** under a "trademarks reserved" carve-out, or just link to respawwwn.com with text only?
3. **Should Panowww get its own icon designed by Pierre Georges** (same purple family), or should a placeholder be made from the tokens?
4. **Credit wording:** "By Respawwwn", "A Respawwwn tool", or "Made by the Respawwwn team"? Should the About panel name the people (Kevin Wenger…) as well?
5. **Which links to show:** all five social links, or only website + Discord + Ko-fi? Is there a GitHub org page or email for support?
6. **Should Panowww link straight to the "Submit new game" Google Form**, or is that form internal / likely to change?
7. Does the human want to **put a licence on gameguessr** (or a `BRAND.md` usage policy), so future tools can refer to it instead of asking each time?
