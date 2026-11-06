# Panowww v1 implementation plan

Ordered issues that implement [docs/spec.md](spec.md), each sized for one Claude Code session (≈ 100K tokens of context). macOS first; Windows issues come after *Windows verification of the spike* closes. Each issue becomes a GitHub issue on `Respawwwn/panowww` once the repo has a remote; until then this file is the backlog.

Conventions for every issue:
- **One branch per issue, never commit on `main`.** Start from up-to-date `main`, name the branch `feat/NN-short-slug` (`feat/01-workspace`, `feat/03-game-schema`), commit on it in small steps with clear messages, and stop there: the human reviews the branch, squashes or reorders the commits as they see fit, and merges into `main` themselves. Open no PR unless asked. If an issue needs work from an unmerged branch, say so and wait rather than branching off it.
- Read `GLOSSARY.md`, `AGENTS.md`, the spec sections listed, and the previous issue's code. Nothing else by default.
- Deliver tests with the code (`cargo nextest`, insta / proptest where the issue says so, Vitest for UI); CI green on the matrix the issue names; clippy pedantic clean.
- Keep the vocabulary of the glossary in identifiers. Keep OS code out of `core` and `store`.
- **AFK** = the agent alone. **HITL** = needs the human (a game, a permission dialog, a secret). Manual checks go into `docs/manual-test.md` as they appear.
- Done = acceptance list ticked, PR body states what was verified and how.

| # | Issue | Type | Depends on |
|---|---|---|---|
| 1 | Workspace, lints, CI | AFK | — |
| 2 | Tauri app skeleton and Svelte shell | AFK | 1 |
| 3 | `core`: Game schema, validation, migration | AFK | 1 |
| 4 | `core`: capture plan and duration estimate | AFK | 3 |
| 5 | `core`: ports, fakes, Runner | AFK | 4 |
| 6 | `core`: screen-unchanged detector and Check overlap maths | AFK | 5 |
| 7 | `store`: app data, Games, settings, local state | AFK | 3 |
| 8 | `store`: Session folders, PNG worker, thumbnails, zip | AFK | 5, 7 |
| 9 | `input` macOS: CGEvent Camera and Accessibility | HITL | 5 |
| 10 | `capture` macOS: ScreenCaptureKit Stream and Display preview | HITL | 5 |
| 11 | `app`: Runner owner, countdown, pre-flight, shortcuts, commands | AFK | 2, 6, 8, 9, 10 |
| 12 | `app`: macOS overlay | HITL | 11 |
| 13 | `app`: Capture zone picker windows | HITL | 11 |
| 14 | UI foundation: tokens, components, strings, API | AFK | 2 |
| 15 | UI: Games sidebar and Game Setup | AFK | 11, 14 |
| 16 | UI: Calibrate card, Nothing moved?, Check overlap window | AFK | 15 |
| 17 | UI: countdown, Session view, Session end | AFK | 15 |
| 18 | UI: Sessions tables, Settings, About, Onboarding, update bar | AFK | 15 |
| 19 | macOS end-to-end pass and `docs/manual-test.md` | HITL | 12, 13, 16, 17, 18 |
| 20 | App icon and brand assets | HITL | 2 |
| 21 | Signing certificate and release workflow | HITL | 19, 20 |
| 22 | README, CONTRIBUTING, templates, LICENSE, NOTICE | AFK | 19 |
| 23 | `input` Windows: SendInput and ViGEm | HITL, after Windows verification | 9 |
| 24 | `capture` Windows: Windows Graphics Capture | HITL, after Windows verification | 10 |
| 25 | `app` Windows: overlay or fallback, manifest, NSIS, WebView2 | HITL, after Windows verification | 12, 13, 21 |
| 26 | Windows end-to-end pass | HITL | 23, 24, 25 |
| 27 | README hero animation | HITL | 19, 22 |

Parallel lanes once issue 1 is in: `core` (3 → 4 → 5 → 6), `store` (7 → 8), UI foundation (2 → 14), icon (20). Issues 9 and 10 can start as soon as 5 defines the traits.

---

## 1. Workspace, lints, CI

**Spec**: §3.1, §4.1, §4.8.

Create the Cargo workspace with the five crates as empty libraries carrying their `//!` role doc, `rust-toolchain.toml` (pinned stable, edition 2024), `[workspace.lints]` as specified, `version = "0.0.1"` at the workspace root, `deny.toml`, `typos.toml`, `justfile` (`check`, `test`, `lint`, `dev`), `.gitignore` (keep `.wayfinder/` and `.idea/`), `AGENTS.md` with `CLAUDE.md` symlink (vocabulary rules, crate map, how to run checks, "no OS code in core/store", the one-branch-per-issue rule above so every later session sees it), `.github/workflows/ci.yml` (macOS + Windows matrix: rustfmt, clippy `-D warnings`, nextest, typos, cargo-deny; Linux job for `core` + `store`; rust-cache, install-action, actions pinned by SHA, read-only permissions). `core` and `store` carry `#![forbid(unsafe_code)]`; `input` and `capture` expose an `unsupported` module on non-target OSes.

Acceptance: `just check` and `just test` pass locally on macOS; the CI file is valid (`actionlint`); the workspace builds on Linux with `--workspace`.

## 2. Tauri app skeleton and Svelte shell

**Spec**: §3.1, §3.2, §4.1, §4.8, §6 (bundle identifier), §11.1 (shell only), §12 (window chrome).

`app/` Tauri 2 crate with `tauri.conf.json` (identifier `com.respawwwn.panowww`, version read from the workspace `Cargo.toml`, main window ≈ 1040 × 640 minimum, overlay title bar / vibrancy, bundle targets dmg + nsis); `ui/` Svelte 5 + TypeScript + Vite SPA with pnpm, Biome, Vitest + `mockIPC`; `ts-rs` export of a first shared type into `ui/src/lib/generated/` wired into `just dev`; `ui/src/lib/api.ts` with one `ping` command; state-based navigation skeleton (sidebar + detail + bottom bar slot, empty). Inter bundled with `OFL.txt`. `tracing` with the rotating log file. Placeholder icon.

Acceptance: `just dev` opens the window on macOS with the sidebar / detail layout and the system appearance (light / dark); `pnpm test` and `pnpm lint` pass; CI builds the app crate on both OSes.

## 3. `core`: Game schema, validation, migration

**Spec**: §5, §4.7 (error shapes), Appendix A (none).

Types `Game`, `Input` (tagged union), `StartPole`, `PhysicalKey` (validated `KeyboardEvent.code`), serde with `deny_unknown_fields`, `format` / `version` envelope, a migration entry point (`v1` only, newer refused with a typed error), the validator producing one plain-English `{ field, message }` per failing field, new-Game defaults, the per-device default blocks used when switching device, name uniqueness helper (case-insensitive), file-name sanitiser (`<name>.panowww.json`, forbidden characters → `-`, ` (2)` on collision). `ts-rs` derives on every public type.

Tests: insta snapshots of serialised sample Games (one per device); proptest that any valid Game round-trips and any out-of-bounds value yields exactly one message for its field; fixtures for refused files (newer version, unknown field, duplicate keys).

Acceptance: 100 % of the rules in the §5 tables have a test; `cargo doc` clean.

## 4. `core`: capture plan and duration estimate

**Spec**: §4.3, §7.1, §7.4, §7.6, Appendix A.

`CapturePlan` and `Op`, `plan_session(game)`, `plan_turn_test`, `plan_pitch_test`, `plan_step_once(axis)`, `plan_check_overlap(axis)`; `Step` expansion into device motions (`KeyHold`, `MouseSubMoves` with ceil(delta / 20) sub-moves, `Drag` with park + press + sub-moves + release, `StickHold`), direction and `invertVertical` from the start pole; `estimate(plan, countdown)` per §7.6; a `format_duration` helper (`≈ 4 min 20 s`).

Tests: insta snapshot of every plan for every sample Game; proptest invariants (N × R Shots in a Session; every Row closes; `Wait(waitAfterShot)` only after Shots; no `waitAfterShot` in Tests; the estimate equals the sum of the plan).

## 5. `core`: ports, fakes, Runner

**Spec**: §4.2, §4.4, §7.2 (stop semantics), §7.3.

Traits `Camera`, `Stream`, `Grabber`, `ShotSink`, `Clock`, `CancelFlag`; `Progress` and `RunOutcome`; `run(...)` executing a plan: primitives in order, `Clock::wait` cancellable, `release_all` first on stop, Shot retry once after a fresh `waitAfterShot`, outcomes `Completed` / `Interrupted` / `Failed { error }`, live thumbnail produced from the frame (small JPEG) and attached to `Progress::Shot`. Feature `test-support` with `RecordingCamera`, `FakeStream` (numbered frames, optional failure schedule), `MemorySink`, `FakeClock`.

Tests: a full Session of each sample Game through the fakes with insta snapshots of the recorded primitive sequence; cancel during a wait leaves `release_all` as the last Camera call and `Interrupted` with the right Shot count; one failing Shot is retried, two fail the run with no hole in the grid; Check overlap returns two Samples.

## 6. `core`: screen-unchanged detector and Check overlap maths

**Spec**: §4.6, §7.4 (Overlap formula), §7.5 (adjust maths).

`unchanged(a, b)` (160 px, greyscale, MAD < 1 %); Runner emits `ScreenUnchanged` from `Op::Sample(BeforeFirstStep / AfterFirstStep)` and, in a Session, from Shots 1 and 2 before they leave memory; `overlap_percent(offset, size)` and the verdict (`Ok` / `Low` / `High` against 25–35 %); `adjust(value, ±2 % | ±10 %, bounds)` integer-rounded and clamped, applied to the right field per device and axis.

Tests: synthetic frames (identical, shifted by one sub-move, noise) for the detector; proptest that adjust never leaves the schema bounds and is monotonic; verdict boundaries.

## 7. `store`: app data, Games, settings, local state

**Spec**: §6 (folders, `settings.json`, `local-games.json`), §5 (storage, import rules, damaged files).

App data folder resolution injected (so tests use a temp dir); atomic JSON writes; `settings.json` and `local-games.json` with envelope + migration; Games folder: list (valid + damaged with their error list), create, save (rename on name change), duplicate (`<name> copy`, new id), delete, share-to-path, import-from-path with the Replace / Keep both decision surfaced to the caller; file watcher that re-reads a changed Game file; zone lookup with the display-changed check given the current display list.

Tests: every operation on a temp folder, including collisions, damaged files staying listed, import of a known id both ways, migration of a `version: 0` fixture.

## 8. `store`: Session folders, PNG worker, thumbnails, zip

**Spec**: §6 (Session folder, `session.json`), §9 (PNG, thumbnails), §7.3.

`ShotSink` implementation: folder `<Game>/<YYYY-MM-DD_HH-mm-ss>/` created at the first Shot, `session.json` written `running` then rewritten at the end, PNG worker thread with a bounded queue (`png` 0.18 fast, RGB8), `rNN_cNN.png` naming; Sessions listing (per Game and global, newest first, with size on demand), mark-stale-running-as-interrupted at launch, history thumbnails on demand into `cache/thumbnails/`, zip next to the folder, reveal in Finder / Explorer (OS command), delete folder + zip.

Tests: a fake Session through the sink produces the expected tree and a `session.json` that validates against its own loader; the queue back-pressures without dropping; a `running` Session is marked `interrupted` on the next listing.

## 9. `input` macOS: CGEvent Camera and Accessibility

**Spec**: §8, §3.3, §4.1. Reference: `spike/` (branch `prototype/overlay-shortcut-spike`) for the working CGEvent code.

`KeyboardEvent.code` → `kVK_*` table (letters, digits, arrows, F1–F12, numpad, space, common punctuation) with the Windows scan-code column filled from the spec table even though unused yet; `Camera` on `objc2-core-graphics`: key down/up, `MouseMoved` / `*MouseDragged` with deltas, button down/up, cursor parking, `release_all`; prerequisite module: `check()`, `request()`, `open_settings()` for Accessibility. `unsupported` on other OSes.

HITL: the human runs the Turn test of the sample mouse Game against the reference game through a tiny example binary; result goes in `docs/manual-test.md`.

Acceptance: compiles and lints on CI macOS; table covered by a test that every supported code maps both ways; manual check recorded.

## 10. `capture` macOS: ScreenCaptureKit Stream and Display preview

**Spec**: §9, §4.2 (`Grabber` / `Stream`), §4.5, §3.3. Reference: `spike/` for the `objc2-screen-capture-kit` filter and screenshot code.

Display enumeration (stable ID, size in points, scale); `Grabber::start(display, zone, preview_sink)` building the content filter that excludes Panowww's windows; `Stream::shot(not_before)` via `SCScreenshotManager` cropped to the zone, RGB8; preview `SCStream` at 2 fps with output size reduced in the SCK configuration, JPEG ≈ 70 on the dispatch queue, latest-wins, `set_preview(bool)` tearing the SCStream down / up; Screen Recording `check()` / `request()` / `open_settings()`; typed `CaptureError` kinds matching §6's error `kind` set.

HITL: the human verifies no overlay in a Shot and the macOS 26 picker alert behaviour; timings logged at 1440p.

Acceptance: compiles and lints on CI macOS; an example binary captures a zone and writes a PNG; preview frames arrive at ≈ 2 fps and stop within one frame of `set_preview(false)`.

## 11. `app`: Runner owner, countdown, pre-flight, shortcuts, commands

**Spec**: §4.4, §4.5, §4.7, §7.2, §10 (shortcuts), §6 (settings used), §11.1 (navigation state).

The single `Runner` state machine (Idle / Countdown / Running / Stopping) on a dedicated thread; countdown 3 s / 5 s; pre-flight (Accessibility) then stream opening, error state payload on failure; `Progress` forwarded on a Tauri Channel together with preview frames; global shortcuts through `tauri-plugin-global-shortcut` (register at launch, re-register on change, registration error surfaced), start = Session of the selected Game, stop = cancel; `AppError`; commands for everything the UI needs (Games CRUD, import/share via dialog plugin, settings, local zone, Sessions listing / zip / reveal / delete, run start / stop, onboarding checks, update check against the Releases API with the Settings switch, reveal logs); quit handler that stops a run and marks it `interrupted`; `ts-rs` export of every command payload.

Acceptance: `tauri::test` covers the Runner transitions (start while running ignored, stop during countdown creates nothing); a scripted run with the fakes streams Progress to a test Channel; shortcuts fire with another app focused (manual, noted).

## 12. `app`: macOS overlay

**Spec**: §10, §4.2 (Overlay port), §12 (colours). Reference: `spike/` NSPanel setup.

`Overlay` trait in `app` and its `tauri-nspanel` implementation: one panel per run on the zone's display, non-activating, click-through, all Spaces, excluded from capture; every call marshalled to the main thread; drawing inside the frame: frame, countdown digit, progress bar, counter pill, end pill for ≈ 2 s; *Show zone* / *Hide zone* mode (frame only, no run).

HITL: the human confirms over the reference game (full-screen Space included) that the game keeps focus and clicks pass through.

## 13. `app`: Capture zone picker windows

**Spec**: §11.4, §6 (zone record), §9 (display list).

One full-size window per display showing a dimmed live screenshot, rectangle drawing with handles and size chip, numeric X / Y / W / H, display chip, *Show frame on screen* (uses the overlay), Cancel / Esc / Save; saves the zone record keyed by Game id; display-changed detection on load surfaced to the Setup status line.

HITL: the human draws a zone on each of two displays with different scales.

## 14. UI foundation: tokens, components, strings, API

**Spec**: §12, §13 (all tables), §11.1, §3.2. Reference: `prototype/visual-design/project/b.css` and the `B-*.dc.html` boards (branch `prototype/visual-design`).

CSS tokens for light / dark (and the Windows radius variant behind a class), typography scale, the component set (pill tabs, primary / secondary / ghost / danger buttons, chips, key caps rendering a shortcut from Settings, numbered badges, status box, tip line / tip box in lavender / amber / green, popover, table with inline confirmation slide, segmented control, switch, collapsible card, thumbnail cell, preview card with placeholder); `strings.ts` with every key of §13 and an interpolation helper; `api.ts` over the generated types for every command of issue 11 (mockable); the app shell (sidebar, detail toolbar, bottom update bar) wired to navigation state.

Acceptance: a Storybook-free component gallery route under `?gallery` showing each component in both themes; Vitest covers the strings helper and the key-cap renderer; Biome clean.

## 15. UI: Games sidebar and Game Setup

**Spec**: §11.1, §11.2, §11.3, §5 (validation, defaults, import), §13.1, §13.2, §13.5 (empty, damaged).

Sidebar list with device icons, damaged ⚠, selection, *+ New*, *Import…* (dialog, Replace / Keep both prompt); empty state; detail header row (title, Duplicate / Share / Delete with confirmation); Setup left pane: Game & input card with the per-device fields and the key recorder, Grid & timing card, Start card with the live estimate and the shortcut lines or the disabled reason; right pane: Capture zone status (set / not set / display changed / HDR), *Set zone…* / *Change zone…*, *Show zone* / *Hide zone*; autosave on valid change with inline validation messages; damaged Game screen; sidebar greyed during a run.

Acceptance: Vitest with `mockIPC` for the Setup form (device switch replaces the block with defaults; validation messages; estimate updates); visual check against the `B-GameInput` board in both themes.

## 16. UI: Calibrate card, Nothing moved?, Check overlap window

**Spec**: §11.2 (Calibrate, result line), §11.8, §11.10, §7.5, §13.2 (calibrate rows), §13.3, §13.6.

Calibrate card with the four numbered tools, adjust rows wired to `adjust`, Step once, disabled reasons, result lines after each run (persisting until the next run or Game switch), `ScreenUnchanged` handling (amber line + auto-open popover), permanent *Nothing moved?* link; the shared popover component filtered by device and OS; Check overlap window (Turn | Pitch, onion-skin at 55 %, dashed band, slider → Overlap % and verdict, Retake Samples, Done).

Acceptance: Vitest for the popover filtering and the slider → verdict mapping; visual check against `B-CheckOverlap`.

## 17. UI: countdown, Session view, Session end

**Spec**: §11.5, §11.6, §11.7, §4.4 (Progress), §9 (preview frames), §13.4.

Countdown screen (digit ring, titles per run kind, preview card with Hide / Show persisted to `displayPreview`, Cancel) and its error state (reason box + action + Back); running Session view (head card, remaining time from the per-Shot average, amber identical line, preview card drawing the zone frame, thumbnails grid filling live, Stop); Test / Step once running state; Session end (summary card per state, Zip, Reveal, Back to Game, grid of real thumbnails loaded from the cache).

Acceptance: a scripted Progress stream through `mockIPC` drives every state; visual check against `B-Countdown`, `B-SessionRunning`, `B-SessionEnd`.

## 18. UI: Sessions tables, Settings, About, Onboarding, update bar

**Spec**: §11.9, §11.11, §11.12, §11.13, §11.1 (update bar), §13.4 (sessions rows), §13.5.

Sessions tab and All Sessions window (table, chips, `?` popover on failed rows, inline delete confirmation, footer link); Settings window (folder, shortcut recorder with Reset and registration error, New Game defaults, Display preview switch, update switch, Review permissions, Reveal logs); About window; Onboarding window for macOS (rows with status, Open System Settings, picker card, gamepad card, Later / Continue gating) and the Windows variant behind the OS flag; bottom update bar.

Acceptance: Vitest for the shortcut recorder and the delete confirmation; visual check against `B-SessionsTab`, `B-AllSessions`, `B-Settings`, `B-About`, `B-Onboarding`.

## 19. macOS end-to-end pass and `docs/manual-test.md`

**Spec**: §7 whole, §11 whole, §4.8 (manual checklist).

HITL. The human runs the full flow on the reference game: onboarding, New Game, zone, Turn / Pitch tests with adjust, Check overlap, Step once, a complete Session, an interrupted one, zip, reveal, delete, Share → Import on the same machine, Settings changes (folder, shortcuts), update bar against a fake newer release. The session writes `docs/manual-test.md` (OS × Input device × game matrix, one row per check, the macOS column filled) and fixes what the pass finds. Name the reference game in the file.

Acceptance: every row of the macOS column passes or has an issue linked.

## 20. App icon and brand assets

**Spec**: §12 (app icon, brand), §14 (licence and brand).

HITL. From `#ic-orbit` in `prototype/visual-design/project/Icons.dc.html`: the human finalises the vector; the session produces the macOS icon set (squircle mask) and `icon.ico`, wires them into `tauri.conf.json`, adds `assets/brand/` with the Respawwwn R mark for About and the `NOTICE`, and the sidebar mark + wordmark.

## 21. Signing certificate and release workflow

**Spec**: §14, [docs/release.md](release.md).

HITL. A guided wizard script generates the self-signed certificate (openssl, EKU codeSigning, 10 years), exports the `.p12`, and tells the human what to store in 1Password and in the three repository secrets; `.github/workflows/release.yml` with `tauri-action@v1` (macOS arm64 + Windows x64), the hand-written keychain import step, draft Release; `docs/release.md` updated where the implementation differs. First tag `v0.1.0` once issue 19 is done.

Acceptance: a tag on a test branch produces a draft Release with both files; the DMG's app has a stable designated requirement across two builds (`codesign -d -r-`).

## 22. README, CONTRIBUTING, templates, LICENSE, NOTICE

**Spec**: §14 (README outline, contributing, licence), §13.6 (Troubleshooting entry), §13.7.

README in the §14 order with real screenshots from issue 19 (taken with `just fixtures`); the hero is a static screenshot until issue 27 lands. `CONTRIBUTING.md`, the three issue templates, `LICENSE`; `NOTICE` exists (issue 20). The going-public checklist stays in `docs/release.md`. Tone: English, "we", short, no slang in instructions.

Acceptance: every §14 section present in order; badges and Download buttons resolve once the repo is public (`releases/latest`); screenshots committed under `docs/readme/`, each ≤ 500 KB.

## 27. README hero animation

**Spec**: §14 (README, *Hero*).

HITL. The AltTab-style hero: two displays as a mockup over a background, the game on one, Panowww at 60–80 % on the other, zooming on each step (Game, Capture zone, Test, Session with the overlay, Sessions list). Built with Remotion in its own folder or repository (decided when the issue starts): screen recordings by the human embedded in mockups, and the real UI (`ui/`, the gallery screen with the mocked API) rendered for the app parts. Animated SVG preferred, built like AltTab's `scripts/build_readme_svg.py` (98 lines of standard-library Python): a handful of stills (Remotion `renderStill`, or plain captures) embedded as base64 JPEG / WebP, animated with SMIL `<animate>` / `<animateTransform>` (cross-fades, zooms, pans; the game camera turn as a pan across a real panorama strip), no script, no external file, no `<video>`; a GIF only if the motion needs real video frames. Output: a looping animated SVG or GIF sized from `docs/research/readme-references.md` (target ≤ 5 MB, ≈ 900 px wide, ≤ 15 s), committed under `docs/readme/` and swapped into the README hero.

## 23. `input` Windows: SendInput and ViGEm

**Spec**: §8, §3.3, §15 (W4, W5). Needs `feat/windows-test-manifest` merged first (`just check` on Windows).

`Camera` on `windows` 0.62: `SendInput` with `KEYEVENTF_SCANCODE`, `MOUSEEVENTF_MOVE`, buttons, `SetCursorPos` parking, `release_all`; the 20 px / 8 ms sub-move constants shared with macOS (W4). ViGEm adapter behind our own trait (`vigem-client`): unplug before plugging, one wait-ready retry on Win32 483, a plugged / not plugged status for the gamepad Game's setup, `BusNotFound` check (its missing-driver path goes into the manual test, W5).

## 24. `capture` Windows: Windows Graphics Capture

**Spec**: §9, §15 (W6, W7).

One WGC session per run, frames cropped to the zone, fresh-frame check by timestamp, preview = CPU downscale of one frame in N on the frame-arrived thread, `WDA_EXCLUDEFROMCAPTURE` on our windows, HDR detection per display. No DXGI fallback (W6 passed). Reference: `spike/src/capture/windows.rs`.

## 25. `app` Windows: overlay or fallback, manifest, NSIS, WebView2

**Spec**: §10, §14, §15 (W2, W3, W8, W9), §11 (Windows strings and chrome).

tao overlay flags (W2 passed; the in-app countdown stays the degraded path); no raw-input shortcut source (W3 passed); `requireAdministrator` in the app binary's manifest only, the test binaries keep the plain one from `feat/windows-test-manifest`; closing the main window quits the process (the spike's overlay window kept it alive); NSIS per-user config with the WebView2 bootstrapper; no `MemoryUsageTargetLevel` tuning (W8); Explorer / UAC / ms-settings strings; Windows radii and caption bar; `nothingMoved.win.rawInput` ends with "Use a gamepad Game for them." (W10).

## 26. Windows end-to-end pass

Same protocol as issue 19 on Windows, including a gamepad Game; fills the Windows column of `docs/manual-test.md`; installs from the setup.exe produced by issue 21.
