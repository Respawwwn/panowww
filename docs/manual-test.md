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
