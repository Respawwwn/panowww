# Input injection in Rust on macOS and Windows

Research for ticket [002](../.wayfinder/tickets/002-input-injection.md). Checked 2026-10-29.
Question: how does Panowww reliably drive a game camera with an injected **keyboard**, **mouse** (including click-and-drag) or **gamepad** (Windows only), what permissions it needs, and how this fails with raw-input and anti-cheat games.

Confidence tags: **[doc]** = stated by the primary source linked; **[src]** = read in source code; **[inferred]** = my reasoning from the sources, not stated by them; **[unverified]** = no primary source found, must be tested.

---

## 1. Facts

### 1.1 Crates (versions from the crates.io API, 2026-10-29)

| Crate | Latest | Last publish | Notes |
|---|---|---|---|
| [`enigo`](https://crates.io/crates/enigo) | 0.6.1 | 2025-08-28 | Repo active (last push 2026-10-16). Big unreleased changelog on `main`. |
| [`rdev`](https://crates.io/crates/rdev) | 0.5.3 | 2023-06-26 | Stale. Not considered further. |
| [`windows`](https://crates.io/crates/windows) | 0.62.2 | 2025-10-06 | Microsoft's official bindings (`SendInput`, tokens, etc.). |
| [`objc2-core-graphics`](https://crates.io/crates/objc2-core-graphics) | 0.3.2 | 2025-10-04 | CGEvent bindings; what enigo `main` now uses. |
| [`core-graphics`](https://crates.io/crates/core-graphics) | 0.25.0 | 2025-05-27 | Servo bindings; what enigo 0.6.1 uses. |
| [`objc2-application-services`](https://crates.io/crates/objc2-application-services) | 0.3.2 | 2025-10-04 | `AXIsProcessTrustedWithOptions`. |
| [`macos-accessibility-client`](https://crates.io/crates/macos-accessibility-client) | 0.0.2 | 2026-01-18 | Tiny wrapper around the AX trust check. |
| [`vigem-client`](https://crates.io/crates/vigem-client) | 0.1.4 | 2022-08-01 | MIT, pure Rust, repo last push 2023-08-11, 4 open issues. |
| [`vigem-rust`](https://crates.io/crates/vigem-rust) | 0.2.0 | 2026-08-20 | MIT/Apache-2.0, created 2025-11, X360 + DS4, 8 stars, 4 commits. |
| [`vigem`](https://crates.io/crates/vigem) / `vigem-sys` | 0.9.1 / 1.1.0 | 2020-05-23 | Bindings to the C client. Dead. |
| [`vjoy`](https://crates.io/crates/vjoy) | 0.7.1 | 2025-12-19 | vJoy driver (DirectInput joystick, not an Xbox pad). |

### 1.2 Keyboard

**Windows**
- `SendInput` inserts the events serially into the input stream, and other input is not interleaved with them. It "does not reset the keyboard's current state" **[doc]** ([SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)).
- `KEYEVENTF_SCANCODE`: "wScan identifies the key and wVk is ignored". Microsoft recommends it for "simulating a physical keystroke regardless of which keyboard is currently being used… the scan code will always be the same", while the VK "can change depending on the current keyboard layout". Extended keys (arrows, etc.) need `KEYEVENTF_EXTENDEDKEY` (0xE0 prefix) **[doc]** ([KEYBDINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput)).
- enigo's Windows `key()` sends both the VK and the scan code from `MapVirtualKeyW(MAPVK_VK_TO_VSC_EX)`, without `KEYEVENTF_SCANCODE`. Its `raw(scan)` sends `KEYEVENTF_SCANCODE` (+ `EXTENDEDKEY` when needed) **[src]** ([win_impl.rs](https://github.com/enigo-rs/enigo/blob/main/src/win/win_impl.rs)).
- The old Python Windows keyboard used the `keyboard` PyPI package: press, then `time.sleep(duration)`, then release (`control/keyboard/windows.py`) **[src]**.
- Hold timing: since Rust 1.75, `std::thread::sleep` on Windows uses a high-resolution waitable timer, so it is no longer stuck at the 15.6 ms tick ([rust-lang/rust#116461](https://github.com/rust-lang/rust/pull/116461), [std source](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/thread/windows.rs)) **[src]**.

**macOS**
- The old Python code posts `CGEventCreateKeyboardEvent(None, keycode, true)` to `kCGHIDEventTap`, sleeps, then posts the key-up (`control/keyboard/macos.py`) **[src]**. That is the correct pattern. One key-down, then one key-up, is a hold: games poll key state. Synthetic events never auto-repeat **[inferred]**.
- macOS virtual keycodes (`kVK_ANSI_W`, …) name a physical key position, like Windows scan codes **[inferred from the kVK_ANSI_ naming; consistent with enigo's layout lookup]**.

### 1.3 Mouse: relative moves

**Windows**
- `MOUSEEVENTF_MOVE` without `MOUSEEVENTF_ABSOLUTE` gives relative `dx/dy` in pixels. "Relative mouse motion is subject to the effects of the mouse speed and the two-mouse threshold values" (pointer speed / acceleration). With `ABSOLUTE`, coordinates are normalized 0–65535 on the primary monitor (or the virtual desktop with `MOUSEEVENTF_VIRTUALDESK`) **[doc]** ([MOUSEINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-mouseinput)).
- Raw input (`WM_INPUT`, `RAWMOUSE`) carries a `usFlags` of `MOUSE_MOVE_RELATIVE` or `MOUSE_MOVE_ABSOLUTE`. "Raw Input mouse events is not subject to the effects of the mouse speed" **[doc]** ([RAWMOUSE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse)). Game engines branch on that flag: SDL's `WIN_HandleRawMouseInput` checks `MOUSE_MOVE_ABSOLUTE` and treats the event differently **[src]** ([SDL_windowsevents.c](https://github.com/libsdl-org/SDL/blob/main/src/video/windows/SDL_windowsevents.c)).
- **enigo trap**: with the default `Settings` (`windows_subject_to_mouse_speed_and_acceleration_level: false`), enigo turns `Coordinate::Rel` into an **absolute** move (`MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE` to current + delta), so that pointer acceleration does not apply **[src]** ([win_impl.rs](https://github.com/enigo-rs/enigo/blob/main/src/win/win_impl.rs), [lib.rs Settings](https://github.com/enigo-rs/enigo/blob/main/src/lib.rs)). In 0.6.1 it does the same. A game reading relative raw input then gets an absolute event instead of a delta **[inferred]**. Only `true` sends a real `MOUSEEVENTF_MOVE` delta.
- The old Python Windows mouse already used real relative `SendInput`/`mouse_event(MOUSEEVENTF_MOVE)` (`control/mouse/windows.py`) **[src]**.

**macOS**
- `kCGMouseEventDeltaX/Y`: "the horizontal mouse delta since the last mouse movement event" **[doc]** ([CGEventField.mouseEventDeltaX](https://developer.apple.com/documentation/coregraphics/cgeventfield/mouseeventdeltax)).
- When a game captures the mouse with `CGAssociateMouseAndMouseCursorPosition(false)`, "the events your application receives from the system have a constant absolute location but contain delta updates" **[doc]** ([CGAssociateMouseAndMouseCursorPosition](https://developer.apple.com/documentation/coregraphics/cgassociatemouseandmousecursorposition(_:))). So a mouse-look game reads **only the deltas**.
- SDL (used by many ports) reads `[NSEvent deltaX/deltaY]` in relative mode, and calls `CGAssociateMouseAndMouseCursorPosition(NO)` **[src]** ([SDL_cocoamouse.m](https://github.com/libsdl-org/SDL/blob/main/src/video/cocoa/SDL_cocoamouse.m)).
- **Old Python bug confirmed**: `move_relative` reads the current location, adds dx/dy, and posts `kCGEventMouseMoved` at that absolute point with no delta fields (`control/mouse/macos.py` l.61–99) **[src]**. A synthesized event created with `CGEventCreateMouseEvent` has its delta fields at 0 unless set, so captured-cursor games see no motion **[inferred, consistent with Apple's doc above]**. And under a dissociated cursor, the "current location" never changes, so the absolute target never accumulates either.
- enigo (0.6.1 and `main`) sets `MouseEventDeltaX/Y` on every move. For `Rel` it posts `current + (dx,dy)` with delta = `(dx,dy)`, through `kCGHIDEventTap` **[src]** ([0.6.1 macos_impl.rs](https://github.com/enigo-rs/enigo/blob/v0.6.1/src/macos/macos_impl.rs), [main](https://github.com/enigo-rs/enigo/blob/main/src/macos/macos_impl.rs)). (0.6.1 has a sign bug in the delta of `Abs` moves, `current - x`. `main` fixes it. Irrelevant for `Rel`.)

### 1.4 Mouse: click-and-drag rotation

- **Windows**: there is no drag event type. Send `MOUSEEVENTF_LEFTDOWN` (or RIGHT/MIDDLE), then relative `MOUSEEVENTF_MOVE`s, then `…UP`. "If the left mouse button is pressed and held down, MOUSEEVENTF_LEFTDOWN is set when the left button is first pressed, but not for subsequent motions" **[doc]** ([MOUSEINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-mouseinput)).
- **macOS**: motion while a button is down must be `kCGEventLeftMouseDragged` / `RightMouseDragged` / `OtherMouseDragged` (Apple: "a mouse drag event with the left button down") **[doc]** ([CGEventType.leftMouseDragged](https://developer.apple.com/documentation/coregraphics/cgeventtype/leftmousedragged)). The old Python posts `kCGEventMouseMoved` during a drag (`pano_capture.py` drag path → `move_relative`) **[src]**. AppKit routes `MouseMoved` to `mouseMoved:`, not `mouseDragged:`, so a game that tracks drags never sees motion **[inferred]**.
- enigo 0.6.1 picks `Left/RightMouseDragged` from `NSEvent::pressedMouseButtons()` (left or right only) **[src]**. enigo `main` tracks its own pressed-button state and timestamps, supports all buttons ("macos: allow dragging all mouse buttons"), and works around the OS reporting a button as still pressed for ~60 ms after release. Its changelog also says "macos: `move_mouse` no longer needs a sleep", so 0.6.1 had timing races here **[src]** ([CHANGES.md](https://github.com/enigo-rs/enigo/blob/main/CHANGES.md)).
- For drag-to-rotate games that keep the cursor visible (no capture), the game may read **positions**, not deltas. The absolute point then matters: the OS clamps it at screen edges **[inferred]**. Start the drag from the centre of the Capture zone and keep each Step's travel on screen, or split it into several drags.

### 1.5 Gamepad (Windows): ViGEmBus

- **ViGEmBus is retired.** The repo was archived on 2023-11-02. Final release v1.22.0 ("It's dead, Jim") removed the updater; its notes say an in-place upgrade doesn't work and to "simply run the setup twice". Licence BSD-3-Clause. Supports Windows 10/11 x64/x86/ARM64 since 1.21 **[doc]** ([repo](https://github.com/nefarius/ViGEmBus), [releases](https://github.com/nefarius/ViGEmBus/releases)).
- End-of-life statement: a trademark dispute with ViGEM GmbH (2023) led to retiring the name and domain. The project "will no longer receive any updates". "This has no impact on existing installations except for the updater components." Old auto-updaters point at a domain no longer controlled by the author (fix with *Legacinator*) **[doc]** ([EOL statement](https://docs.nefarius.at/projects/ViGEm/End-of-Life/)).
- Successor: **Nefarius VirtualPad**, described as "a commercial framework for gaming peripherals emulation", available only to business partners. Not an option for an MIT app **[doc]** ([VirtualPad](https://docs.nefarius.at/projects/VirtualPad/)).
- The driver is commonly installed by controller tools (e.g. DS4Windows depends on it), and the old Python app used it through `vgamepad`. I found no primary source about breakage on Windows 11 24H2/25H2 **[unverified]**.
- **`vigem-client`** (pure Rust, does not need `ViGEmClient.dll`) **[doc]** ([README](https://github.com/CasualX/vigem-client)). Usage: `Client::connect()` → `Xbox360Wired::new(client, TargetId::XBOX360_WIRED)` → `plugin()` → `wait_ready()` → `update(&XGamepad)`. Errors include `BusNotFound` ("ViGEmBus driver is not installed"), `BusAccessFailed`, `BusVersionMismatch`, `NoFreeSlot` and `TargetNotReady` **[doc]** ([docs.rs Error](https://docs.rs/vigem-client/latest/vigem_client/enum.Error.html)). Unmaintained since 2022, but it speaks a frozen driver ABI, so staleness costs little **[inferred]**.
- **`vigem-rust`** 0.2.0: newer and also maintained, X360 + DS4, rumble notifications, and `BusError::BusNotFound` / `VersionMismatch` for detection. Very small user base **[doc]** ([repo](https://github.com/arounre/vigem-rust)).
- **Driver detection**: the cheapest reliable check is `Client::connect()` returning `BusNotFound` **[doc]** (above). Installing needs admin rights (it is a kernel driver), so Panowww should link to, or bundle and launch, the official v1.22.0 installer. BSD-3 allows redistribution with the notice **[inferred]**.
- **Alternatives**
  - **vJoy** (`vjoy` crate): a DirectInput joystick, not XInput. Games that only accept Xbox pads ignore it **[inferred]**.
  - **`Windows.UI.Input.Preview.Injection.InputInjector.InjectGamepadInput`**: Windows' own API (10.0.16299+). It needs the restricted capability `inputInjectionBrokered` in a packaged (`appxmanifest`) app **[doc]** ([docs](https://learn.microsoft.com/en-us/uwp/api/windows.ui.input.preview.injection.inputinjector.injectgamepadinput)). Whether XInput or desktop games see it is undocumented **[unverified]**. Not viable for an unsigned, unpackaged v1.
  - `doppelpad` (UMDF2/VHF, 0.0.1 placeholder from 2026-08) and `logi-vhid` (Logitech G HUB bus) exist but are not usable today **[doc: crates.io]**.

### 1.6 Permissions

**macOS**
- Posting events needs the Accessibility permission (TCC). APIs:
  - `AXIsProcessTrustedWithOptions({kAXTrustedCheckOptionPrompt: true/false})` "Returns whether the current process is a trusted accessibility client". With the prompt option it opens the system dialog **[doc]** ([AXIsProcessTrustedWithOptions](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)).
  - `CGPreflightPostEventAccess()` / `CGRequestPostEventAccess()` (macOS 10.15+) check or request the right to post events specifically **[doc]** ([preflight](https://developer.apple.com/documentation/coregraphics/cgpreflightposteventaccess()), [request](https://developer.apple.com/documentation/coregraphics/cgrequestposteventaccess())).
  - enigo calls `AXIsProcessTrustedWithOptions` in `Enigo::new` (with `open_prompt_to_get_permissions` default `true`) **[src]**.
  - "Open settings" deep link: `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility` (widely used, not formally documented by Apple) **[unverified]**.
- **Unsigned / ad-hoc builds and TCC**: "Unsigned code has no DR. Ad hoc signed code… has a DR but it's tied to that specific version of the code. In both cases macOS can't reliably track the identity of the code" **[doc]** ([TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements)). So each new Panowww release (new cdhash) is likely to **lose the Accessibility grant**. The old row may stay ticked in System Settings while the check returns false, or `AXIsProcessTrusted` returns true while events are dropped (reported by third parties, e.g. [MiddleClick #162](https://github.com/artginzburg/MiddleClick/issues/162)) **[inferred from TN3127 + reports]**. Fix: remove the row (`−`) and re-add, or `tccutil reset Accessibility <bundle-id>`.

**Windows**
- No permission is needed for `SendInput` at the same integrity level.
- **UIPI**: "Applications are permitted to inject input only into applications that are at an equal or lesser integrity level", and "neither GetLastError nor the return value will indicate the failure was caused by UIPI blocking" **[doc]** ([SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)). A game running as admin silently ignores a non-elevated Panowww.
- Bypassing UIPI legitimately (`uiAccess="true"` in the manifest) requires an Authenticode signature **and** installation in a secure location (Program Files / System32). The secure-location list is not configurable **[doc]** ([UIAccess policy](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-R2-and-2008/dd834830(v=ws.11)), [Q&A](https://learn.microsoft.com/en-us/answers/questions/568475/is-the-secure-location-to-bypass-user-interface-pr)). Out of reach for unsigned v1, so the only fix is "run Panowww as administrator".
- Detection **[inferred]**: get the foreground window's PID (`GetWindowThreadProcessId`), `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`, then compare `GetTokenInformation(TokenIntegrityLevel)` with our own. If the game is higher, warn.
- ViGEm gamepad input arrives as a (virtual) device, not through `SendInput`, so UIPI should not apply to it **[inferred]**.

### 1.7 Raw input / anti-cheat failure modes

- **Injection is visible.** Low-level hooks see `LLMHF_INJECTED` (and `LLMHF_LOWER_IL_INJECTED`) on every `SendInput` mouse event **[doc]** ([MSLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct)). The keyboard equivalent is `LLKHF_INJECTED`. Any game or anti-cheat can filter on it. Kernel anti-cheats (EAC, BattlEye, Vanguard…) can also see a ViGEm device **[inferred]**. I found no first-party anti-cheat documentation on what they block **[unverified]**.
- **Absolute vs relative raw input** (Windows): see 1.3. An absolute injected move can be ignored by mouse-look code **[inferred]**.
- **Wrong key identity**: VK-only injection without a scan code. Games reading raw input or DirectInput key off the make code **[inferred]**. Use scan codes.
- **macOS: GCMouse.** Since macOS 11, GameController's `GCMouse` exposes raw mouse deltas. SDL's `main` (upcoming 3.6, the code landed 2025-12 → 2026-09) reads relative motion from `GCMouse` and **skips NSEvent deltas** when a GCMouse is present **[src]** ([SDL_cocoamouse.m](https://github.com/libsdl-org/SDL/blob/main/src/video/cocoa/SDL_cocoamouse.m)). SDL's new hint `SDL_HINT_MAC_USE_GCMOUSE` says GCMouse "may cause problems in some scenarios (remote control software that wants to send non-GCMouse input events, etc)" **[doc]** ([SDL_hints.h](https://github.com/libsdl-org/SDL/blob/main/include/SDL3/SDL_hints.h)). SDL issue [#16241](https://github.com/libsdl-org/SDL/issues/16241) (open, 2026-09) reports that synthetic `CGEvent` clicks are dropped while a GCMouse is connected, and maintainers plan "relative-only" GCMouse for 3.6.0. So **games reading GCMouse (SDL ≥ 3.6, native Metal ports) will probably ignore CGEvent mouse moves** **[inferred from the above; untested]**. SDL release-3.4.18 (2026-10-02) has no GCMouse code **[src]**. The same likely applies to `GCKeyboard` **[unverified]**.
- **Foreground only**: GCMouse/GCKeyboard deliver only to the foreground app (Apple forum reply, [thread 696020](https://developer.apple.com/forums/thread/696020)). Panowww already requires the game to keep focus.
- **Sandboxed apps** can't post events. Panowww is not sandboxed (no App Store), so this doesn't apply **[inferred]**.

---

## 2. Recommendation per device

**General shape**
- Write a thin, platform-specific **injection module** directly on `windows` 0.62 and `objc2-core-graphics` 0.3 (+ `objc2-application-services` for AX). Its interface: `key_down(Key)`, `key_up(Key)`, `mouse_move_rel(dx, dy)`, `button_down(Button)`, `button_up(Button)`.
- Use enigo as a reference, not a dependency, for three reasons:
  - its Windows relative-move default is wrong for games;
  - the released macOS drag logic is the racy 0.6.1 version;
  - the surface we need is ~150 lines per OS and must control exact flags.
- If time is short, enigo 0.6.x with `windows_subject_to_mouse_speed_and_acceleration_level: true` and only press/move/release is acceptable. Re-check drag on macOS.

**Keyboard**
- Store keys in the Game file as **physical positions**: USB HID usage or scan code, mapped to the Windows scan code and the macOS `kVK_*` code. A Game made on QWERTY then presses the same physical key on AZERTY. That matches how games bind WASD, and the human uses a French keyboard.
- Windows: `SendInput` with `KEYEVENTF_SCANCODE` (+ `EXTENDEDKEY` for arrows, etc.).
- macOS: `CGEventCreateKeyboardEvent(source, kVK, down)` posted to `kCGHIDEventTap`.
- Hold = one down, `std::thread::sleep(duration)`, one up. Always release in a `Drop`/stop path so a stopped Session never leaves a key stuck.

**Mouse (relative move)**
- Windows: `SendInput` `MOUSEEVENTF_MOVE` (never `ABSOLUTE`).
  - Raw-input games get the exact delta, with no pointer acceleration.
  - Games using `WM_MOUSEMOVE`/cursor position get accelerated motion. That is acceptable because Steps are calibrated empirically per machine.
- macOS: `CGEventCreateMouseEvent(source, kCGEventMouseMoved, current+delta, 0)`, set `kCGMouseEventDeltaX/Y` = delta, post to `kCGHIDEventTap`.
- **Split each Step into fixed sub-moves**, for example N chunks of ≤ ~20 px every ~8–10 ms, with the remainder on the last chunk.
  - Big single jumps can trip per-event clamping or smoothing in some engines, and on Windows cross the acceleration thresholds [inferred].
  - Keep the chunking deterministic so calibration stays reproducible.
  - Make the sub-move count/interval a per-Game advanced value, with a sane default.

**Mouse (click-and-drag)**
- Button down → (short settle, ~20–50 ms) → sub-moves → settle → button up.
- On macOS every move while the button is down **must** be `kCGEvent{Left,Right,Other}MouseDragged`, with delta fields and the button number. Track button state ourselves; don't query `NSEvent.pressedMouseButtons`.
- Park the cursor at the centre of the Capture zone before each drag (`CGWarpMouseCursorPosition` / `SetCursorPos` or an absolute move), so position-reading games don't hit the screen edge.

**Gamepad (Windows)**
- Use **ViGEmBus v1.22.0 + `vigem-client` 0.1.4**: pure Rust, MIT, frozen ABI, most-used crate.
- Keep `vigem-rust` as a fallback if a bug appears. Wrap it behind our own small trait so the swap stays cheap.
- Create one `Xbox360Wired` target per Session, `wait_ready()`, then stick deflection for a duration (as in the old code), then neutral, and `unplug` on stop.
- Onboarding:
  - `Client::connect()`: `Ok` → green.
  - `BusNotFound` → red, with "Install ViGEmBus" opening the v1.22.0 release page. Bundling the installer is possible under BSD-3 but needs admin.
  - Explain that the driver is no longer maintained but still works.

**Permissions**
- macOS:
  - Onboarding check uses `AXIsProcessTrustedWithOptions(prompt: false)` and `CGPreflightPostEventAccess()`.
  - The "Grant" button calls `CGRequestPostEventAccess()` / AX with prompt `true`, then opens the Privacy & Security → Accessibility deep link.
  - Poll every ~1 s while onboarding is shown.
  - README/onboarding must say: **after each update, remove Panowww from the Accessibility list and add it again** (unsigned builds lose the grant, TN3127).
- Windows:
  - No onboarding step for keyboard/mouse.
  - At Session start, compare the foreground process integrity level with ours. If it's higher: "This game runs as administrator: run Panowww as administrator too."

---

## 3. Open risks (need hands-on tests)

1. **macOS GCMouse games** (SDL ≥ 3.6, some native ports) probably ignore CGEvent mouse motion entirely. Test against an SDL 3.6 build and a few native Mac games. If confirmed, the Mac mouse mode only works for games reading NSEvents, and the "game ignores input" diagnosis must mention it. Keyboard via `GCKeyboard` is likewise unverified.
2. **Accessibility grant lost on every update** for unsigned/ad-hoc builds. A stable self-signed certificate would pin the DR to the cert instead of the cdhash. That's a cheap mitigation to consider despite "no signing" in v1.
3. **ViGEmBus is unmaintained**. A future Windows release, HVCI/driver blocklist change or anti-cheat could break or block it, and there is no free successor. VHF-based crates (`doppelpad`) are only placeholders today.
4. **Windows UIPI** fails silently. The integrity check covers the known case, but not games launched elevated by a launcher after the Session starts.
5. **Anti-cheat**: injected input is trivially detectable (`LLMHF_INJECTED`) and ViGEm devices are visible. Some online games may ignore input or flag the account. Panowww should warn to use it only in single-player / photo modes. No primary source on actual anti-cheat policies.
6. **Sub-move defaults** (chunk size, interval, drag settle times) are guesses. Calibrate them on 3–4 real games per OS.
7. **Windows acceleration**: whether `SendInput` relative moves reach raw input *before* pointer ballistics (the expected, unaccelerated path) is not stated by Microsoft for injected events. Verify with a raw-input logger.
8. **macOS cursor-visible drag games** that read absolute positions: confirm the "park at zone centre" approach on a strategy-style camera.

## 4. Sources (primary)

- Microsoft: [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput), [MOUSEINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-mouseinput), [KEYBDINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput), [RAWMOUSE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse), [RAWINPUTHEADER](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputheader), [MSLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct), [InjectGamepadInput](https://learn.microsoft.com/en-us/uwp/api/windows.ui.input.preview.injection.inputinjector.injectgamepadinput), [UIAccess secure locations](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-R2-and-2008/dd834830(v=ws.11)).
- Apple: [CGEventField.mouseEventDeltaX](https://developer.apple.com/documentation/coregraphics/cgeventfield/mouseeventdeltax), [CGEventType.leftMouseDragged](https://developer.apple.com/documentation/coregraphics/cgeventtype/leftmousedragged), [CGAssociateMouseAndMouseCursorPosition](https://developer.apple.com/documentation/coregraphics/cgassociatemouseandmousecursorposition(_:)), [CGPreflightPostEventAccess](https://developer.apple.com/documentation/coregraphics/cgpreflightposteventaccess()), [CGRequestPostEventAccess](https://developer.apple.com/documentation/coregraphics/cgrequestposteventaccess()), [AXIsProcessTrustedWithOptions](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions), [TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements).
- Source: [enigo](https://github.com/enigo-rs/enigo) (`v0.6.1` and `main`), [vigem-client](https://github.com/CasualX/vigem-client), [vigem-rust](https://github.com/arounre/vigem-rust), [SDL](https://github.com/libsdl-org/SDL) (cocoa mouse, Windows raw input, hints, issue #16241), [rust-lang/rust#116461](https://github.com/rust-lang/rust/pull/116461).
- ViGEm: [ViGEmBus repo](https://github.com/nefarius/ViGEmBus), [releases](https://github.com/nefarius/ViGEmBus/releases), [EOL statement](https://docs.nefarius.at/projects/ViGEm/End-of-Life/), [VirtualPad](https://docs.nefarius.at/projects/VirtualPad/).
- Prior art: `game-pano-capture` branch `dev`, `control/{keyboard,mouse,gamepad}/*.py`, `pano_capture.py`.
