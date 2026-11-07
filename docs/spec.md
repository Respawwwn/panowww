# Panowww v1 specification

**Status: final, 2026-11-01.** Every decision below was taken on the planning map (wayfinder tickets 001–016) and is folded here with its later amendments applied. The Windows facts were verified on the Windows machine on 2026-11-01; [§15](#15-pending-windows-facts) records what was found and the decisions taken from it.

Companion files: [GLOSSARY.md](../GLOSSARY.md) (vocabulary, binding), [docs/adr/](adr/) (architecture decisions), [docs/release.md](release.md) (release process), [docs/implementation-plan.md](implementation-plan.md) (the ordered issues that implement this spec).

## 0. How to read this

- Written for Claude Code implementation sessions. Each issue in the implementation plan names the sections it needs; read those, not the whole file.
- Glossary terms are capitalised as in the UI: Game, Session, Shot, Sample, Step, Row, Overlap, Test, Capture zone, Display preview, Input device. Use them in code identifiers too (`Shot`, `Row`, `CaptureZone`), never *profile*, *region*, *screenshot*, *dry-run*.
- **App constant** = a value fixed in code, not in the Game file and not in Settings. They are gathered in [Appendix A](#appendix-a-app-constants).
- "Verified 2026-11-01" marks a Windows fact settled by the spike pass described in §15.
- *Decided at assembly* marks the few gaps the assembling session filled itself ([Appendix B](#appendix-b-decided-at-assembly)). They are cheap to reverse; veto them on the plan if wrong.

---

## 1. Product

### 1.1 What Panowww is

Panowww captures the source Shots of a 360° panorama inside a video game: it drives the in-game camera (keyboard, mouse or virtual gamepad), takes a lossless screenshot of a Capture zone after every Step, and leaves a folder (or a zip) ready to stitch in **PTGui or Autopano Giga**. Open source (MIT), Rust, macOS and Windows, made by Respawwwn. It replaces the Python script [game-pano-capture](https://github.com/Respawwwn/game-pano-capture) (branch `dev` is the prior art; no license, same author).

Tagline (`about.tagline`): *Captures the Shots of a 360° panorama inside a video game, ready to stitch.*

### 1.2 Scope of v1

In:
- Games as shareable JSON files; create, edit, duplicate, share (export), import, delete.
- Capture zone drawn with the mouse, remembered per Game on this machine.
- Calibration tools: Turn test, Pitch test, Step once, Check overlap, relative adjust buttons.
- Sessions with a countdown, a non-activating overlay, global shortcuts, live thumbnails, Display preview, end summary with zip / reveal.
- Sessions history per Game and global, with delete.
- First-launch onboarding for OS prerequisites; Settings; About; passive update check.
- Unsigned (macOS: self-signed) builds on GitHub Releases, README with non-technical install steps.

Out of scope (ruled out on the map; a fresh effort if ever):
in-app stitching; Linux; gamepad on macOS; a website; resuming an interrupted Session; importing the old `game_config.json`; FOV-based Shot planning and automatic Overlap measurement; Apple notarization and Windows code signing; Intel Macs, Windows ARM64, a portable Windows build; package managers (Homebrew cask is the v1.x candidate) and an in-app updater; exclusive fullscreen and HDR capture (tips and a warning only); macOS older than 14; archiving the old Python repo.

### 1.3 Platforms and requirements

| | macOS | Windows |
|---|---|---|
| Minimum | macOS 14, Apple Silicon only | Windows 10 1903+ / 11, x64 only |
| Permissions | Accessibility, Screen Recording | Runs **always elevated** (UAC prompt at each launch) |
| Gamepad | Unavailable (UI says so) | Virtual Xbox 360 pad via ViGEmBus 1.22.0, user-installed |
| Game display mode | Borderless windowed | Borderless windowed |
| Artifact | `Panowww.dmg` (`aarch64-apple-darwin`) | `Panowww-setup.exe` (NSIS, per-user, WebView2 bootstrapper) |

### 1.4 Principles

- **The game keeps the focus.** Nothing Panowww does during a countdown or a run may activate a window: start/stop come from global shortcuts, feedback comes from a non-activating, click-through overlay, and the app window never raises itself.
- **Performance must not suffer while a game runs.** Capture and encoding happen off the Session thread; the webview stays idle; the Display preview can be switched off.
- **Empirical movement model.** Step sizes are calibrated by hand per Game with the tools; no FOV maths, no computer vision.
- **Local stays local.** The Capture zone, the Sessions folder and the shortcuts belong to the machine; a Game file never carries them.
- **One pure core.** Everything that decides *what happens* in a run is OS-free and tested with fakes ([ADR 0001](adr/0001-session-logic-in-pure-core.md)).

---

## 2. Domain model

Vocabulary is in [GLOSSARY.md](../GLOSSARY.md). Relationships:

- A **Game** (file) describes one setup: Input device and its parameters, grid (`shotsPerRow` × `rows`), two delays, start pole. Several setups for one title are several Games.
- A Game on this machine has machine-local state keyed by its `id`: the **Capture zone** (display, rect, scale).
- A **run** is one execution of the Game's movements. Five run kinds:

| Run kind | Shots | Creates a Session | Started from | Countdown | Stream (preview) open |
|---|---|---|---|---|---|
| **Session** | yes, N × R | yes | global shortcut or *Start Session* | 3 s (shortcut) / 5 s (button) | yes |
| **Turn test** | no | no | Calibrate card | 5 s | yes |
| **Pitch test** | no | no | Calibrate card | 5 s | yes |
| **Step once** (turn or pitch) | no | no | Calibrate card | 5 s | yes |
| **Check overlap** (turn or pitch) | two in-memory **Samples** | no | Calibrate card | 5 s | yes |

- A **Session** produces a timestamped folder holding only its Shots (`rNN_cNN.png`) and a `session.json`. Row 1 is the start pole; column 1 is the start heading.
- A **Shot** is one PNG of the Capture zone. A **Sample** is an in-memory image of the zone taken outside a Session (Check overlap, screen-unchanged detector), never saved. A **Display preview** frame is a reduced image of the whole display, shown live, never saved.

---

## 3. Stack

### 3.1 Rust side

| Piece | Choice | Notes |
|---|---|---|
| Toolchain | Rust stable pinned in `rust-toolchain.toml` (1.99.x at planning time), edition 2024 | `unsafe` forbidden in `core` and `store` |
| Shell | **Tauri 2.x** (2.12.x; no 3.0 alpha) | All logic in Rust; the webview renders only |
| Overlay | macOS: `tauri-nspanel` 2.1.x. Windows: tao window flags (`focusable: false`, ignore cursor events), verified 2026-11-01 | §10 |
| Global shortcuts | `tauri-plugin-global-shortcut` (`global-hotkey`: Carbon `RegisterEventHotKey` / `RegisterHotKey`) | §10 |
| Input | own module: `objc2-core-graphics` 0.3 (CGEvent); `windows` 0.62 (`SendInput`); `vigem-client` 0.1.x behind our own trait | §8 |
| Capture | own module: **`objc2-screen-capture-kit` 0.3** (not the `screencapturekit` crate, whose `apple-metal` dependency needs the macOS 26 SDK); `windows-capture` 2.0.x on Windows Graphics Capture, verified 2026-11-01 | §9 |
| Images | `png` 0.18 `Compression::Fast`, RGB8 no alpha; `fast_image_resize` for thumbnails, previews and the detector; JPEG encoder for previews / thumbnails | §9 |
| Errors / logs | `thiserror` per crate, no `anyhow`; `tracing` + rotating file in Tauri's logs folder | §4.7 |
| Tests / quality | `cargo nextest`, `insta`, `proptest`, clippy pedantic, `cargo-deny`, `typos` | §4.8 |

Pin crate versions at implementation time to the latest compatible release; the numbers above are the ones researched.

### 3.2 UI side

Svelte 5 + TypeScript + Vite SPA (no SvelteKit, no router: state-based navigation), pnpm, Biome, Vitest with `@tauri-apps/api/mocks` (`mockIPC`). TS types generated from Rust with `ts-rs` into `ui/src/lib/generated/`; a hand-written `ui/src/lib/api.ts` wraps `invoke`. Inter (OFL 1.1) bundled. One strings module holds every user-visible string (§13).

### 3.3 OS APIs

| Concern | macOS | Windows |
|---|---|---|
| Key press | `CGEvent` keyboard, `kVK_*` codes | `SendInput`, `KEYEVENTF_SCANCODE` |
| Mouse move | `MouseMoved` / `*MouseDragged` with `kCGMouseEventDeltaX/Y` set | `MOUSEEVENTF_MOVE` (relative, never `ABSOLUTE`) |
| Gamepad | none | ViGEmBus + `vigem-client` (`BusNotFound` = driver missing) |
| Input permission | `AXIsProcessTrustedWithOptions`, `CGPreflightPostEventAccess` / `CGRequestPostEventAccess` | app manifest `requireAdministrator` (UIPI) |
| Capture | ScreenCaptureKit: `SCScreenshotManager` for Shots and Samples, low-rate `SCStream` for the preview, content filter excluding our own windows | Windows Graphics Capture, one session, frames cropped to the zone; own windows `WDA_EXCLUDEFROMCAPTURE` (verified 2026-11-01) |
| Capture permission | `CGPreflightScreenCaptureAccess` / `CGRequestScreenCaptureAccess` | none |
| Overlay | `NSPanel` non-activating, `CanJoinAllSpaces \| FullScreenAuxiliary \| Stationary`, level ScreenSaver, `ignoresMouseEvents` | `WS_EX_NOACTIVATE \| WS_EX_TRANSPARENT \| WS_EX_LAYERED` via tao (verified 2026-11-01) |
| Reveal / open settings | Finder, `x-apple.systempreferences:` URLs | Explorer `/select`, `ms-settings:` |

---

## 4. Architecture

### 4.1 Crates

Cargo workspace, `version.workspace = true`, `publish = false`, one `[workspace.lints]`. Every crate starts with a `//!` doc stating its role and interface.

| Crate | Role | OS code |
|---|---|---|
| `panowww-core` | Game schema, validation, migration; **capture plan**; run **Runner**; screen-unchanged comparison; duration estimate; the **ports** it consumes. `#![forbid(unsafe_code)]`. Fakes behind feature `test-support`: `RecordingCamera`, `FakeGrabber`/`FakeStream` (numbered images), `FakeClock` (advances instantly), in-memory `ShotSink`. | none |
| `panowww-input` | `Camera` adapters: macOS `CGEvent`, Windows `SendInput` + ViGEm. Own prerequisite checks: Accessibility, ViGEmBus. Shared `KeyboardEvent.code` → `kVK_*` / scan-code table. `unsupported` module so the workspace builds on Linux. | yes |
| `panowww-capture` | `Grabber`/`Stream` adapters: ScreenCaptureKit, WGC. Display enumeration (stable ID, size, scale, HDR flag on Windows). Screen Recording check / request. Display preview production. `unsupported` module. | yes |
| `panowww-store` | File system: app data folder, Games folder, `settings.json`, `local-games.json`, Session folders and `session.json`, PNG worker, thumbnails, zip, reveal, delete, sizes. Implements `ShotSink`. `#![forbid(unsafe_code)]`. | paths only |
| `panowww-app` | Thin Tauri shell: commands, the single `Runner` owner, countdown, pre-flight, overlay, global shortcuts, windows, onboarding aggregation, update check, `AppError`. | window glue |

### 4.2 Ports (traits defined in `core`, implemented outside)

Adapters are deliberately dumb ([ADR 0001](adr/0001-session-logic-in-pure-core.md)): they perform primitives; `core` decides sequences.

```rust
/// Input device primitives. Waits never happen here: core uses Clock.
pub trait Camera {
    fn key_down(&mut self, key: PhysicalKey) -> Result<(), InputError>;
    fn key_up(&mut self, key: PhysicalKey) -> Result<(), InputError>;
    fn mouse_delta(&mut self, dx: i32, dy: i32) -> Result<(), InputError>;   // relative, one sub-move
    fn button_down(&mut self, b: MouseButton) -> Result<(), InputError>;
    fn button_up(&mut self, b: MouseButton) -> Result<(), InputError>;
    fn park_cursor(&mut self, at: Point) -> Result<(), InputError>;          // absolute, once per drag Step
    fn stick(&mut self, x: f32, y: f32) -> Result<(), InputError>;           // right stick, −1..1, Windows only
    fn release_all(&mut self) -> Result<(), InputError>;                     // called first on every stop
}

/// An open capture of one display, cropped to the Capture zone for Shots.
pub trait Stream {
    /// A fresh frame whose timestamp is after `not_before`, cropped to the zone. RGB8 + size.
    fn shot(&mut self, not_before: Instant) -> Result<Frame, CaptureError>;
    fn set_preview(&mut self, on: bool);          // Display preview source on/off, immediate
    fn stop(self: Box<Self>);
}
/// Opened by `app` (not by core) before the countdown; see §4.5.
pub trait Grabber {
    fn start(&mut self, display: DisplayId, zone: Rect, preview: PreviewSink) -> Result<Box<dyn Stream>, CaptureError>;
}

pub trait ShotSink {
    fn begin(&mut self, meta: &SessionStart) -> Result<(), StoreError>;      // creates the folder, writes session.json (running)
    fn shot(&mut self, shot: ShotRecord, frame: Frame) -> Result<(), StoreError>; // queues PNG + live thumbnail
    fn finish(&mut self, outcome: &RunOutcome) -> Result<(), StoreError>;    // flushes, updates session.json
}

pub trait Clock {
    fn now(&self) -> Instant;
    /// Sleeps `d` or returns early with `false` when `cancel` is raised.
    fn wait(&self, d: Duration, cancel: &CancelFlag) -> bool;
}
```

`PhysicalKey` is the W3C `KeyboardEvent.code` string validated against the supported table (§8.1). Frames are `{ width, height, rgb: Vec<u8> }`.

The **Overlay** port lives in `app` (not `core`): `show_countdown(zone, n)`, `show_frame(zone)`, `progress(taken, total)`, `end_pill(text)`, `hide()`. Every macOS call is marshalled to the main thread (`run_on_main_thread`); calling AppKit from the Session thread traps.

### 4.3 Capture plan

`core` turns a Game and a run kind into a `CapturePlan`: a finite `Vec<Op>` with

```
Op::Move(Step)             Step = Turn | Pitch, carrying the device-specific motion computed from the Game
Op::Wait(Duration)
Op::Shot { row, col }      Sessions only
Op::Sample(slot)           Samples for Check overlap (First, Second) and the detector (BeforeFirstStep, AfterFirstStep)
```

Sub-moves, holds, drag press/release, inversion and the closing turn are expanded by `core` at run time from `Step`, not stored in the plan. The duration estimate is the sum of the plan with the constants of §7.6. Each run kind's plan is insta-snapshotted per sample Game. *Decided at assembly*: `Op::Sample` is the one addition to the researched plan shape, so the detector and Check overlap are plan-driven and testable like Shots.

### 4.4 Runner and runtime

```rust
pub fn run(
    plan: &CapturePlan, camera: &mut dyn Camera, stream: &mut dyn Stream,
    sink: &mut dyn ShotSink, clock: &dyn Clock, cancel: &CancelFlag, progress: &dyn Fn(Progress),
) -> RunOutcome
```

- Runs on a **dedicated OS thread** (blocking, precise timing), no async runtime. The cancel flag is checked between primitives and inside every `Clock::wait`.
- `Progress` events: `Started { expected }`, `Shot { index, row, col, capture_ms, thumbnail: Jpeg }`, `Moved { row, col }`, `ScreenUnchanged`, `Finished(RunOutcome)`. Preview frames travel on the same channel from the capture adapter (`Progress::Preview { jpeg, display_size, zone }`).
- `RunOutcome`: `Completed { shots }`, `Interrupted { shots }`, `Failed { shots, error }`. For Tests and Check overlap the same enum with `shots = 0`; Check overlap additionally returns its two Samples in memory.
- `app` owns one `Runner` with states `Idle → Countdown → Running → Stopping → Idle`. Shortcuts, UI buttons and window close all go through it. Start while not `Idle` is ignored. Quitting the app stops cleanly and marks a running Session `interrupted`.
- One **PNG worker thread** in `store` with a bounded queue; the Session thread never encodes.

### 4.5 Countdown, pre-flight and stream opening (in `app`)

At the start of a countdown, in this order, before the first tick is drawn:
1. **Pre-flight**: macOS `AXIsProcessTrusted()` for every Game; Windows ViGEmBus present for gamepad Games (`BusNotFound` from the connect call; the missing-driver path was not exercised on 2026-11-01, the driver being installed, so issue 23's manual test covers it).
2. **Open the capture `Stream`** on the zone's display (absorbs the ≈ 3× first-Shot warm-up; the preview is visible during the 3-2-1).
3. Any failure above → countdown **cancelled, nothing created**; the countdown screen shows the error state (§11.9) with the reason's action; the overlay is hidden.

Then the countdown runs (3 s / 5 s), the overlay draws the digit in the zone, and at zero the started `Stream` is handed to the Runner. `app` closes the stream when the run ends; the end screen has no preview.

### 4.6 Screen-unchanged detector

Pure function in `core`: `fn unchanged(a: &Frame, b: &Frame) -> bool` = downscale both to 160 px on the long side, greyscale, mean absolute difference < 1 % of full scale. One-sided: unchanged proves nothing moved; changed proves nothing.
- Turn test, Pitch test, Step once: Samples `BeforeFirstStep` (just before the first Step) and `AfterFirstStep` (after its `waitAfterMove`), through the Shot path at full resolution, independent of the Display preview switch. First Step only.
- Session: compare Shots 1 and 2 already in memory before PNG encoding; no extra Sample.
- Result: `Progress::ScreenUnchanged`. The UI decides what to show (§11.5, §11.10). Nothing is written to `session.json`. No automatic stop.
- No heuristic for exclusive fullscreen, focus elsewhere, SDL / GameController-mouse games, anti-cheat, raw-input games: checklist only (§13.7).

### 4.7 Errors and logs

- Library crates: `thiserror` enums typed per module (`InputError`, `CaptureError`, `StoreError`, `GameError`…). No `anyhow`.
- Command layer: one serialisable `AppError { kind: AppErrorKind, message: String }` with plain-English messages, the only error shape the UI sees. Validation errors carry a list of `{ field, message }`.
- `tracing` everywhere; rotating log file in Tauri's logs folder; Settings → *Reveal logs*.

### 4.8 Repo layout, tooling, quality

```
Cargo.toml                # workspace, [workspace.lints], version (single source of truth)
rust-toolchain.toml
crates/core/ crates/input/ crates/capture/ crates/store/
app/                      # Tauri crate, tauri.conf.json, icons, capabilities
ui/                       # Svelte: src/{screens,components,lib/api.ts,lib/strings.ts,lib/generated/}
assets/brand/             # Respawwwn mark (NOTICE), Inter + OFL.txt
docs/spec.md docs/implementation-plan.md docs/release.md docs/manual-test.md docs/adr/
GLOSSARY.md AGENTS.md (CLAUDE.md -> symlink) README.md CONTRIBUTING.md LICENSE NOTICE
.github/workflows/ci.yml .github/workflows/release.yml .github/ISSUE_TEMPLATE/
justfile                  # just check / test / dev / lint
```

Quality rules:
- `[workspace.lints]`: clippy `all = deny`, `pedantic = warn` (CI runs `-D warnings`, so effectively deny), `undocumented_unsafe_blocks`, `dbg_macro`, `unsafe_op_in_unsafe_fn` = deny; workspace-allowed `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`.
- Tests: `cargo nextest` + `cargo test --doc`; insta for Game files and capture plans; proptest for Step / Overlap / estimate maths with regression files committed; the whole Session runs in tests through the fakes. `tauri::test` mock runtime for commands; Vitest + `mockIPC` for the UI.
- OS adapters are compiled and linted on CI but exercised by hand through `docs/manual-test.md` (OS × Input device × reference game; the human's drag-camera macOS game is the first reference, name to fill in).
- CI (`ci.yml`): matrix macOS + Windows (rustfmt, clippy `-D warnings`, nextest, `typos`, `cargo-deny`), plus a fast Linux job for `core` + `store`. `Swatinem/rust-cache`, `taiki-e/install-action`, actions pinned by SHA, read-only permissions.
- Later (not v1 issues): `cargo-llvm-cov` report, `cargo-shear`, `prek` hooks, WebdriverIO e2e, `zizmor`.

### 4.9 ADRs

- [0001 Session logic lives in a pure core; OS adapters only perform primitives](adr/0001-session-logic-in-pure-core.md).

---

## 5. Game file

One Game = one JSON file, envelope + device-tagged `input` block. camelCase keys, durations in integer milliseconds.

```json
{
  "format": "panowww-game",
  "version": 1,
  "id": "3f6c1e2a-…",
  "name": "Elden Ring – photo mode",
  "notes": "FOV 90, HUD off",
  "startPole": "nadir",
  "shotsPerRow": 13,
  "rows": 6,
  "waitAfterMoveMs": 300,
  "waitAfterShotMs": 300,
  "input": {
    "device": "mouse",
    "mode": "drag",
    "dragButton": "right",
    "turnDelta": 840,
    "pitchDelta": 610,
    "invertVertical": false
  }
}
```

| Field | Rule |
|---|---|
| `format` | `"panowww-game"`, required |
| `version` | integer, currently `1`. Older → migrated on load and rewritten. Newer → refused ("Update Panowww"). Unknown fields → rejected. |
| `id` | UUID v4 made at creation, kept through share/import. Keys all machine-local state. |
| `name` | 1–60 chars, trimmed, unique on the machine case-insensitively. Display only. |
| `notes` | optional, ≤ 2000 chars |
| `startPole` | `"nadir"` \| `"zenith"`, default `nadir`. Vertical direction follows from it. |
| `shotsPerRow` | 3–72. Rotation always to the right; the Nth turn closes back to the start heading. |
| `rows` | 2–36, poles included. No special pole handling. |
| `waitAfterMoveMs`, `waitAfterShotMs` | 0–10000, Game-level, device-independent |
| `input.device` | `"keyboard"` \| `"mouse"` \| `"gamepad"`; the block holds only that device's fields. Switching device in the editor replaces the block with that device's defaults. |

`input` blocks:

| Device | Fields |
|---|---|
| `keyboard` | `turnRightKey`, `lookUpKey`, `lookDownKey`: W3C `KeyboardEvent.code` physical keys (layout-independent), one key each, no modifiers, all distinct, no system / modifier-only keys. `turnHoldMs`, `pitchHoldMs`: 1–5000. |
| `mouse` | `mode: "move" \| "drag"`; `dragButton: "left" \| "right" \| "middle"` (drag only); `turnDelta`, `pitchDelta`: raw mouse units per Step, 1–100000; `invertVertical`. Sub-move size and interval are **app constants**. |
| `gamepad` (Windows only) | right stick; `stickDeflection` 0.1–1.0 (default 0.8); `turnHoldMs`, `pitchHoldMs` 1–5000; `invertVertical`. |

- **New Game defaults**: keyboard, `ArrowRight` / `ArrowUp` / `ArrowDown`, holds 100 / 100 ms, nadir, 13 Shots per Row, 6 Rows, delays 300 / 300 ms (overridable in Settings → New Game defaults for start pole and delays).
- **Validation**: same checks on load and in the editor, one plain-English message per field. Import of an invalid file is refused with the error list, nothing created. A damaged file in the Games folder stays listed as *Damaged* with Reveal and Delete, never silently dropped; it is re-read when it changes on disk.
- **Identity on import**: unknown `id` → added (name made unique with ` (2)`). Known `id` → the user chooses **Replace** (keeps local zone) or **Keep both** (new `id`, suffixed name). **Duplicate** = new `id`, name + ` copy`, no zone.
- **Cross-OS**: a gamepad Game imports on macOS, is shown with the amber banner and cannot run. Mouse values are not promised to match between OSes (tip says so).
- **Storage**: `<name>.panowww.json` in the Games folder (forbidden characters → `-`, ` (2)` on collision); the file name is not the identity and is renamed when the name changes. **Share** copies the file through a save dialog (no drag-and-drop anywhere). The Capture zone and other local state live in `local-games.json` (§6).

---

## 6. Local storage

Bundle identifier **`com.respawwwn.panowww`**, never to change (data folders, macOS permission grants and the signing designated requirement depend on it). All JSON is written atomically (temp file + rename), versioned and migrated like the Game file.

App data folder (Tauri `app_data_dir`):

```
games/<name>.panowww.json
settings.json
local-games.json
cache/thumbnails/<session-id>/rNN_cNN.jpg     # history thumbnails, generated on demand
```

`settings.json` (`format: "panowww-settings"`, `version: 1`):

| Key | Default |
|---|---|
| `sessionsFolder` | `~/Pictures/Panowww` |
| `shortcuts.start` / `shortcuts.stop` | `Ctrl+Shift+8` / `Ctrl+Shift+9` (stored as modifiers + `KeyboardEvent.code`) |
| `newGameDefaults.startPole`, `.waitAfterMoveMs`, `.waitAfterShotMs` | `nadir`, 300, 300 |
| `checkForUpdates` | `true` |
| `displayPreview` | `true` |
| `appearance` | `auto` (`light`, `dark`) |
| `onboarded` | `false`; `true` once onboarding was closed with *Later* or *Continue* (§11.11) |

`local-games.json` (`format: "panowww-local-games"`, `version: 1`): map Game `id` → `{ zone: { displayId, x, y, w, h, displayW, displayH, scale } }`. Coordinates are points on macOS, physical pixels on Windows; `displayId` is the OS's stable display identifier. At load, a zone whose display is gone or whose size/scale changed is reported as *display changed*; it is never rescaled, the user redraws it.

Session folder: `<sessionsFolder>/<Game name sanitised>/<YYYY-MM-DD_HH-mm-ss>/` (*decided at assembly*: this timestamp format sorts and is legal on both file systems), created at the **first Shot**, never during the countdown. Contents: `rNN_cNN.png` (two-digit, 1-based, row then column) and `session.json`:

```json
{
  "format": "panowww-session", "version": 1,
  "id": "…", "appVersion": "0.1.0",
  "startedAt": "…", "endedAt": "…",
  "status": "running | completed | interrupted | failed",
  "error": { "kind": "captureUnavailable | displayGone | permissionRevoked | shotFailed", "message": "…" },
  "game": { "…full Game snapshot at run time…" },
  "zone": { "displayId": "…", "x": 0, "y": 0, "w": 0, "h": 0, "displayW": 0, "displayH": 0, "scale": 2 },
  "shotsExpected": 78, "shotsTaken": 78,
  "shots": [ { "file": "r01_c01.png", "row": 1, "col": 1, "takenAt": "…", "captureMs": 41 } ]
}
```

Written with `status: "running"` at the first Shot, rewritten at the end. A Session still `running` at next launch is marked `interrupted`. Session size shown in tables is computed on demand (sum of the folder) and cached in memory. **Zip Session** writes `<folder name>.zip` next to the folder (store level, no compression beyond PNG's). **Delete** removes the whole folder (and its zip if present) after inline confirmation.

---

## 7. Capture sequence

### 7.1 Session

1. Trigger → pre-flight + stream open (§4.5) → countdown (3 s shortcut / 5 s button). Overlay shows the zone frame and the digit.
2. At zero: **first Shot at once** at the start pole (the countdown is the settling time).
3. Per Row, N times: `Shot → waitAfterShot → turn Step → waitAfterMove`. The Nth turn is the closing move back to the start heading.
4. After a Row that is not the last: `pitch Step → waitAfterMove`, next Row. The last Row still closes (camera ends at the start heading, at the opposite pole); no pitch after it.
5. Every move, closing and pitch included, gets its own `waitAfterMove`; no merged delay.

Steps per device:
- **Keyboard**: key down → hold `turnHoldMs` / `pitchHoldMs` → key up. Pitch key = `lookUpKey` or `lookDownKey` chosen from the start pole (nadir → up).
- **Mouse move**: `ceil(delta / 20)` sub-moves of ≤ 20 px, one every 8 ms, sign from direction and `invertVertical`.
- **Mouse drag**: park the cursor at the Capture zone centre → button down → 20 ms → sub-moves → 20 ms → button up. One complete drag per Step, never held across a Row; re-park before every Step.
- **Gamepad**: right stick to `±stickDeflection` on the axis → hold `turnHoldMs` / `pitchHoldMs` → stick to 0.

### 7.2 Triggers

- **Start shortcut** (default `Ctrl+Shift+8`, rebindable) always starts a **Session** of the Game selected in the app with the 3 s countdown. Ignored while anything runs, with no Game selected, or when Start is disabled (no zone, gamepad on macOS, damaged file).
- ***Start Session*** button: same, 5 s countdown.
- Tests, Check overlap and Step once start only from the Calibrate card (5 s countdown), never from the shortcut.
- **Stop shortcut** (default `Ctrl+Shift+9`) or in-app *Stop* / *Cancel*: **first** `release_all` (every held key, button, stick), then stop the run. During a countdown it cancels and nothing is created. Pressed while nothing runs: ignored.

### 7.3 End states and retry

All keep the Shots taken: `completed`; `interrupted` (stopped by the user, app quit, display preview irrelevant); `failed` (capture impossible, display gone, permission revoked, Shot failed twice). A failing Shot is **retried once** after a fresh `waitAfterShot`; a second failure fails the Session. The grid never continues with a hole.

### 7.4 Tests, Step once, Check overlap

All use the Game's `waitAfterMove` and **no** `waitAfterShot` (nothing to settle; a faster Test would make input-dropping games fail a Test the Session would pass). None creates a Session or writes anything.
- **Turn test**: N turn Steps (closing included) from the current position; the camera must land back on its landmark.
- **Pitch test**: R−1 pitch Steps from the start pole; the camera must land on the opposite pole. The user repositions by hand afterwards.
- **Step once**: one turn or pitch Step.
- **Check overlap** (turn or pitch variant): Sample → Step → `waitAfterMove` → Sample, from the **current** camera position (tip: aim at the horizon). Shown as an onion-skin with an alignment slider; Overlap = 1 − offset / size along the Step axis; verdict against the 25–35 % target (§11.8). Samples stay in memory.
- No "full test".

### 7.5 Calibration assistant

A guided loop per axis in the Calibrate card, in the recommended order, no blocking wizard:
1. Turn test → *Overshot* / *Undershot* buttons adjust the turn value **relatively** (fine ±2 %, coarse ±10 %, integer-rounded, clamped to the schema bounds; `turnDelta`, `turnHoldMs` or `stickDeflection`), re-run.
2. Check overlap (turn) → off-target Overlap means changing *Shots per Row* and going back to 1.
3. Pitch test → adjust the pitch value the same way.
4. Check overlap (pitch) → adjust *Rows*.
5. Step once (turn / pitch) to feel a Step.

Delays are not tool-calibrated (tips under the fields). The assistant edits the Game's own fields; nothing else is stored.

### 7.6 Duration estimate

```
total = countdown
      + N × R × (waitAfterShot + stepTime(turn) + waitAfterMove + 50 ms)
      + (R − 1) × (stepTime(pitch) + waitAfterMove)

stepTime = holdMs                                (keyboard, gamepad)
         = ceil(delta / 20) × 8 ms  (+ 40 ms in drag mode)   (mouse)
```

Shown in the Start card as `≈ 4 min 20 s · 78 Shots · 6 Rows × 13`, recomputed on every change; during a Session as remaining time from the measured per-Shot average.

---

## 8. Input injection

### 8.1 Keys

Games store keys as W3C `KeyboardEvent.code` (physical position). `panowww-input` holds one table `code → (macOS kVK, Windows scan code)` covering letters, digits, arrows, function keys, numpad, space, shift-less punctuation. Codes outside the table, modifiers alone and system keys are rejected by validation. Hold = down, `Clock::wait`, up.

### 8.2 Mouse

Relative moves only: macOS `MouseMoved` (move mode) or `<button>MouseDragged` (drag mode) events with `kCGMouseEventDeltaX/Y` set (captured-cursor games read only the deltas); Windows `MOUSEEVENTF_MOVE`. Cursor parking for drag: `CGWarpMouseCursorPosition` / `SetCursorPos` at the zone centre. Validated on macOS by the spike (20 px / 8 ms sub-moves moved a drag camera).

### 8.3 Gamepad (Windows)

Virtual Xbox 360 controller through ViGEmBus 1.22.0 + `vigem-client` behind our own trait (`vigem-rust` is the swap). ViGEmBus is archived since 2023-11 and never redistributed: onboarding links to the official release page. Verified 2026-11-01: the virtual pad is detected as an Xbox 360 controller and its sticks move (hardwaretester.com; no gamepad game tried yet). The bus's wait-ready call is unreliable on an old driver (1.17.333 answered Win32 483 and 259 at random while the pad was attached), so the adapter plugs **one pad per process at the pre-flight** of a gamepad Game, keeps it plugged across runs, treats the wait-ready answer as a log line (the countdown gives the system its seconds to enumerate), and the gamepad Game's setup shows *plugged* / *not plugged* (XInput user index). The virtual pad must be XInput player 1: physical controllers take the lower slots and most games read only the first, which is why Battlefront 2 (offline) showed no reaction to a pad plugged beside a physical Xbox One pad. Games that enumerate controllers only at start need one restart after the first countdown; `nothingMoved.gamepad` says both. `BusNotFound` as "driver missing" is unverified (driver present on the test machine); issue 23's manual test covers it.

### 8.4 Prerequisites and failure modes

- macOS Accessibility: checked at launch (onboarding), at every countdown (pre-flight). Unsigned builds lose the grant on update; the self-signed certificate (§14) keeps it.
- Windows: Panowww always runs elevated (UIPI drops input into elevated games otherwise). Driving a game launched as administrator was not tried on 2026-11-01 (no such game at hand); the README states that an elevated game needs the elevated Panowww, which is always the case.
- Known, undetectable: raw-input games on Windows that ignore `SendInput` (none met on 2026-11-01: Battlefront 2 took it), SDL / GameController-mouse games on macOS, anti-cheat. Covered by the *Nothing moved?* checklist.

---

## 9. Screen capture and Display preview

- One `Stream` per run on one display, opened by `app` before the countdown (§4.5). Shots and Samples are full-resolution frames cropped to the zone; the adapter guarantees a **fresh frame after the move** (`not_before` timestamp).
- **macOS**: `SCScreenshotManager` per Shot/Sample with a display content filter **excluding Panowww's own windows** (overlay never in a Shot); output size = points × scale; no cursor; SDR. Measured in release: first Shot 95–155 ms, then 30–47 ms; PNG `fast` ≈ 6 ms at 1080p, ≈ 10 ms at 1440p (≈ 25 ms expected at 4K). Preview = a separate low-rate `SCStream` whose output size is reduced in the SCK configuration, same filter.
- **Windows** (verified 2026-11-01 over NoClip at 1920×1080): one WGC session for the display, Shots cropped on the CPU, preview = downscaled copy of one frame in N; own windows `WDA_EXCLUDEFROMCAPTURE` (never in a Shot; no yellow border without a package manifest); frame timestamp checked against the move (every Shot's frame arrived 420–620 ms after it); no DXGI Desktop Duplication fallback needed; HDR detected per display → amber warning in the zone status. Measured in release at 1080p: over NoClip grab 5–21 ms and PNG `fast` 31–48 ms for ≈ 2 MB; over Battlefront 2 grab 40–85 ms and PNG `fast` 52–75 ms for 3.3–4.1 MB (5× the Mac's PNG time, so the PNG worker queue matters more). 1440p / 4K not measured (1080p displays only); by pixel count expect ≈ 2× and ≈ 4× those times. The 50 ms capture constant stays as the 1080p average.
- **Display preview**: 2 frames/s, longest side 640 px, JPEG ≈ 70 (30–60 KB), sent on the progress Channel with the display size and zone rect; **the UI draws the zone frame** over it. Encoded on the adapter's own callback thread, latest-wins. Not encoded while the main window is minimised or hidden. `set_preview(false)` cuts the source (no `SCStream`, no downscale). Preview failure while Shots work = placeholder, run continues, logged.
- **PNG**: `png` 0.18, `Compression::Fast`, RGB8, on the `store` worker thread (bounded queue). **Live thumbnails**: downscaled in memory to a small JPEG from the same buffer, sent on the progress Channel, never written to the Session folder. **History thumbnails**: generated on demand into `cache/thumbnails/`.
- **Permission**: `CGPreflightScreenCaptureAccess` / `CGRequestScreenCaptureAccess`; the app usually needs a relaunch after a grant (onboarding says so). **macOS 26** shows a "requesting to bypass the system private window picker" alert on the first capture of a new build even when granted; capture works, the alert lands in the first Shot; onboarding tells the user to click Allow first.

---

## 10. Overlay and global shortcuts

- **Overlay**: one transparent, always-on-top, click-through, **non-activating** window per run, sized to the zone's display. macOS: `tauri-nspanel` panel validated over a full-screen-Space game. Windows (verified 2026-11-01 over NoClip, maximized browser, both displays): tao `focusable: false` + ignore cursor events; the game kept focus and real-mouse camera control while the overlay showed, clicks on the frame reached it, countdown and frame drew in place. The **in-app countdown fallback** (the Session view already shows the digit and "Switch to the game") stays as the degraded path only.
- Drawing, all **inside the zone frame** (the zone may be the whole display; nothing is drawn outside it): frame `#a596ea` 2 px just outside the rectangle; countdown digit centred; during a run a 2 px progress bar along the top inner edge filling left→right (Shots taken / expected) and a `17 / 78` pill top right; at the end `Done · 78 Shots` / `Stopped · 47 / 78` for ≈ 2 s. Excluded from capture.
- **Shortcuts**: `tauri-plugin-global-shortcut`, defaults `Ctrl+Shift+8` (start) and `Ctrl+Shift+9` (stop), rebindable in Settings with a recorder. Never F12, Win-key combos, Option-only / Option+Shift-only on macOS; F-keys discouraged on Mac laptops (media keys without `fn`). Registered at launch and re-registered when changed; a combo that fails to register shows an inline error in Settings. Windows: `RegisterHotKey` fired with NoClip, Overwatch and Battlefront 2 focused (verified 2026-11-01), so no raw-input listener is added; a game that sets `RIDEV_NOHOTKEYS` falls back to the in-app Start button. Modifier-based defaults have a side effect in games that bind Ctrl or Shift (crouch, sprint); the recorder in Settings is the answer.

---

## 11. UX

One main window, macOS master/detail; minimum ≈ 1040 × 640, no responsive stacking. Separate windows: Capture zone picker, Check overlap, All Sessions, Settings (also `Cmd+,` / `Ctrl+,`), About, Onboarding. Everything is navigated by state in the Svelte app. No drag-and-drop anywhere (Windows runs elevated; UIPI blocks drops): file pickers and save dialogs only. Wireframes: variant D of `prototype/ux-wireframes/index.html` (branch `prototype/ux-wireframes`); visual reference: direction B on branch `prototype/visual-design` and the Claude Design canvas (private).

### 11.1 Window shell

- **Sidebar** (lavender): Panowww mark + wordmark; Games list, each row = name + minimalist stroke device icon (keyboard / mouse / gamepad), amber ⚠ for a damaged file, selected row purple with white text; toolbar *+ New* and *Import…*; footer *All Sessions*, Settings (⚙), About (?). Greyed to 45 % during a countdown or run.
- **Detail** for the selected Game: a toolbar with the segmented **Setup | Sessions**; then one full-width header row (title left; ghost *Duplicate* / *Share* / *Delete* right), then the content. *Start Session* lives in the Start card (§11.2), top right. *Decided at assembly*: the toolbar carries no second Start button.
- **Update bar**: app-shell bar at the **bottom of the window**, on every screen: `update.banner` with a link to the release page and ✕ to dismiss for this launch. Hidden when up to date, offline, or `checkForUpdates` is off.
- **Empty list**: centred `empty.*` with *New Game* / *Import Game…*.

### 11.2 Game Setup (two panes)

Left pane, collapsible cards with numbered lavender badges, one open at a time, header = title + one-line summary:
1. **Game & input**: Name, Notes (`setup.game.notes.placeholder`), Input device segmented control, then the device's fields. Keyboard: three key caps with *Change…* (a recorder that captures the next physical key), turn hold, pitch hold. Mouse: Move / Drag, drag button, turn delta, pitch delta, Invert vertical. Gamepad: amber `setup.input.gamepad.unavailableMac` box on macOS, deflection, holds, invert. Tips as in §13.1.
2. **Grid & timing**: Start pole, Shots per Row, Rows, Wait after move, Wait after Shot, tips as in §13.2.
3. **Start card** (purple block): estimate + config line (`≈ 4 min 20 s · 78 Shots · 6 Rows × 13`), the Sessions folder path on one ellipsised line, *Start Session* top right. Under the card, right-aligned, one per line: `setup.start.shortcutStart`, `setup.start.shortcutStop`. When disabled, the reason line (`setup.start.disabled.*`) replaces the shortcut lines.

Right pane, always visible:
- **Capture zone · this machine**: status line with green/red dot: `Display {n} · {w} × {h} at ({x}, {y})` or `setup.zone.notSet`; amber `setup.zone.displayChanged` / `setup.zone.hdr` boxes when relevant; *Set zone…* / *Change zone…*; *Show zone* / *Hide zone* toggle that draws the overlay frame alone, without a run, so the user can position the game.
- **Calibrate**: `setup.calibrate.countdown` on top; 1 · Turn test [Run] + `Overshot −10 % −2 %  +2 % +10 % Undershot`; 2 · Check overlap (turn) [Open]; 3 · Pitch test [Run] + the same adjust row; 4 · Check overlap (pitch) [Open]; *Step once: Turn / Pitch*; tips per tool (§13.2); a permanent *Nothing moved?* link at the bottom. Tools are disabled with the same reasons as Start.
- **Result line** after a Test or Step once: under the tool that ran, `setup.calibrate.result.*` ending with the *Nothing moved?* link; stays until the next run or a Game switch. If `ScreenUnchanged` fired: the amber `setup.calibrate.nothingMoved.detected` replaces it and the checklist popover opens by itself.
- Form labels left-aligned. Validation messages inline under the field, the Game is saved on every valid change (autosave, no Save button).

### 11.3 Damaged Game

Same header row (title; *Reveal in Finder* / *Delete* right). Banner `damaged.title` + the validation error list + `damaged.body`. No panes.

### 11.4 Capture zone picker

Separate window per display, covering it fully, dimmed live display behind; the user draws a purple rectangle with corner handles and a size chip; above, the display chip and `zonePicker.tip`; `zonePicker.instruction`; thin bottom card with X / Y / W / H fields, *Show frame on screen* checkbox, *Cancel* / *Save zone*. Esc cancels. Saved with display ID, size and scale (§6).

### 11.5 Countdown screen

Replaces both panes (sidebar greyed): big digit in a ring; `countdown.title`; `countdown.body` (Session) or `countdown.body.tool` (other run kinds); Display preview card on the right with *Hide preview* / *Show preview*; `countdown.stop`; *Cancel*. Same screen for every run kind.

**Error state** (pre-flight or stream failure, §4.5): the ring is replaced by `countdown.error.title` and one amber box (`countdown.error.*`) with its action button (*Open System Settings* / *Change zone…* / *Install ViGEmBus ↗* / *Try again*) and a ghost *Back*. The preview card shows the placeholder. Overlay hidden. *Decided at assembly*.

### 11.6 Session view (running)

Head card: Game name, `Row 2 of 6`, progress bar, `17 / 78 Shots`, `≈ 3 min 10 s remaining`, `session.running.hint`, *Stop*; on the right the Display preview card (zone frame drawn by the UI, *Hide preview* button, placeholder when off or failed, no "Live" label). If Shots 1 and 2 are identical: amber `session.running.identical` in the head card until the end. Below: thumbnails grid Rows × Shots per Row of real 16:9 thumbnails with `rNN_cNN` overlaid, next cell purple outline, pending cells dashed; Session folder path.

**Test / Step once running state**: minimal: tool name, Display preview card, *Stop*, `test.running.hint`. Check overlap shows the same, then opens its window with the two Samples.

### 11.7 Session end

Summary card: `session.end.completed.title` / `interrupted.title` / `failed.title`; line `date · duration · 6 × 13 · PNG`; Shots count; size; **Zip Session**; **Reveal in Finder** / **Reveal in Explorer**; *Back to Game*. Completed: `session.end.next` box. Interrupted: amber chip + `session.end.interrupted.note`. Failed: red chip + `sessions.failed.template` with its reason fragment. Full Shots grid below. No preview.

### 11.8 Check overlap window

Turn | Pitch toggle; the two Samples onion-skinned (second at 55 %); dashed band with the Overlap % chip; alignment slider; verdict box `overlap.verdict.*`; `overlap.instruction`, `overlap.tip`; *Retake Samples* (runs the countdown again) / *Done*.

### 11.9 Sessions tables

Per Game in the Sessions tab, global in the **All Sessions** window (adds a Game column): When · (Game) · State chip (completed green / interrupted amber / failed red) · Shots `n / N` · Size · *Zip* / *Reveal* / *Delete*. A failed row shows a round `?` button opening a popover with `sessions.failed.template`, never inline error text. *Delete* swaps the row's actions for `sessions.deleteConfirm` [Delete] [Cancel] sliding in from the right (180 ms ease-out). Footer `sessions.folder` linking to Settings. Sorted newest first.

### 11.10 Nothing moved? popover

One popover component (shared with the failed-row `?`): `nothingMoved.title`, `nothingMoved.intro`, five numbered items filtered by the Game's Input device and OS: 1 `fullscreen`, 2 `focus`, 3 the device line (`keyboard` / `mouse` / `gamepad`), 4 the OS line (`mac.controllerMouse` / `win.rawInput`), 5 `antiCheat`. Closes on click elsewhere or Esc.

### 11.11 Onboarding window

Shown at first launch (`onboarded` in `settings.json`, set by *Later* or *Continue*), whenever a macOS permission is missing at launch, and from Settings → *Review permissions…*. A missing ViGEmBus never brings it back: the driver is optional. `onboarding.title`.
- macOS: `onboarding.mac.subtitle`, `intro`; Accessibility and Screen Recording rows with red/green dot, status text and *Open System Settings*; `onboarding.mac.stuck`; the macOS 26 picker card; the gamepad card. *Later* / *Continue* (Continue disabled until both are green).
- Windows: `onboarding.win.subtitle`; *Running as administrator* card; ViGEmBus row with *Install ViGEmBus ↗* (official release page) and `onboarding.win.vigem.tip`; *Later* / *Continue*.

### 11.12 Settings window

Left-aligned labels. Sessions folder (path, *Change…*, *Reveal*, `settings.folder`); Shortcuts Start / Stop (*Record…*, *Reset*, `settings.shortcuts`, `settings.shortcuts.mac`); New Game defaults (Start pole, two delays, `settings.defaults`); Display preview switch (`settings.displayPreview.*`); General: *Check for updates at launch*, *Review permissions…*, *Reveal logs*.

### 11.13 About window

Centred: app icon, `Panowww {version}`, `about.tagline`, Respawwwn R mark + *By Respawwwn*, links respawwwn.com · Discord · Ko-fi · GitHub, `about.licence`, update line (`update.banner` text as a chip, or `about.upToDate`).

---

## 12. Visual design

Direction **B "Respawwwn family"**: Sindre Sorhus-style native calm wearing Respawwwn's purple. Reference stylesheet `prototype/visual-design/project/b.css`; boards `B-*.dc.html` on that branch (superseded by §13 where wording differs, and by the 2026-10-30 refinements folded into §11).

- **Type**: Inter, tabular numbers for every count. Title 22/700, window title 18/700, section 14/600, body 13/400, label 12/500 muted, tip 11.5/400 muted, overline 11/600 uppercase, counter 26/700. Buttons, chips and key caps carry a 1 px optical top nudge.
- **Light**: content `#fbfbfd`, sidebar `#ebe8f6`, card `#fff`, line `#e6e3f1`, lavender `#efecf9` (text `#4a3c96`), accent `#6454b5`, accent deep `#4a3c96`, text `#1b1740`, muted `#6b6788`, faint `#9a96b6`, ok `#2eaa5d`, warn `#d98c00`, bad `#e5365f`.
- **Dark**: content `#15122b`, sidebar `#201b44`, card `#1e1a38`, line `#2d2852`, lavender `#2a2550` (text `#c4b9f6`), accent `#a596ea`, text `#ecebf5`, muted `#a8a4c6`, ok `#4cd17a`, warn `#f0b544`, bad `#ff6b8d`. Follows the OS appearance.
- **Shapes**: cards 14 px, fields 8 px, buttons / tabs / chips are pills (Windows: 8 / 4 px, caption bar ─ ☐ ✕). Shadow `0 1px 2px` at 6 % on light, none on dark. Spacing 4 · 8 · 12 · 16 · 20 · 24; main padding 24, dialogs 20, cards 14; sidebar rows 34, controls 28.
- **Components**: pill tabs (selected purple); primary pill purple, secondary white with lavender border, ghost, danger (soft red); numbered lavender badges; lavender status box with green/red dot; chips completed / interrupted / failed; **key caps** = flat lavender caps joined by `+`, never button-like; tips = 11.5 muted line or lavender box (amber warning, green verdict); thumbnails = real images 16:9 with the name overlaid; popover; inline confirmation slide.
- **Overlay**: see §10.
- **App icon**: concept *Lens + orbit* on the Respawwwn recipe (ground `#6454B5`, white glyph, thick black rounded outline): lens with pupil and highlight, 248° orbit arc ending in an arrowhead tangent at the bottom right. Vector source `#ic-orbit` in `prototype/visual-design/project/Icons.dc.html`. The human finalises it; implementation produces the macOS squircle set and the Windows ICO.
- **Brand**: Respawwwn marks are rights-reserved; only the R mark in About (and the wordmark if wanted) under `assets/brand/` with the NOTICE; colours and OFL fonts are free to reuse. Panowww has its own icon.

---

## 13. Strings module

All user-visible strings live in `ui/src/lib/strings.ts`, keyed as below; per-OS variants are separate keys. Rules: short sentences, imperative or "you"; never an exclamation mark; glossary terms capitalised; `→` only in a one-line tip under a field; shortcuts rendered as key caps `[…]` from the current Settings, never written out; `{…}` = dynamic value. Button captions and labels not listed here are fixed by §11 and keyed at the implementer's discretion. No "hide tips" setting.

### 13.1 Game Setup · Game & input

| Key | Location | Style | Text |
|---|---|---|---|
| `setup.game.notes.placeholder` | Notes field | placeholder | Photo mode, HUD off, borderless 2560 × 1440, weather settled. |
| `setup.input.keyboard.keys` | under the three key caps | line | Three distinct keys, no modifiers. Panowww presses the key at that position on your keyboard, whatever the layout. |
| `setup.input.keyboard.hold` | under the turn / pitch holds | line | How long the key stays pressed for one Step. Calibrate with the Turn and Pitch tests. |
| `setup.input.mouse.mode` | under Move / Drag | line | Move: the cursor alone. Drag: a button held while moving, what most free cameras expect. |
| `setup.input.mouse.crossOs` | under the deltas | line | Mouse values are not promised to match between macOS and Windows: recalibrate on each. |
| `setup.input.gamepad.unavailableMac` | banner, macOS only | box amber | Virtual gamepad is unavailable on macOS. You can set this Game up here, but it only runs on Windows, where it needs ViGEmBus. |
| `setup.input.gamepad.deflection` | under Deflection, Windows | line | Needs ViGEmBus. Right stick of a virtual Xbox 360 controller, as a share of full travel. |
| `setup.input.gamepad.hold` | under the holds | line | How long the stick stays pushed for one Step. Calibrate with the Turn and Pitch tests. |

### 13.2 Game Setup · Grid & timing, Start card, Capture zone, Calibrate

| Key | Location | Style | Text |
|---|---|---|---|
| `setup.grid.startPole.nadir` | under Start pole (nadir) | line | Start looking straight down. The first Row is the ground, the last one the sky. |
| `setup.grid.startPole.zenith` | under Start pole (zenith) | line | Start looking straight up. The first Row is the sky, the last one the ground. |
| `setup.grid.shotsPerRow` | under Shots per Row | line | Each Row ends with a closing turn back to the start heading. |
| `setup.grid.rows` | under Rows | line | Off-target Overlap? Change Shots per Row or Rows, then recalibrate. |
| `setup.grid.waitAfterMove` | under Wait after move | line | Motion blur in the Shots → raise this. |
| `setup.grid.waitAfterShot` | under Wait after Shot | line | A swallowed Step → raise this. |
| `setup.start.shortcutStart` | under the Start card, line 1 | line | Start with the game in front  [Ctrl+Shift+8] |
| `setup.start.shortcutStop` | line 2 | line | Stop at any time  [Ctrl+Shift+9] |
| `setup.start.disabled.noZone` | under Start Session / Calibrate tools | line | Set the Capture zone first. |
| `setup.start.disabled.gamepadMac` | idem | line | Gamepad Games run on Windows only. |
| `setup.start.disabled.damaged` | idem | line | Fix the file to start a Session. |
| `setup.zone.set` | status, zone set | status | Display {n} · {w} × {h} at ({x}, {y}) |
| `setup.zone.notSet` | status, no zone | line | Not set on this machine. Draw it over the game once, Panowww remembers it for this Game. |
| `setup.zone.displayChanged` | status, display gone / scale changed | box amber | Display {n} is gone or its resolution changed. Draw the zone again. |
| `setup.zone.hdr` | status, Windows, HDR display | box amber | HDR is on for Display {n}. Shots come out washed out: turn HDR off in Windows or in the game. |
| `setup.calibrate.countdown` | top of the Calibrate card | line | Tests start after a 5 s countdown: switch to the game. |
| `setup.calibrate.turnTest` | under 1 · Turn test | line | Pick a landmark. After the turn the camera must point at it again. Overshot or undershot? Adjust, then run again. |
| `setup.calibrate.overlapTurn` | under 2 · Check overlap (turn) | line | Aim at the horizon first. Off-target Overlap → change Shots per Row, then back to step 1. |
| `setup.calibrate.pitchTest.nadir` | under 3 · Pitch test (nadir) | line | Point the camera straight down first. Reposition it by hand afterwards. |
| `setup.calibrate.pitchTest.zenith` | under 3 · Pitch test (zenith) | line | Point the camera straight up first. Reposition it by hand afterwards. |
| `setup.calibrate.overlapPitch` | under 4 · Check overlap (pitch) | line | Off-target Overlap → change Rows, then back to step 3. |
| `setup.calibrate.stepOnce` | under Step once | line | One Step on its own, to feel its size. |
| `setup.calibrate.result.turnTest` | under 1, after the run | line | Turn test done · {n} Steps. Back on the landmark? |
| `setup.calibrate.result.pitchTest` | under 3, after the run | line | Pitch test done · {n} Steps. On the opposite pole? |
| `setup.calibrate.result.stepOnce` | under Step once | line | Step done. |
| `setup.calibrate.result.stopped` | under the tool that was stopped | line | Test stopped. |
| `setup.calibrate.nothingMoved.link` | end of the result line, bottom of the card | link | Nothing moved? |
| `setup.calibrate.nothingMoved.detected` | replaces the result line | line amber | The screen did not change during the Test. |

### 13.3 Capture zone picker, Check overlap

| Key | Location | Style | Text |
|---|---|---|---|
| `zonePicker.instruction` | above the display | line | Draw the zone over the game. Esc cancels. |
| `zonePicker.tip` | next to the display chip | line | Game in borderless windowed mode. Keep HUD and menus out of the zone. |
| `overlap.instruction` | under the title | line | Two Samples one Step apart, superimposed. Slide until they line up. |
| `overlap.verdict.ok` | verdict box | box green | {p} % Overlap, in the 25–35 % target. |
| `overlap.verdict.low.turn` | verdict box | box amber | {p} % Overlap, too little → more Shots per Row. |
| `overlap.verdict.low.pitch` | verdict box | box amber | {p} % Overlap, too little → more Rows. |
| `overlap.verdict.high.turn` | verdict box | box amber | {p} % Overlap, more than needed → fewer Shots per Row. |
| `overlap.verdict.high.pitch` | verdict box | box amber | {p} % Overlap, more than needed → fewer Rows. |
| `overlap.tip` | bottom of the window | box | Aim at the horizon before taking Samples. Samples stay in memory, nothing is saved. |

### 13.4 Countdown, Session, Session end, Sessions tables, overlay

| Key | Location | Style | Text |
|---|---|---|---|
| `countdown.title` | under the digit | title | Switch to the game |
| `countdown.body` | under the title, Session | line | The Session starts when the countdown ends. Then leave the mouse and the keyboard alone: {n} Shots, about {duration}. |
| `countdown.body.tool` | under the title, Tests / Step once / Check overlap (*assembly*) | line | The {tool} runs when the countdown ends. Leave the mouse and the keyboard alone until it is done. |
| `countdown.stop` | bottom | line | Stop at any time with [Ctrl+Shift+9] |
| `countdown.cancel` | button | button | Cancel |
| `countdown.error.title` | error state (*assembly*) | title | Cannot start |
| `countdown.error.accessibility` | macOS, action *Open System Settings* | box amber | Accessibility permission is missing. Panowww cannot move the camera without it. |
| `countdown.error.screenRecording` | macOS, action *Open System Settings* (*assembly*) | box amber | Screen Recording permission is missing. Panowww cannot capture the zone without it. |
| `countdown.error.vigem` | Windows, gamepad Game, action *Install ViGEmBus ↗* | box amber | ViGEmBus is not installed. Gamepad Games need it. |
| `countdown.error.displayGone` | action *Change zone…* (*assembly*) | box amber | Display {n} is gone or its resolution changed. Draw the zone again. |
| `countdown.error.captureUnavailable` | action *Try again* (*assembly*) | box amber | The display could not be captured. Close other capture tools, then try again. |
| `countdown.error.back` | ghost button (*assembly*) | button | Back |
| `session.running.hint` | head card | line | The game keeps the focus. Do not touch the mouse or the keyboard. Stop with [Ctrl+Shift+9]: the Shots already taken are kept. |
| `session.running.identical` | head card | line amber | The first two Shots are identical. Nothing moved? Stop and run a Turn test. |
| `session.preview.hide` | preview card | button | Hide preview |
| `session.preview.show` | preview card, when off | button | Show preview |
| `test.running.hint` | Test running state (*assembly*) | line | Leave the mouse and the keyboard alone. Stop with [Ctrl+Shift+9]. |
| `session.end.completed.title` | summary card | title | All {n} Shots are in |
| `session.end.interrupted.title` | summary card | title | Stopped after {n} Shots |
| `session.end.failed.title` | summary card | title | Failed at Shot {n} |
| `session.end.next` | summary card, completed | box | Next: drop the folder or the zip into PTGui or Autopano Giga. |
| `session.end.interrupted.note` | summary card, interrupted | line | The Shots taken are kept. An interrupted Session cannot be resumed: start a new one from the same pole. |
| `session.end.failed.note` | summary card, failed | line | = `sessions.failed.template` |
| `sessions.folder` | footer of both tables | line | Sessions are saved in {path} · Change the folder in Settings |
| `sessions.deleteConfirm` | row, replaces the actions | inline | Delete the whole folder? |
| `sessions.failed.template` | `?` popover, failed end card | popover | Stopped at Shot {n}: {reason} The {n−1} Shots taken are kept in the folder. |
| `sessions.failed.reason.captureUnavailable` | fragment | | the display could not be captured. |
| `sessions.failed.reason.displayGone` | fragment | | Display {d} was disconnected. |
| `sessions.failed.reason.permissionRevoked` | fragment, macOS | | Screen Recording permission was revoked. |
| `sessions.failed.reason.shotFailed` | fragment | | the Shot failed twice in a row. |
| `overlay.done` | overlay pill, ≈ 2 s | pill | Done · {n} Shots |
| `overlay.stopped` | overlay pill, ≈ 2 s | pill | Stopped · {k} / {n} |

### 13.5 Empty list, damaged Game, onboarding, Settings, About, update

| Key | Location | Style | Text |
|---|---|---|---|
| `empty.title` | empty Games list | title | Add your first Game |
| `empty.body` | empty Games list | line | A Game holds the camera moves, the grid and the delays for one title. Set it up once, then shoot as many panoramas as you like. |
| `empty.import` | empty Games list | line | Someone already set up this title? Import their shared .panowww.json file. |
| `damaged.title` | damaged Game detail | title | This Game file cannot be read |
| `damaged.body` | under the validation errors | box amber | Edit the file by hand and save it, Panowww reloads it. Or delete the Game and set it up again. |
| `onboarding.title` | window | title | Welcome to Panowww |
| `onboarding.mac.subtitle` | macOS | subtitle | Two permissions before the first Session |
| `onboarding.mac.intro` | macOS | line | Panowww moves the camera for you and captures the game. macOS asks you to allow both. |
| `onboarding.mac.accessibility.ok` | Accessibility row | status | Granted · lets Panowww send keys and mouse moves to the game |
| `onboarding.mac.accessibility.missing` | Accessibility row | status | Not granted · needed to move the camera |
| `onboarding.mac.screenRecording.ok` | Screen Recording row | status | Granted · lets Panowww capture the zone |
| `onboarding.mac.screenRecording.missing` | Screen Recording row | status | Not granted · needed to capture the zone |
| `onboarding.mac.stuck` | under the two rows | line | Ticked in System Settings but nothing happens? Remove Panowww from the list and add it again, then relaunch. |
| `onboarding.mac.picker` | card (macOS 26) | box | At the first capture, macOS may ask whether Panowww can bypass the system window picker. Click Allow before you start a Session, or the alert lands in your first Shot. |
| `onboarding.mac.gamepad` | card | box | Virtual gamepad is unavailable on macOS. Games that need a controller run on Windows only; keyboard and mouse work here. |
| `onboarding.win.subtitle` | Windows | subtitle | Before the first Session |
| `onboarding.win.admin` | card | box | Panowww must run as administrator. Windows asks at every launch; this is normal. Elevated, Panowww can inject input into games that run elevated themselves, which Windows would otherwise block. |
| `onboarding.win.vigem` | ViGEmBus row, with *Install ViGEmBus ↗* | status | Virtual controller driver, only needed for gamepad Games. |
| `onboarding.win.vigem.tip` | under the row | line | Keyboard and mouse Games work without ViGEmBus. |
| `settings.folder` | under Sessions folder | line | One folder per Game, one timestamped folder per Session. |
| `settings.shortcuts` | under Start / Stop | line | Pick a combo the game does not use. The game keeps the focus when you press it. |
| `settings.shortcuts.mac` | line 2, macOS | line | On a Mac laptop, avoid the F-keys: they act as media keys unless fn is held. |
| `settings.defaults` | under New Game defaults | line | Applied to every new Game. Existing Games keep their values. |
| `settings.displayPreview.label` | Settings | label | Show the Display preview during a countdown and a Session |
| `settings.displayPreview.hint` | under the switch | line | Turn it off if the game stutters while Panowww runs. |
| `about.tagline` | About, README, empty state | line | Captures the Shots of a 360° panorama inside a video game, ready to stitch. |
| `about.licence` | About | line | Panowww is a Respawwwn project by Kevin Wenger. MIT License. The Respawwwn name and logo are not covered by the license. |
| `about.upToDate` | About | chip | You're up to date. |
| `update.banner` | bottom bar, every screen | bar | Panowww {version} is available · See what's new |

### 13.6 Nothing moved? popover

| Key | Where | Style | Text |
|---|---|---|---|
| `nothingMoved.title` | popover | title | Nothing moved? |
| `nothingMoved.intro` | popover | line | Panowww sends the keys and mouse moves but gets nothing back from the game. Check, in this order: |
| `nothingMoved.fullscreen` | 1 | item | The game runs in borderless windowed mode, not exclusive fullscreen. |
| `nothingMoved.focus` | 2 | item | The game had the focus when the countdown ended. Click into it next time. |
| `nothingMoved.keyboard` | 3, keyboard Games | item | The three keys match the game's camera bindings. Panowww presses the key at that position on your keyboard: check them on your layout. |
| `nothingMoved.mouse` | 3, mouse Games | item | Switch between Move and Drag, or try another drag button. Some games read the mouse only after a click inside their window. |
| `nothingMoved.gamepad` | 3, gamepad Games, Windows | item | ViGEmBus is installed and the game accepts an Xbox 360 controller. Unplug physical controllers: the virtual pad must be player 1. Some games only look for controllers at launch: restart the game once after the first countdown. |
| `nothingMoved.mac.controllerMouse` | 4, macOS | item | Some Mac games read the mouse through the game-controller path and ignore injected moves. Use the game's keyboard camera controls if it has them. |
| `nothingMoved.win.rawInput` | 4, Windows | item | Some games read the mouse and keyboard straight from the device and ignore injected input. Use a gamepad Game for them. |
| `nothingMoved.antiCheat` | 5 | item | Multiplayer games with anti-cheat block injected input and may ban you for it. Use Panowww offline, in single-player. |

### 13.7 Not in the module

Schema validation messages (produced by `core`'s validator, one per field, plain English, serialised through `AppError`); README Tips and Troubleshooting (reuse the wording above; the *Nothing moves during a Test or a Session* entry is §13.6 word for word).

---

## 14. Distribution, release, repository

Full process in [docs/release.md](release.md). Summary:

- **Artifacts**: `Panowww.dmg` (Apple Silicon, macOS 14+) and `Panowww-setup.exe` (NSIS, per-user, x64, Windows 10 1903+ / 11, WebView2 `downloadBootstrapper`). No MSI, no portable, no universal binary.
- **Windows manifest**: `requireAdministrator` via `tauri_build::WindowsAttributes::app_manifest`; UAC prompt at each launch, explained in README and onboarding. No WebView2 `MemoryUsageTargetLevel` tuning: the idle footprint measured ≈ 80–95 MB on 2026-11-01 (WebView2 75–90 MB, exe 5 MB), not worth the code.
- **macOS signing**: stable **self-signed** certificate (openssl, EKU codeSigning, 10 years) so Accessibility and Screen Recording grants survive updates; imported by a hand-written CI step into a temp keychain, then `APPLE_SIGNING_IDENTITY=<CN>`. Secrets `MACOS_SIGNING_P12_BASE64`, `MACOS_SIGNING_P12_PASSWORD`, `APPLE_SIGNING_IDENTITY`; 1Password item *Panowww macOS signing certificate*. Not notarized: Gatekeeper still shows "Apple could not verify" → *Open Anyway*. No Windows signing (SmartScreen *More info → Run anyway*).
- **Versioning**: SemVer, single source of truth = workspace `Cargo.toml` `version` (`tauri.conf.json` reads it). Repo starts at `0.0.1`; first CI-built release `v0.1.0`; all `0.x` pre-releases; public debut `v1.0.0-beta.1`; stable `v1.0.0`.
- **Release**: tag `vX.Y.Z` → `tauri-action@v1` (`macos-latest` arm64 + `windows-latest`) → **draft** GitHub Release with auto-generated notes → human reviews and publishes. No release-please, no Conventional Commits. Private repo until the beta: tag sparingly (macOS minutes ×10).
- **Update check**: passive, at launch, GitHub Releases API; newer → bottom bar linking the release page; switch in Settings; silent failure (private repo, offline).
- **ViGEmBus**: never redistributed; onboarding and README link the official v1.22.0 release page with the "archived but works, gamepad only" note.
- **README** (English, non-technical, centred header in HTML, the rest in Markdown), in this order, decided 2026-11-06 against reference READMEs (`docs/research/readme-references.md`):
  1. *Header*, centred: app icon, title, tagline, badges (latest release, platforms macOS · Windows, licence MIT, Discord), then two buttons *Download for macOS* / *Download for Windows* linking `releases/latest`.
  2. *Hero*: an animated two-display mockup (the game on one, Panowww at 60–80 % on the other, over a background, zooming on each step), produced by issue 27; a static screenshot until it exists.
  3. *Why Panowww*: what it does, the hand-off to PTGui or Autopano Giga, link to [How to Create a 360° Panorama in a Video Game](https://www.youtube.com/watch?v=ZR5Tn9tmDEM).
  4. *Screenshots*: Capture zone picker, Game Setup, Check overlap, overlay during a Session, Session view, Sessions list (stitching joins when it exists).
  5. *Quick start*: five lines, Game → Capture zone → Test → Session → zip → stitch.
  6. *Requirements*: macOS 14+ on Apple Silicon, Windows 10/11 x64, game in borderless windowed mode, ViGEmBus for a gamepad Game, a stitching program.
  7. *Download*: the two files of the latest release.
  8. *Installation*, one part per OS: one honest sentence about unsigned builds; macOS: Applications → open → *Open Anyway* in Privacy & Security → grant Accessibility and Screen Recording → relaunch, terminal alternative `xattr -dr com.apple.quarantine /Applications/Panowww.app`; Windows: SmartScreen *More info → Run anyway*, why the UAC prompt, ViGEmBus link.
  9. *FAQ* as symptom → fix pairs, the tips for best results folded in ("permission ticked but nothing happens", "Nothing moves during a Test or a Session" = §13.6, Gatekeeper, UAC, ViGEmBus, nadir start, borderless windowed, no HUD in the zone, photo mode, HDR off).
  10. *Motivation*: why Respawwwn built it, Nvidia Ansel's 360° capture gone with the Nvidia App, one all-in-one tool for 360° capture.
  11. *Contributing*: short, points to `CONTRIBUTING.md`.
  12. *License*: MIT, then the credits: "Made by [Respawwwn](https://respawwwn.com)" + Discord (https://discord.gg/49tS4mVNbp) + Ko-fi (https://ko-fi.com/respawwwn), "Panowww is a Respawwwn project by Kevin Wenger".

  No *Sharing Games* section and no per-feature blocks: the hero shows the whole flow.
- **Licence and brand**: `LICENSE` MIT `Copyright (c) 2026 Kevin Wenger (Respawwwn)`; `NOTICE`: "The Respawwwn name and logo are not covered by the MIT license"; `assets/brand/`; Inter with `OFL.txt`.
- **Contributing**: short `CONTRIBUTING.md` (Rust + Node + pnpm, `just dev`, `just test`, clippy passes, manual checklist link); issue templates *Bug report*, *Game doesn't work* (asks for the Game JSON, OS, Input device, game, display mode), *Feature request*. No Code of Conduct.
- **Repository hygiene**: private until `v1.0.0-beta.1`, then flipped public as-is: never commit secrets, personal paths or `.wayfinder/` (git-ignored). Going-public checklist in `docs/release.md`.

---

## 15. Pending Windows facts

*Windows verification of the spike* (branch `prototype/overlay-shortcut-spike`, `spike/README.md` protocol) ran on **2026-11-01** and is closed: Windows 11 Home, two 1920×1080 displays, GTX 1070, ViGEmBus 1.22.0; targets NoClip in a maximized browser, Overwatch (borderless, passive tests only because of its anti-cheat) and Star Wars Battlefront 2 (offline Arcade, raw-input Frostbite game). Facts not reached are recorded as such with the decision taken; the design holds everywhere.

| # | Open fact | Design as written | If it fails | Result |
|---|---|---|---|---|
| W1 | Spike's Windows code compiles | — | small fixes | ✅ Builds unchanged on Rust 1.99.1 (3 unused warnings). One fix: a missing `;` in `ui/index.html` left every list of the page empty; the spike's last commit had never been run. |
| W2 | tao `focusable: false` overlay stays non-activating when re-shown over a borderless game | overlay on Windows | in-app countdown fallback only; overlay never shown on Windows | ✅ Over NoClip on both displays and over Overwatch borderless: game kept focus and camera control, clicks reached it, countdown and frame drawn in place. Overlay stays; in-app countdown is the degraded path only. |
| W3 | `Ctrl+Shift+8/9` fire over a raw-input game; `RIDEV_INPUTSINK` listener sees them under `RIDEV_NOHOTKEYS` | `RegisterHotKey` | add the raw-input listener as a second shortcut source in `app` | ✅ `Ctrl+Shift+8` / `9` started and stopped runs through `RegisterHotKey` with NoClip, Overwatch and Battlefront 2 focused: no game silenced it. The sink was not exercised and **no listener is added**. Side effect: Ctrl / Shift trigger crouch or sprint in some games; the Settings recorder answers that. |
| W4 | Which mouse sub-move settings move cameras | 20 px / 8 ms app constants | per-OS constants; a Game field only in schema v2 | ✅ for the design: 400 px in 20 × 20 px / 8 ms turned Battlefront 2's camera, Shots all different; NoClip ignores plain moves (drag only). Other sub-move sizes were not compared; **20 px / 8 ms stays on both OSes**, no per-OS constants. |
| W5 | ViGEm virtual stick moves a camera; `BusNotFound` as the pre-flight | gamepad device, `countdown.error.vigem` | gamepad out of v1 | ✅ Pad detected as Xbox 360, sticks move at the OS level (hardwaretester.com), and with issue 23's adapter the virtual stick turned Battlefront 2's camera once the physical Xbox One pad was unplugged (the virtual pad must be XInput player 1). The bus's wait-ready call is unreliable on driver 1.17.333 (Win32 483, 259) → logged only, pad plugged at the pre-flight and kept across runs, *plugged* / *not plugged* shown (§8.3). `BusNotFound` unverified (driver present). Gamepad stays in v1. |
| W6 | WGC yellow border hidden without a package manifest; every Shot fresh (timestamp after the move) | WGC | DXGI Desktop Duplication fallback | ✅ No yellow border, own windows absent from Shots, every frame 410–620 ms after the move over NoClip and Battlefront 2. No DXGI fallback. |
| W7 | Grab and PNG timings at 1440p / 4K | 50 ms capture constant in the estimate | adjust the constant | ✅ at 1080p (both displays): NoClip grab 5–21 ms, PNG `fast` 31–48 ms, ≈ 2 MB; Battlefront 2 grab 40–85 ms, PNG `fast` 52–75 ms, 3.3–4.1 MB. 1440p / 4K not measured; expect ≈ 2× and ≈ 4×. **50 ms constant kept.** |
| W8 | WebView2 RAM idle with `MemoryUsageTargetLevel=LOW` | set it while the game has focus | — | ✅ Idle ≈ 80–95 MB (WebView2 75–90 MB, exe 5 MB) without the setting; the LOW toggle was not applied in the end. **Setting dropped**: nothing to gain at that footprint. |
| W9 | `requireAdministrator` build drives a game launched as administrator | always elevated | README caveat | ◻ Not tried by decision (no game runs elevated here). Design unchanged: always elevated, README caveat kept. |
| W10 | Ending of `nothingMoved.win.rawInput` | depends on W3 | "Use a gamepad Game for them." | ✅ No raw-input fallback exists, so the string ends with "Use a gamepad Game for them." (§13). |

Side findings for issues 23 to 25, same day:

- Multiplayer games with anti-cheat (Overwatch) were used for passive tests only, no injection; the `nothingMoved.antiCheat` warning stands.
- Battlefront 2 ignored a virtual pad plugged after launch; `nothingMoved.gamepad` now says to restart the game once.

- `tauri-build` embeds its Windows manifest in the app binary only, so the app crate's test binaries die with `STATUS_ENTRYPOINT_NOT_FOUND` and `just generate` / `check` / `test` fail on Windows. Fixed on `feat/windows-test-manifest` (the linker embeds `app/windows-app-manifest.xml` in every binary); `requireAdministrator` must go into the app binary's manifest only, or every test run prompts UAC (issue 25).
- Closing the spike's main window leaves the process alive (the overlay window keeps it up). Closing the app's main window must quit on Windows (issue 25).
- The ViGEm adapter: unplug before plugging, one wait-ready retry, a plugged / not plugged status in the gamepad Game's setup (issue 23).
- The raw-input shortcut listener, if W3 asks for it, matches the configured shortcut keys (issue 25).

---

## Appendix A: App constants

| Constant | Value | Where |
|---|---|---|
| Countdown, shortcut / button | 3 s / 5 s | `app` |
| Mouse sub-move | ≤ 20 px every 8 ms | `core` |
| Drag press / release settle | 20 ms each | `core` |
| Capture cost in the estimate | 50 ms per Shot | `core` |
| Shot retry | once, after a fresh `waitAfterShot` | `core` |
| Overlap target | 25–35 % | `core` / UI |
| Adjust buttons | ±2 % fine, ±10 % coarse, integer-rounded, clamped | UI → Game fields |
| Screen-unchanged detector | 160 px long side, greyscale, MAD < 1 % | `core` |
| Display preview | 2 fps, 640 px long side, JPEG ≈ 70, latest-wins | `capture` |
| Live thumbnail | small JPEG from the Shot buffer, progress Channel | `store` |
| PNG | `Compression::Fast`, RGB8, worker thread, bounded queue | `store` |
| Overlay | frame `#a596ea` 2 px; progress bar 2 px; end pill ≈ 2 s | `app` |
| Minimum window | ≈ 1040 × 640 | `app` |
| Delete confirmation slide | 180 ms ease-out | UI |
| Shortcut defaults | `Ctrl+Shift+8` / `Ctrl+Shift+9` | `app` |
| Session folder timestamp | `YYYY-MM-DD_HH-mm-ss` | `store` |

## Appendix B: Decided at assembly

Gaps the assembling session filled on 2026-10-30, each cheap to reverse:

1. `Op::Sample(slot)` in the capture plan so Check overlap and the detector are plan-driven (§4.3).
2. `session.json` envelope `format: "panowww-session", version: 1`, error `kind` set, and the folder timestamp format (§6).
3. The toolbar carries no second *Start Session* button; the Start card is the only one (§11.1), following the latest visual boards.
4. Countdown error state shape and its keys `countdown.error.title`, `.screenRecording`, `.displayGone`, `.captureUnavailable`, `.back` (§11.5, §13.4), closing the gap left by *Live display preview* and *Diagnosis when a game ignores injected input*.
5. `countdown.body.tool` and `test.running.hint` for non-Session run kinds; `countdown.cancel`; `setup.zone.set` as a keyed format string (§13).
6. Autosave of the Game on every valid change, file renamed when the name changes; Duplicate names the copy `<name> copy` (§5, §11.2).
7. The *Show zone* toggle draws the overlay frame without a run (§11.2), the simplest reading of the spike's request.

## Appendix C: Source tickets

| Section | Wayfinder tickets |
|---|---|
| §1, §2 | map Notes, 004, 009 |
| §3, §4 | 001, 002, 003, 008, 011, 012, 015, 016 |
| §5 | 005 |
| §6 | 005, 008, 006 |
| §7 | 006, 012 |
| §8 | 002, 012, 016 |
| §9 | 003, 012, 015 |
| §10 | 001, 012, 006 |
| §11 | 007, 010, 015, 016 |
| §12 | 004, 010 |
| §13 | 014, 015, 016 |
| §14 | 009, 004 |
| §15 | 013 |
