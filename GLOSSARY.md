# Glossary

**Panowww**
The app: captures 360° panorama source shots inside video games by driving the in-game camera and screenshotting a zone of the screen. Made by Respawwwn.

**Game**
A shareable, self-contained description of how to capture panoramas in one game: input device, camera movements, delays, starting pole. One Game = one file. Two setups for the same title are simply two Games with different names.
_Avoid_: profile, config, preset.

**Input device**
How Panowww moves the in-game camera for a Game: keyboard, mouse or gamepad.

**Capture zone**
The rectangle of the screen that is captured. Belongs to the local machine, not to the Game: remembered per Game on this machine, never shared with it.
_Avoid_: region, area.

**Session**
One full capture run of a Game, producing a folder containing only that run's shots. Starting over means starting a new Session.

**Shot**
One screenshot of the Capture zone taken during a Session.
_Avoid_: capture (ambiguous), screenshot.

**Test**
A run of a Game's camera movements without taking Shots, used to verify the Game before a Session. Creates no Session.
_Avoid_: dry-run.

**Nadir / Zenith**
Straight down / straight up. The pole where a Session starts.

**Step**
One camera movement between two Shots, horizontal or vertical. Its size is calibrated by hand per Game.

**Row**
The Shots taken at one camera pitch during a full horizontal turn.

**Overlap**
The share of image two neighbouring Shots have in common. Stitching needs enough of it, but not too much.
