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
| I2 | Key hold | `turn_test keyboard KeyD 120 13` with the game's turn key, landmark picked | The camera turns 13 times and ends on the landmark | |
| I3 | Mouse move | `turn_test mouse move 840 13` on a mouse-look game | Same as I2; note the delta that closes the turn | |
| I4 | Mouse drag | `turn_test mouse drag right 840 13` on the drag-camera reference game | The cursor is parked, the camera orbits 13 times and ends on the landmark | Pass via the app's Turn test on NoClip (mouse drag, left button): the camera orbits and the result line appears (2026-11-01, f46c8df) |
| I5 | Stop releases everything | Start I4, press Ctrl+C after two Steps | No key or button stays held (the game is not dragging on its own) | |

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
| O7 | Second display | Draw a zone on a second display with a different scale, run a Session | The overlay covers that display, the frame matches the zone there | Second display not available on 2026-11-01; planned with one |
| O8 | Release build transparency | `pnpm tauri build`, run the bundle, *Show zone* | The overlay window is transparent (no white sheet over the game); if it is white, enable `app.macOSPrivateApi` in `tauri.conf.json` and the `macos-private-api` feature of `tauri` | |

## macOS · full flow (issue 19)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| F1 | Session row actions | Sessions tab: Zip, Reveal, Delete on one row | The zip appears next to the folder; Finder opens on it; Delete removes the folder and its zip | Pass (2026-11-01) |
| F2 | Share → Import, same machine | Game → Share to a file; sidebar → Import that file | The `.panowww.json` is written; Import shows Replace / Keep both; Keep both adds the Game under a renamed copy | Pass (2026-11-01) |
| F3 | Settings folder | Settings → Change the Sessions folder, run a Session, change it back | The Session lands in the new folder; the tables follow | Pass (2026-11-01) |
| F4 | Calibrate tools | Step once (Turn and Pitch), Pitch test | One Step moves the camera; the Pitch test tilts from the start pole and the result line appears | Pass (2026-11-01) |
| F5 | Update bar | Against a newer Release once the repository has one (issue 21) | The bottom bar links the release page; the Settings switch hides it | Waits for issue 21 |
| F6 | Onboarding on a fresh install | Release bundle, `tccutil reset All com.respawwwn.panowww`, launch | Both rows red; *Open System Settings* on each shows the system prompt and opens the right pane; a row turns green as soon as its switch is ticked and the window comes back to the front; Screen Recording asks for a relaunch, then *Continue* is enabled | Pass on the release build after a TCC reset: both rows turn green live, Continue enabled after the Screen Recording relaunch (2026-11-01, 51bda69) |
| F7 | No Input Monitoring | Same run, watch the prompts | macOS never asks for Input Monitoring: Panowww only posts events, which the Accessibility grant covers | Pass after a TCC reset: no Input Monitoring prompt (2026-11-01). The first run had one, accepted at 14:29:52, trigger not found; watch for it on fresh installs |

## macOS · Capture zone picker (issue 13)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| P1 | One window per display | Setup → *Set zone…* with two displays connected | A borderless window covers each display with a dimmed screenshot of it, `Display n · w × h @sx` top left; the one holding the current zone (else the first) has the focus | Pass with one display, after two fixes found on this run: the window is now a transparent panel above the menu bar over the live, dimmed display (no screenshot), built on the main thread (the first build crashed on a tokio worker) (2026-11-01) |
| P2 | Drawing | Drag on the picture, then drag a corner handle, then drag the rectangle body | The purple rectangle follows with its size chip; handles resize from the opposite corner; the body moves and stops at the display's edge; X / Y / W / H follow and edit it | Pass (2026-11-01, f46c8df) |
| P3 | Two scales | Draw a zone on a 2× display, Save; then on a 1× display, Save | Setup shows `Display n · w × h at (x, y)` in points for each; `local-games.json` stores `displayW` / `displayH` / `scale` of the display drawn on; a Session's Shots are `w × scale` pixels wide | Second display not available on 2026-11-01; planned with one |
| P4 | Show frame on screen | Tick the checkbox while drawing, then Save | The overlay frame appears exactly on the rectangle (through the picker's dim); it goes away with the windows | Pass (2026-11-01, f46c8df) |
| P5 | Cancel and Esc | *Cancel* in one window; open again and press Esc in the other | Every picker window closes, the zone is unchanged, Setup is usable again | Found: Esc did nothing (the panel was not the key responder). Fixed: the focused picker window takes the keyboard. Retest: pass, Esc and Cancel both close (2026-11-01) |
| P6 | Display changed | Save a zone, change that display's resolution, reopen Setup | The amber *Display n is gone or its resolution changed* box; *Change zone…* opens the picker on the display as it is now | Second display not available on 2026-11-01; planned with one |
