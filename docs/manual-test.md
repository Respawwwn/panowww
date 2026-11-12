# Manual tests

What only a human with a game can verify. One table per OS. Reference game
on macOS: **NoClip** (noclip.website in the browser, a window on the single
display), camera dragged with the left mouse button. Record the date and
the build (commit) next to every result.

## Fixtures

`just fixtures` writes, through the store code the app uses, three Games (keyboard, mouse drag, gamepad), one damaged Game file and eight Sessions per Game covering every row state of the tables: completed today, interrupted today, failed yesterday (display gone), completed three days ago, failed ten days ago (capture unavailable), completed last month and last year, and one still `running` (marked interrupted at the next launch, as a crash would be). Shots are small gradient PNG files.

- `just fixtures --game "NoClip - Generic"` also adds the eight Sessions to that existing Game.
- `just fixtures --clean` removes the fixture Games and their Sessions first (never the named Games).
- `--data DIR` and `--sessions DIR` point elsewhere than the app's folders.

Running it twice adds another set of Sessions; the app picks the files up live.

## macOS · input (issue 9)

Prerequisites: Accessibility granted to the binary that runs the check (for
the example below, the terminal that launches `cargo run` must be in the
Accessibility list, or the events are dropped silently). After every new
build of the app itself, remove Panowww from the list and add it again
(unsigned build, spec §8.4).

```sh
cargo run -p panowww-input --example turn_test -- check
```

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| I1 | Accessibility check | `turn_test check` before and after granting | `false` before, `true` after; the request shows the system prompt once; System Settings opens on the Accessibility list | Pass: `true` with the terminal in the list (the before/after prompt was checked at the first grant) (2026-11-01) |
| I2 | Key hold | `turn_test keyboard KeyD 120 13` with the game's turn key, landmark picked | The camera turns 13 times and ends on the landmark | Pass in the app: a keyboard Game on NoClip (arrow keys) turns and captures a full Session (2026-11-01, release build) |
| I3 | Mouse move | `turn_test mouse move 840 13` on a mouse-look game | Same as I2; note the delta that closes the turn | |
| I4 | Mouse drag | `turn_test mouse drag right 840 13` on the drag-camera reference game | The cursor is parked, the camera orbits 13 times and ends on the landmark | Pass via the app's Turn test on NoClip (mouse drag, left button): the camera orbits and the result line appears (2026-11-01, f46c8df) |
| I5 | Stop releases everything | Start I4, press Ctrl+C after two Steps | No key or button stays held (the game is not dragging on its own) | Covered by the keyboard Session of I2: nothing stayed held after Stop (2026-11-01) |

Game matrix:

| Game | Input device | Mode | Display mode | I2 | I3 | I4 | Notes |
|---|---|---|---|---|---|---|---|
| NoClip (noclip.website) | Mouse | Drag, left button | Browser window, single display | – | – | ✓ 2026-11-01 | Turn test, Check overlap and a full Session pass from the app |

## macOS · capture (issue 10)

Prerequisites: Screen Recording granted to the binary that runs the check
(the terminal for the example), then relaunch it.

```sh
cargo run -p panowww-capture --example shot -- check
cargo run -p panowww-capture --example shot -- list
cargo run -p panowww-capture --example shot -- 0 100 100 1600 900
```

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| C1 | Screen Recording check | `shot check` before and after granting, relaunch between | `false` before, `true` after; the request shows the system prompt once; System Settings opens on the Screen Recording list | Pass: `true` with the terminal granted (2026-11-01) |
| C2 | Displays | `shot list` with one and two displays | Each display once, size in points, scale 2 on Retina, the id stable across a relaunch and a replug | Pass with one display: `0610-a050-fd626d62 · 1728×1117 pt · scale 1 · origin (0, 0)`, stable across relaunches; two displays not available (2026-11-01) |
| C3 | Zone Shot | `shot 0 100 100 1600 900` with a Panowww window and the terminal over the zone | `shot.png` is 3200 × 1800 px on Retina, shows the game, not the Panowww window, no cursor | Pass: `shot.png` 1600 × 900 px on a scale-1 display, the screen content, no cursor; the own-window exclusion was checked through the app (O5) (2026-11-01) |
| C4 | Timings at 1440p | Same, zone 2560 × 1440 | Note the first and second shot times (spike: 95–155 ms then 30–47 ms) and the PNG time | On the 1728 × 1117 display, zone 1600 × 900, debug build: first Shot 162 ms, second 85 ms, PNG 340 ms (debug; release measured in the spike) (2026-11-01) |
| C5 | Preview | Same run, the three preview lines | About 6 frames in 3 s, 30–60 KB each, none after stop | Pass: 6 frames in 3 s, 38.9–39.3 KB each, none after stop (2026-11-01) |
| C6 | macOS 26 picker alert | First capture of a fresh build | The "bypass the system private window picker" alert appears once; after Allow, captures work and the alert is gone | Pass on the release build: the alert came once at the first capture, captures work after Allow (2026-11-01) |

## macOS · app shell (issue 11)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| A1 | Start shortcut with another app focused | Select a Game with a zone, switch to the game, press Ctrl+Shift+8 | The countdown starts in Panowww (3 s) while the game keeps the focus; the game shows no reaction to the keys | Pass: Ctrl+Shift+8 from NoClip starts the countdown, the game keeps the focus (2026-11-01) |
| A2 | Stop shortcut | During the countdown and during a Session, press Ctrl+Shift+9 | Countdown: cancelled, no folder created. Session: `interrupted`, the Shots taken are in the folder, nothing stays held | Pass: Ctrl+Shift+9 stops (2026-11-01) |
| A3 | Rebinding | Settings → change Start to another combo, Save | The old combo does nothing, the new one starts; a combo already taken by the system shows the inline error | Found: the same combo for Start and Stop was refused silently, and the shortcuts fired while Settings was open (recording Start started a Session). Fixed: Start = Stop is refused with an inline error before anything is saved, the shortcuts pause while Settings is open. Retest: pass (2026-11-01) |
| A4 | Quit during a Session | Cmd+Q while a Session runs | The app quits within a second; `session.json` says `interrupted` with the Shots on disk | Found: Cmd+Q quit at once but the Session stayed `running` (the menu Quit arrives as `Exit`, not `ExitRequested`) and the relaunch marked it interrupted with 0 Shots while 5 PNG files existed (the running record lists none). Fixed: shutdown on `Exit` too, and the stale marking counts the rNN_cNN.png files on disk. Retest: pass, the Session reads interrupted with its Shots (2026-11-01) |

## macOS · overlay (issue 12)

Reference game in a full-screen Space on the zone's display. The overlay is a `tauri-nspanel` non-activating panel; the spike validated the same setup.

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| O1 | Show zone | Setup → *Show zone*, switch to the game (full-screen Space) | The lavender frame sits exactly on the zone over the game; the game keeps the focus (keep moving the camera with the real mouse); clicks inside the frame reach the game; *Hide zone* removes it, as does leaving the Game | Pass: frame on the zone over the game, game keeps the focus, clicks reach it (2026-11-01, f46c8df) |
| O2 | Countdown digit | Start a Session with the shortcut from the game | 3, 2, 1 centred in the frame over the game, the game never loses the focus, the Panowww window does not come forward | Pass, started from the app (5 s) (2026-11-01, f46c8df) |
| O3 | Progress | Let the Session run | A 2 px bar fills along the top inner edge and the `k / n` pill top right follows every Shot; nothing is drawn outside the zone | Pass (2026-11-01, f46c8df) |
| O4 | End pill | Let the Session finish; run another and stop it with the shortcut | `Done · n Shots` for about 2 s, then the overlay is gone; `Stopped · k / n` (amber) after a stop | Pass, Done pill after a complete Session (2026-11-01, f46c8df) |
| O5 | Not in the Shots | Open the Session folder | No frame, digit, bar or pill in any PNG | Pass (2026-11-01) |
| O6 | Cannot start | Revoke Screen Recording, start a Session | The error state shows in the window and no overlay appears (or a frame shown by *Show zone* goes away) | Pass on the release build: the *Cannot start* state shows, no overlay (2026-11-01) |
| O7 | Second display | Draw a zone on a second display with a different scale, run a Session | The overlay covers that display, the frame matches the zone there | Pass: overlay, digit, bar and pill on the 2× display, on the zone, nothing on the other display (2026-11-02) |
| O8 | Release build transparency | `pnpm tauri build`, run the bundle, *Show zone* | The overlay window is transparent (no white sheet over the game); if it is white, enable `app.macOSPrivateApi` in `tauri.conf.json` and the `macos-private-api` feature of `tauri` | Pass: the frame is transparent over the game in the release bundle (2026-11-01) |

## macOS · full flow (issue 19)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| F1 | Session row actions | Sessions tab: Zip, Reveal, Delete on one row | The zip appears next to the folder; Finder opens on it; Delete removes the folder and its zip | Pass (2026-11-01) |
| F2 | Share → Import, same machine | Game → Share to a file; sidebar → Import that file | The `.panowww.json` is written; Import shows Replace / Keep both; Keep both adds the Game under a renamed copy | Pass (2026-11-01) |
| F3 | Settings folder | Settings → Change the Sessions folder, run a Session, change it back | The Session lands in the new folder; the tables follow | Pass (2026-11-01) |
| F4 | Calibrate tools | Step once (Turn and Pitch), Pitch test | One Step moves the camera; the Pitch test tilts from the start pole and the result line appears | Pass (2026-11-01) |
| F5 | Update bar | Build with a version below the latest published Release (edit the workspace `version`, not committed), launch with *Include pre-releases* off, then on | Off: no bar while the latest Release is marked as pre-release; on: the bottom bar names the Release and links its page; *Check for updates at launch* off hides it | Pass (2026-11-08, b802761 built as 0.0.9 against the published `v0.1.0` pre-release) |
| F6 | Onboarding on a fresh install | Release bundle, `tccutil reset All com.respawwwn.panowww`, launch | Both rows red; *Open System Settings* on each shows the system prompt and opens the right pane; a row turns green as soon as its switch is ticked and the window comes back to the front; Screen Recording asks for a relaunch, then *Continue* is enabled | Pass on the release build after a TCC reset: both rows turn green live, Continue enabled after the Screen Recording relaunch (2026-11-01, 51bda69) |
| F7 | No Input Monitoring | Same run, watch the prompts | macOS never asks for Input Monitoring: Panowww only posts events, which the Accessibility grant covers | Pass after a TCC reset: no Input Monitoring prompt (2026-11-01). The first run had one, accepted at 14:29:52, trigger not found; watch for it on fresh installs |
| F8 | Reset the Sessions folder | Settings → *Change* the folder, then *Reset* | *Reset* appears next to *Reveal* only while the folder is not `~/Pictures/Panowww`; it brings that folder back (full path) and disappears; *Appearance* and *Folder* sit in a narrower label column | Pass (2026-11-05, b1cfda6) |
| F9 | Start card folder | Game → Setup, look at the bottom line of the purple Start card | `<Sessions folder>/<Game>/` in a translucent white code chip, ellipsis when too long, the full path on hover | Pass (2026-11-05, 780ec03) |
| F10 | Settings shortcut caps | Look at the sidebar footer, light then dark; press Cmd+, | *Settings* ends with `Cmd` + `,` key caps like Start and Stop, white caps on the lavender sidebar (card colour in dark); Cmd+, opens Settings | Pass (2026-11-05, 91b1238) |
| F11 | Settings label columns | Open Settings | General and Shortcuts share a narrow label column; New Game defaults keeps the wide one for its long labels | Pass (2026-11-05, e9322a0) |
| F12 | Modal scroll shadow | Settings, scroll the body down then back to the top | A line and soft shadow appear under the title bar once scrolled, gone at the top; light and dark | Pass (2026-11-05, 9a655e9) |
| F13 | Detail scroll shadow | A Game's Setup then Sessions tab, scroll; switch tab while scrolled; All Sessions, scroll | The shadow appears under the Game's title row (12 px under its buttons) once scrolled, gone at the top and after a tab or Game switch; under the All Sessions toolbar, the shade only (it keeps its border) | Pass on a Game's Setup (2026-11-05, 6c2947b); All Sessions not looked at |
| F14 | Dark colours | Settings → Appearance → Dark, walk the main screens | Neutral greys of direction A (no purple tint) on sidebar, content, cards and fields; purple accent; the Start card's *Start Session* is the toolbar purple; the `Cmd` + `,` caps stand out from the sidebar | Pass (2026-11-05, a45f4e4) |
| F15 | Sidebar footer | Look at the sidebar footer, light then dark | The cog icon of the design beside *Settings*; a separator line above All Sessions, Settings and About, visible in both themes | Pass (2026-11-05, 413bc92) |
| F16 | Onboarding once | Both permissions granted; delete `settings.json` (or set `onboarded` to `false`), launch, *Continue*; relaunch | Onboarding shows at the first launch although both rows are green; after *Continue*, `onboarded` is `true` in `settings.json` and the next launch goes straight to the window | Not tested yet: the `onboarded` flag was added on 2026-11-04 (issue 26, found on Windows where nothing is ever missing). Run at the next macOS pass |
| F17 | Open in a Stitcher | Settings → Stitchers: *Choose…* PTGui and Autopano Giga (`.app`); open a completed Session from a table; *Open in PTGui* twice, *Open in Autopano Giga*; then *Reset* one | The two buttons show for a completed Session only; PTGui opens the project with every Shot (the `.pts` sits in the Session folder, the second click opens it without recreating it); Autopano Giga loads every Shot and *Detect* runs; a moved program is reported under the buttons with a pointer to Settings; the reset Stitcher's button disappears | Pass (2026-11-08, eab2802): both buttons on a completed Session only, PTGui opens the project with every Shot (second click reuses the `.pts`), Autopano Giga loads them and *Detect* runs, *Reset* removes the button |
| F18 | Stitcher version line, button order | Settings → Stitchers with both set; move or rename one `.app`, click the refresh icon; put it back, refresh; *Choose…* another application (Safari) for PTGui, then PTGui again; open a completed Session, then reset PTGui, then both | Under each set row: green dot and `PTGui version: 13.10` / `Autopano Giga version: 4.4.2`; the moved one turns red with *not installed*, green again after the refresh; Safari turns the PTGui line red with *not PTGui (Safari)*. On the Session the right-aligned row above the card reads Open in PTGui (purple), Open in Autopano Giga, Zip Session, Reveal; with PTGui reset, Autopano Giga is purple; with none, Zip Session is. The Shots card's background covers every Shot when the window is small | Pass (2026-11-08, eab2802): both lines green with their versions, *not installed* and *not PTGui (Safari)* in red, the refresh icon re-runs the check; button order and purple button as expected; the Shots card covers its grid in a small window |
| F19 | Calibrate, Overlap first | Game → Setup: read the Calibrate card and the hold / delta units; on NoClip run 1 · Check overlap (turn), press −10 % / +2 %, 2 · Turn test, press −3 / +1, then 3 and 4 for pitch | The card reads 1 · Check overlap + Overshot/Undershot row, 2 · Turn test + Decrease/Increase row, 3 · Check overlap, 4 · Pitch test; the `?` of each Check overlap explains the Step buttons, that of each Test the count buttons; every hold reads `ms per Step`, every delta `units per Step`; the Overlap verdict names Overshot (−) or Undershot (+); the Step buttons change the hold / delta in the input card, the count buttons change Shots per Row / Rows in the grid card, clamped at 3 and 2 | Not run yet |
| F20 | Adjust buttons glow their field | With Grid & timing open, press +2 % under Check overlap (turn); then −3 under Pitch test; then +1 twice | Game & input opens, the Turn hold (or Turn delta) field glows for under a second and the field is scrolled into view; Grid & timing opens for Rows the same way; the second +1 glows again | Not run yet |

## macOS · Capture zone picker (issue 13)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| P1 | One window per display | Setup → *Set zone…* with two displays connected | A borderless window covers each display with a dimmed screenshot of it, `Display n · w × h @sx` top left; the one holding the current zone (else the first) has the focus | Pass with two displays (EIZO 1×, MacBook 2×): a cover per display, chips, focus on the zone's display; found that a rectangle could exist on both displays at once, fixed the same day (2026-11-02). With one display: pass after two fixes, transparent panel over the live display, built on the main thread (2026-11-01) |
| P2 | Drawing | Drag on the picture, then drag a corner handle, then drag the rectangle body | The purple rectangle follows with its size chip; handles resize from the opposite corner; the body moves and stops at the display's edge; X / Y / W / H follow and edit it | Pass (2026-11-01, f46c8df) |
| P3 | Two scales | Draw a zone on a 2× display, Save; then on a 1× display, Save | Setup shows `Display n · w × h at (x, y)` in points for each; `local-games.json` stores `displayW` / `displayH` / `scale` of the display drawn on; a Session's Shots are `w × scale` pixels wide | Pass: zones drawn and Sessions captured on the 2× MacBook screen and on the 1× EIZO (2026-11-02) |
| P4 | Show frame on screen | Tick the checkbox while drawing, then Save | The overlay frame appears exactly on the rectangle (through the picker's dim); it goes away with the windows | Pass (2026-11-01, f46c8df) |
| P5 | Cancel and Esc | *Cancel* in one window; open again and press Esc in the other | Every picker window closes, the zone is unchanged, Setup is usable again | Found: Esc did nothing (the panel was not the key responder). Fixed: the focused picker window takes the keyboard. Retest: pass, Esc and Cancel both close (2026-11-01) |
| P6 | Display changed | Save a zone, change that display's resolution, reopen Setup | The amber *Display n is gone or its resolution changed* box; *Change zone…* opens the picker on the display as it is now | Not run by decision (2026-11-02): a resolution change after the zone is saved is out of scope for v1; the amber box and *Change zone…* stay as the safety net |

## macOS · after the Windows app work (issue 25)

The Windows commits touched shared code: window creation order, the theme
the main window is born with, `~` expansion and OS separators for the
Sessions folder, the overlay and the picker. Rerun on the macOS release
bundle (`pnpm tauri build`).

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| MR1 | Window creation | Launch the bundle | Games and saved settings show at once, no reload needed | Pass (2026-11-05, ce60d17) |
| MR2 | Theme at launch | Pick Light, then Dark, relaunch each time | The window opens in the saved theme, no flash | Pass (2026-11-05, ce60d17) |
| MR3 | Sessions folder | Settings and Sessions tab: folder lines, *Reveal* on a row | Paths use `/`, `~` shows the home folder, Finder opens with the Session selected | Pass (2026-11-05, ce60d17) |
| MR4 | Overlay and zone picker | *Set zone…* with two displays, *Show zone*, run a Session | Picker and overlay as in P1–P4 and O1–O8 | Pass (2026-11-05, ce60d17) |

## Windows · input (issue 23)

Reference games on Windows: **NoClip** (drag camera, left button, in a
maximized browser) and **Star Wars Battlefront 2** in offline Arcade
(raw-input mouse-look, takes an Xbox pad). Windows has no input permission:
the example runs from a normal terminal and reaches any game that is not
elevated; the app itself runs elevated (spec §8.4). Never inject into a
multiplayer game with anti-cheat.

Gamepad rows: **unplug every physical controller** first, the virtual pad
must be XInput player 1. ViGEmBus 1.22.0 is the version the README names;
1.17.333 works but answers the wait-ready call at random (Win32 483, 259),
which the adapter only logs. A pad left plugged by a killed process stays
listed until a reboot.

```sh
cargo run -p panowww-input --example turn_test -- check
```

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| WI1 | Prerequisites | `turn_test check` from a normal terminal, then from one run as administrator | `elevated` false then true; `ViGEmBus: installed` with the driver, `The virtual gamepad driver is not installed.` after `sc stop ViGEmBus` (as administrator; `sc start ViGEmBus` afterwards) | Pass from an administrator prompt: `elevated: true`, `ViGEmBus: installed` (driver 1.17.333). The non-elevated prompt and the stopped-driver message were not tried (2026-11-01, 1dc654d) |
| WI2 | Key hold | `turn_test keyboard ArrowRight 120 13` on NoClip with the arrow keys, landmark picked | The camera turns 13 times in equal Steps; it ends on the landmark only once the hold is calibrated for the game (the example's 120 ms is a fixed guess, calibration is per Game in the app) | Pass on NoClip: 13 equal turns, landmark far off with the uncalibrated hold, as expected (2026-11-01, 1dc654d) |
| WI3 | Mouse move | `turn_test mouse move 840 13` on Battlefront 2 | Same as WI2 with the example's fixed 840-unit delta | Pass on Battlefront 2, offline Arcade: 13 equal turns, landmark not closed with the uncalibrated delta (2026-11-01, 1dc654d) |
| WI4 | Mouse drag | `turn_test mouse drag left 840 13` on NoClip | The cursor is parked, the camera orbits 13 times in equal Steps (landmark as in WI2) | Pass on NoClip, left button: parked, 13 equal orbits (2026-11-01, 1dc654d) |
| WI5 | Stop releases everything | Start WI4, press Ctrl+C after two Steps | No key or button stays held (the game is not dragging on its own) | Pass: Ctrl+C ends the run, nothing stays held (2026-11-01, 1dc654d) |
| WI6 | Virtual pad | `turn_test gamepad 0.8 120 13`: it plugs the pad and waits; start or restart the game (some only look for controllers at launch), pick a landmark, press Enter in the terminal and switch to the game | The game shows pad prompts, the camera turns 13 times in equal Steps (landmark as in WI2); `virtual pad plugged: true` at the end | Pass on Battlefront 2 with the physical Xbox One pad unplugged, ViGEmBus 1.17.333: 13 equal turns with the uncalibrated 0.8 / 120 ms stick (2026-11-01, 1dc654d) |

Game matrix:

| Game | Input device | Mode | Display mode | WI2 | WI3 | WI4 | WI6 | Notes |
|---|---|---|---|---|---|---|---|---|
| NoClip (noclip.website) | Keyboard, Mouse | Arrow keys; drag, left button | Maximized browser, primary display | ✓ 2026-11-01 | – | ✓ 2026-11-01 | – | Plain mouse moves do nothing in NoClip, only drags |
| Star Wars Battlefront 2 (2017) | Mouse, Gamepad | Move (raw input); right stick | Borderless, primary display | – | ✓ 2026-11-01 | – | ✓ 2026-11-01 | Offline Arcade only (anti-cheat online); physical pad unplugged for WI6 |

## Windows · capture (issue 24)

No permission to grant on Windows. The example makes itself per-monitor
DPI aware, so sizes and zones are physical pixels. Our own windows are
excluded from capture by the app, not by the example: the terminal that
runs it does appear in `shot.png` when it covers the zone.

```sh
cargo run -p panowww-capture --example shot -- list
cargo run -p panowww-capture --example shot -- 0 100 100 1600 900
```

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| WC1 | Displays | `shot list` with the two displays | Each display once, primary first, physical size, scale 1 at 100 % (1.25 at 125 %), origin of the second display at ±1920, the id (a `\\?\DISPLAY#…` device path) identical across a relaunch | Pass at 100 %: two displays, primary first, `\\?\DISPLAY#SAM09E9#…` and `#SAM0B43#…` identical across relaunches, second at (−1920, 0); 125 % not tried (2026-11-01, 21e4312) |
| WC2 | Zone Shot | `shot 0 100 100 1600 900` with a game or NoClip on the primary display | `shot.png` is 1600 × 900 px, shows the screen content, no cursor; no yellow border appears around the display while it runs | Content half passes: 1600 × 900 px of the NoClip page at (100, 100), no cursor, 10 ms in release. The border watch is deferred to the Windows end-to-end pass (issue 26) with the final app; the spike saw none with the same settings (W6) (2026-11-01, 21e4312) |
| WC3 | Timings at 1080p | `shot 0 0 0 1920 1080` over a running game | Note the first and second Shot times (spike, release: 5–21 ms over NoClip, 40–85 ms over Battlefront 2) and the PNG time | Release over an idle NoClip page: first Shot 19 ms, second 574 ms (the display composed nothing sooner: WGC hands out one frame per composition, every 0.5–1 s on a still screen, every refresh over a running game), PNG `fast` 17 ms. Debug build: 113–124 ms Shots, PNG 960 ms (2026-11-01, 21e4312) |
| WC4 | Preview | Same run, the preview lines | About 6 frames in 3 s, 30–60 KB each, none after stop | Pass: 6 frames in 3 s at 32 KB when the display composes continuously, 4 in 3 s on a still screen (same cause as WC3), none after stop once the late-frame race was fixed (2026-11-01, 8250840) |
| WC5 | Nothing composes | `shot 0 0 0 1920 1080` on a display showing a still picture, cursor parked elsewhere and not moved | The first Shot fails after 2 s with `no frame newer than the move`: WGC delivers frames only when the display composes a new one, a running game always does | Not reproduced: the still secondary display still composed every 0.5–1 s (Shots 124 and 105 ms, 6 preview frames). The 2 s timeout stays as the safety net (2026-11-01, 21e4312) |
| WC6 | HDR flag | `shot list` with HDR on for one display (Settings → System → Display), then off | `· HDR` after that display's line, gone after turning it off | Not available: neither display of the test machine supports HDR; the flag reads `false` for both (2026-11-01) |

## Windows · app (issue 25)

Reference games as for the input rows: NoClip in a maximized browser and
Star Wars Battlefront 2 in offline Arcade, borderless, on the primary
display. The app runs elevated (UAC prompt at launch).

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| WA1 | Elevation | Launch the built app (`pnpm tauri build`, then the exe in `target/release`); then `cargo test -p panowww-app` | The app asks for UAC at launch and onboarding shows the *Running as administrator* card; the test binaries ask nothing | Pass: UAC prompt (unsigned publisher), the card and a green ViGEmBus row; `cargo test -p panowww-app` asks nothing (2026-11-02, c762758). Found: no Games and default settings at launch until a reload, Tauri creates the window before `setup` manages the state; fixed in a177336, pass after (2026-11-03) |
| WA2 | Show zone | Setup → *Show zone*, switch to the game | The lavender frame sits exactly on the zone over the game; the game keeps the focus (keep moving the camera with the real mouse); clicks inside the frame reach the game; *Hide zone* removes it | Pass on NoClip (2026-11-03, a177336). Found on the way: *Draw zone* hung the app at once, `Picker::open` held its lock while the first page asked for its setup; fixed in c26fab7, the picker covers both displays after |
| WA3 | Countdown digit | Start a Session with Ctrl+Shift+8 from the game | 3, 2, 1 centred in the frame over the game, the game never loses the focus, the Panowww window does not come forward | Pass (2026-11-03, a177336) |
| WA4 | Progress and end pill | Let the Session run, then run another and stop it with Ctrl+Shift+9 | Bar and `k / n` pill inside the frame; `Done · n Shots` then gone after about 2 s; `Stopped · k / n` after a stop | Pass (2026-11-03, a177336) |
| WA5 | Not in the Shots | Open the Session folder | No frame, digit, bar or pill in any PNG, no Panowww window, no yellow border was visible during the run | Pass: Shots clean, only the lavender frame on screen, no yellow border (2026-11-03, a177336) |
| WA6 | Close quits | Close the main window with ✕ while idle, then while a Session runs | The process is gone within a second (Task Manager); the Session reads `interrupted` with its Shots; a plugged virtual pad disappears from Device Manager | Pass: the process quits, a Session closed mid-run or stopped reads `interrupted` (2026-11-03, a177336); the pad check passed in WF5 (2026-11-07) |
| WA7 | Windows chrome | Look at the main window | Native caption bar (─ ☐ ✕), 8 px cards and 4 px small radii, no space reserved for macOS traffic lights | Pass (2026-11-02, c762758). Found: the sidebar showed `⌘,` and Light came back dark after a relaunch (the caption bar kept the OS theme); fixed in dbf79ca and 3fdeec6, pass after. 2026-11-04: the caption bar flashed dark before turning light and folder lines mixed `/` and `\`; fixed in 247b32d (window born with the saved theme) and 96fed26 (OS separator in the UI), pass after |
| WA8 | Reveal in Explorer | Sessions tab → *Reveal* on a row; Settings → *Reveal logs* | Explorer opens with the folder selected | Found: both opened Documents, and Sessions went to `target\release\~\Pictures\Panowww`: the UI's `~/Pictures/Panowww` placeholder had been saved as the Sessions folder (a consequence of the WA1 race) and a literal `~` is relative; `~` now expands on load and save, Explorer gets a native path. A space in the path (`Kevin Wenger`) still sent Explorer to Documents: `/select,"<path>"` is now one raw argument. Pass (2026-11-04, 96fed26): Explorer opens the parent with the Session (or `logs`) selected, as Finder's reveal does |
| WA9 | Nothing moved? popover | Open it from a Windows mouse Game and a gamepad Game | Item 4 ends with "Use a gamepad Game for them."; the gamepad line mentions player 1 and the restart | Pass (2026-11-03, a177336) |
| WA10 | Second display at 125 % | Set the secondary display to 125 %, draw a zone there, *Show zone*, run a Session | The picker covers that display exactly, the frame lands where the Shot is taken | Skipped by decision, as the matching macOS row (2026-11-03). Reopened by issue 8: a user with a 1080p primary at 100 % and a 1440p at 125 % got a picker covering 80 % of the 1440p display. Pass on three layouts: secondary 1920×1080 at 125 %, secondary at another resolution (1440×900 at 100 %), secondary at 125 % on the left (2026-11-12, a7768af with main merged, installed build). The log shows the three picker openings (`1920×1080 @1.25`, `1440×900 @1`) and no `picker window does not cover its display` line. Not run: the primary at 125 % with the secondary at 100 % |
| WA11 | Reset the Sessions folder | Settings → *Change* the folder, then *Reset* | As F8, the folder back to `C:\Users\<you>\Pictures\Panowww` with backslashes | Pass: `settings.json` holds `C:\Users\<you>\Pictures\Panowww` after the Reset (2026-11-07, 389d6f1, installed build) |
| WA12 | Start card folder | As F9 | As F9, with `\` separators and a trailing `\` | Pass (2026-11-07, 389d6f1) |
| WA13 | Settings shortcut caps | As F10 with Ctrl+, | `Ctrl` + `,` caps, Ctrl+, opens Settings | Pass (2026-11-07, 389d6f1) |
| WA14 | Sessions tab folder | A Game → Sessions tab, the footer line | `Sessions are saved in <folder>\<Game>\` with backslashes only | Pass (2026-11-07, 389d6f1) |
| WA15 | Settings and scroll shadows | As F11, F12 and F13 | Same as on macOS | Pass (2026-11-07, 389d6f1) |
| WA16 | Dark colours | As F14, with `Ctrl` + `,`; the caption bar follows Dark | Same as on macOS | Pass, Dark left saved (2026-11-07, 389d6f1) |
| WA17 | Sidebar footer | As F15 | Same as on macOS | Pass (2026-11-07, 389d6f1) |

## Windows · full flow (issue 26)

The issue-19 protocol on Windows, from the installer. Build it with
`pnpm tauri build`; the installer is
`target\release\bundle\nsis\Panowww_0.0.1_x64-setup.exe` (per-user, no
administrator rights to install; the app itself asks for UAC at every
launch). A fresh install for WF2 = move `%APPDATA%\com.respawwwn.panowww`
aside before launching. Reference games as above: NoClip for the keyboard
and mouse-drag Games, Star Wars Battlefront 2 (offline Arcade, borderless)
for the mouse-look and gamepad Games. Gamepad rows: physical controllers
unplugged.

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| WF1 | Installer | Run the setup.exe, then launch Panowww from the Start menu; later Settings → Apps → Uninstall | Installs under `%LOCALAPPDATA%\Programs\Panowww` without a UAC prompt, a Start menu entry, the app launches with its UAC prompt; the uninstaller removes the program, not the data or the Sessions | Pass: installs and launches as intended, UAC at launch (2026-11-04, 6802bde); the uninstall half not run yet |
| WF2 | Onboarding on a fresh install | Data folder moved aside, launch, *Continue*, relaunch | The *Running as administrator* card, a green ViGEmBus row (driver installed), *Continue* enabled; the main window follows with no Games and the default Settings (`~` expanded to the home folder); the relaunch skips onboarding (`onboarded` true in `settings.json`), and so does a launch with ViGEmBus stopped (the driver is optional) | Found: no onboarding at all. It only showed when a permission was missing, which on Windows with ViGEmBus never happens; macOS got its first launch by accident. Fixed: an `onboarded` flag in `settings.json`, set by *Later* / *Continue*; a missing ViGEmBus no longer counts as missing. Retest: pass, onboarding once with the two new texts, none at the relaunch (2026-11-04, 6615ded) |
| WF3 | Keyboard Game, full flow | New Game on NoClip, keyboard (arrow keys), *Draw zone*, Turn test, adjust the hold, Turn test again, Pitch test, Check overlap (Turn, then Pitch), Step once, then a complete Session | Every tool runs from the app and its result line appears; the Session completes with every Shot, `Done · n Shots`, the end screen shows the grid and the folder | Pass: every tool ran with its result line, the Session completed with 91 Shots (7 × 13) in 95 s, Shots 8–41 ms each (2026-11-04, 6615ded, installed build). Side note from the log: every autosave of the editor is an atomic write the folder watcher reports as Removed + Changed, so the window re-lists the Games folder about 7 times a second while a field is being edited; harmless, a candidate for ignoring the app's own writes |
| WF4 | Mouse-look Game | New Game on Battlefront 2, mouse (move), Turn test, Pitch test, a short Session | The camera turns in equal Steps with raw-input mouse look; the Session completes | Pass: Turn and Pitch tests with their result lines, then three complete Sessions of 78 Shots (6 × 13) in 90–100 s each, turn delta 806, pitch 790 (2026-11-04, installed build 6802bde) |
| WF5 | Gamepad Game | New Game on Battlefront 2, gamepad; Turn test (restart the game once after the first countdown if it ignores the pad), a short Session; then close Panowww | The setup shows the pad as plugged; the game turns with the virtual right stick; the Session completes; after the close, the virtual controller is gone from Device Manager (the WA6 remainder) | Run on Overwatch (practice range) instead of Battlefront 2, physical pad unplugged: pad plugged at the first countdown (`virtual pad ready`, 2026-11-04 19:33) and kept across runs; tests with the stick, then a complete Session of 35 Shots (5 × 7, stick 0.8, turn hold 1054 ms, pitch 1429 ms) on 2026-11-07, installed build 6615ded. Close: the process gone, the virtual Xbox 360 controller gone from the device list, the ViGEmBus alone left (2026-11-07). Pass |
| WF6 | Interrupted Session | Ctrl+Shift+9 during a Session; then close the window with ✕ during another | Both rows read `interrupted` with the Shots taken so far; nothing stays held in the game | Pass: the shortcut stop closed the Session at once (`Interrupted shots=1`), the ✕ close wrote `Interrupted` before the process left, the relaunch 12 s later found nothing stale; nothing held (2026-11-04, 6615ded) |
| WF7 | Session row actions | Sessions tab: Zip, Reveal, Delete on one row | The zip appears next to the folder; Explorer opens with the folder selected; Delete removes the folder and its zip | Pass (2026-11-04, 6615ded) |
| WF8 | Share → Import, same machine | Game → Share to a file; sidebar → Import that file | The `.panowww.json` is written where chosen; Import shows Replace / Keep both; Keep both adds the Game under a renamed copy | Pass (2026-11-04, 6615ded) |
| WF9 | Settings folder | Settings → Change the Sessions folder, run a Session, change it back | The folder dialog returns a native path; the Session lands in the new folder; the tables follow | Pass: a Session landed in `Documents\ShareX\Screenshots\<Game>\…` after the change, the folder changed back after (2026-11-04, 6615ded) |
| WF10 | Rebinding | Settings → change Start to another combo, Save; try Start = Stop | The old combo does nothing, the new one starts; Start = Stop is refused inline | Pass (2026-11-04, 6615ded) |
| WF11 | Update bar | Build with a version below the latest published Release (edit the workspace `version`, not committed), launch with *Include pre-releases* off, then on | Off: no bar while the latest Release is marked as pre-release; on: the bottom bar names the Release and links its page; *Check for updates at launch* off hides it | Pass (2026-11-08, 9c4f017 built as 0.0.9 against the published v0.1.0 pre-release): no bar with the switch off, the bar names 0.1.0 with it on, no bar once *Check for updates at launch* is off; three launches in the log |
| WF12 | WebView2 bootstrapper | Install on a machine without the WebView2 runtime | The installer downloads and installs the runtime before the app | Not available: Windows 11 ships the runtime |
| WF13 | Open in a Stitcher | Settings → Stitchers: *Choose…* `PTGui.exe` and `AutopanoGiga_x64.exe`; open a completed Session from a table; *Open in PTGui* twice, *Open in Autopano Giga*; then *Reset* one | Same as F17 | Pass (2026-11-08, 4b2caed): both buttons on a completed Session only, PTGui opens the project with the 78 Shots (second click reuses the `.pts`), Autopano Giga loads them and *Detect* runs, *Reset* removes the button |
| WF14 | Stitcher version line, button order | Same as F18 with `PTGui.exe` and `AutopanoGiga_x64.exe` (rename the Kolor folder for *not installed*, `notepad.exe` for the wrong program) | Same as F18, versions `13.10` and `4.4.2.400` | Pass (2026-11-08, d5cccb2): both lines green with their versions, the renamed Kolor folder reads *not installed*, `notepad.exe` reads *not PTGui (Bloc-notes)*, the refresh icon re-runs the check; right-aligned row above the card with the Stitcher button purple, Zip Session purple once both are reset; the Shots card covers its grid in a small window |

Game matrix:

| Game | Input device | Mode | Display mode | WF3 | WF4 | WF5 | Notes |
|---|---|---|---|---|---|---|---|
| NoClip (noclip.website) | Keyboard | Arrow keys | Maximized browser, primary display | ✓ 2026-11-04 | – | – | 7 × 13 Session, 91 Shots |
| Star Wars Battlefront 2 (2017) | Mouse | Move (raw input) | Borderless, primary display | – | ✓ 2026-11-04 | – | Offline Arcade only; 6 × 13 Sessions, 78 Shots |
| Overwatch | Gamepad | Right stick | Primary display | – | – | ✓ 2026-11-07 | Practice range; 5 × 7 Session, 35 Shots |

## Release (issue 21)

Two test tags, `v0.0.1-test.1` and `v0.0.1-test.2`, pushed on `feat/21-release`; their draft Releases and tags were deleted afterwards.

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| RL1 | Draft Release | Push a tag `v*` | One draft pre-release with `Panowww.dmg` and `Panowww-setup.exe`, both jobs green | Pass, twice (2026-11-08, aa1a9bb) |
| RL2 | Stable designated requirement | Compare the *show the designated requirement* step of two runs | Both read `identifier "com.respawwwn.panowww" and certificate root = H"ebcaea28…7bae"`, never a `cdhash` | Pass (2026-11-08, aa1a9bb) |
| RL3 | Permissions survive an update | Install the first DMG, grant Accessibility and Screen Recording, install the second over it, relaunch | Both permissions still green, nothing to grant again | Pass (2026-11-08, aa1a9bb) |
| RL4 | Gatekeeper | Open a downloaded DMG, then the app the first time | macOS warns once for each and *Open Anyway* gets through, as the README says (not notarized) | Pass, expected (2026-11-08, aa1a9bb) |
| RL5 | Windows install from the Release | Download `Panowww-setup.exe` from the published Release and run it over an earlier install | UAC prompt, the app replaces the earlier one and starts with the settings kept | Pass (2026-11-08, v0.1.0) |
| RL6 | macOS install from the Release | Download `Panowww.dmg` from the published Release, install, run a Session | The app opens after the Gatekeeper warnings and the Session completes | Pass (2026-11-08, v0.1.0) |
