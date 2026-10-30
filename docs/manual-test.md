# Manual tests

What only a human with a game can verify. One table per OS; the reference
game is the drag-camera macOS game of the human (name to fill in). Record
the date and the build (commit) next to every result.

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
| I1 | Accessibility check | `turn_test check` before and after granting | `false` before, `true` after; the request shows the system prompt once; System Settings opens on the Accessibility list | |
| I2 | Key hold | `turn_test keyboard KeyD 120 13` with the game's turn key, landmark picked | The camera turns 13 times and ends on the landmark | |
| I3 | Mouse move | `turn_test mouse move 840 13` on a mouse-look game | Same as I2; note the delta that closes the turn | |
| I4 | Mouse drag | `turn_test mouse drag right 840 13` on the drag-camera reference game | The cursor is parked, the camera orbits 13 times and ends on the landmark | |
| I5 | Stop releases everything | Start I4, press Ctrl+C after two Steps | No key or button stays held (the game is not dragging on its own) | |

Game matrix:

| Game | Input device | Mode | Display mode | I2 | I3 | I4 | Notes |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

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
| C1 | Screen Recording check | `shot check` before and after granting, relaunch between | `false` before, `true` after; the request shows the system prompt once; System Settings opens on the Screen Recording list | |
| C2 | Displays | `shot list` with one and two displays | Each display once, size in points, scale 2 on Retina, the id stable across a relaunch and a replug | |
| C3 | Zone Shot | `shot 0 100 100 1600 900` with a Panowww window and the terminal over the zone | `shot.png` is 3200 × 1800 px on Retina, shows the game, not the Panowww window, no cursor | |
| C4 | Timings at 1440p | Same, zone 2560 × 1440 | Note the first and second shot times (spike: 95–155 ms then 30–47 ms) and the PNG time | |
| C5 | Preview | Same run, the three preview lines | About 6 frames in 3 s, 30–60 KB each, none after stop | |
| C6 | macOS 26 picker alert | First capture of a fresh build | The "bypass the system private window picker" alert appears once; after Allow, captures work and the alert is gone | |

## macOS · app shell (issue 11)

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| A1 | Start shortcut with another app focused | Select a Game with a zone, switch to the game, press Ctrl+Shift+8 | The countdown starts in Panowww (3 s) while the game keeps the focus; the game shows no reaction to the keys | |
| A2 | Stop shortcut | During the countdown and during a Session, press Ctrl+Shift+9 | Countdown: cancelled, no folder created. Session: `interrupted`, the Shots taken are in the folder, nothing stays held | |
| A3 | Rebinding | Settings → change Start to another combo, Save | The old combo does nothing, the new one starts; a combo already taken by the system shows the inline error | |
| A4 | Quit during a Session | Cmd+Q while a Session runs | The app quits within a second; `session.json` says `interrupted` with the Shots on disk | |

## macOS · overlay (issue 12)

Reference game in a full-screen Space on the zone's display. The overlay is a `tauri-nspanel` non-activating panel; the spike validated the same setup.

| # | Check | How | Pass | Result |
|---|---|---|---|---|
| O1 | Show zone | Setup → *Show zone*, switch to the game (full-screen Space) | The lavender frame sits exactly on the zone over the game; the game keeps the focus (keep moving the camera with the real mouse); clicks inside the frame reach the game; *Hide zone* removes it, as does leaving the Game | |
| O2 | Countdown digit | Start a Session with the shortcut from the game | 3, 2, 1 centred in the frame over the game, the game never loses the focus, the Panowww window does not come forward | |
| O3 | Progress | Let the Session run | A 2 px bar fills along the top inner edge and the `k / n` pill top right follows every Shot; nothing is drawn outside the zone | |
| O4 | End pill | Let the Session finish; run another and stop it with the shortcut | `Done · n Shots` for about 2 s, then the overlay is gone; `Stopped · k / n` (amber) after a stop | |
| O5 | Not in the Shots | Open the Session folder | No frame, digit, bar or pill in any PNG | |
| O6 | Cannot start | Revoke Screen Recording, start a Session | The error state shows in the window and no overlay appears (or a frame shown by *Show zone* goes away) | |
| O7 | Second display | Draw a zone on a second display with a different scale, run a Session | The overlay covers that display, the frame matches the zone there | |
| O8 | Release build transparency | `pnpm tauri build`, run the bundle, *Show zone* | The overlay window is transparent (no white sheet over the game); if it is white, enable `app.macOSPrivateApi` in `tauri.conf.json` and the `macos-private-api` feature of `tauri` | |
