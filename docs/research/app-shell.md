# Research: App shell (Tauri vs pure-Rust UI)

Planning ticket 001 *App shell: Tauri or pure Rust* (the planning map is not published). Researched 2026-10-29.

Statements marked **(inference)** are my reasoning, not something a source says. Everything else links to the source that owns it.

## 1. Current versions (crates.io, 2026-10-29)

| Crate | Latest stable | Notes |
|---|---|---|
| `tauri` | 2.12.5 (2026-10-21) | 3.0.0-alpha.5 exists, so stay on 2.x ([crates.io](https://crates.io/crates/tauri)) |
| `tao` (Tauri windowing) | 0.37.2 (git HEAD) | [repo](https://github.com/tauri-apps/tao) |
| `tauri-plugin-global-shortcut` | 2.4.1 | needs Rust ≥ 1.90 ([docs](https://v2.tauri.app/plugin/global-shortcut/)) |
| `global-hotkey` (used by the plugin) | 0.9.0 | [repo](https://github.com/tauri-apps/global-hotkey) |
| `tauri-nspanel` | 2.1.1 | macOS NSPanel for Tauri 2 ([repo](https://github.com/ahkohd/tauri-nspanel)) |
| `window-vibrancy` | 0.8.1 | [repo](https://github.com/tauri-apps/window-vibrancy) |
| `eframe`/`egui` | 0.37.0 | [docs](https://docs.rs/egui/latest/egui/viewport/struct.ViewportBuilder.html) |
| `iced` | 0.14.0 (2025-12-07) | |
| `slint` | 1.18.1 | |
| `gpui` | 0.2.2 (2025-10-22) | pre-1.0, few releases |

## 2. Overlay window: always-on-top, transparent, click-through, non-activating

### Tauri, all platforms (config and API)
- The `WindowConfig` fields `transparent`, `alwaysOnTop`, `focus` (initial focus, default `true`), `focusable` (default `true`), `skipTaskbar` (Windows/Linux), `decorations`, `shadow` and `visibleOnAllWorkspaces` (unsupported on Windows) all exist ([config reference](https://v2.tauri.app/reference/config/#windowconfig)).
- `macOSPrivateApi` (the transparent background API) is "No-op in Tauri 2.12.1+ because the APIs are always enabled now" ([config reference](https://v2.tauri.app/reference/config/)). So a transparent window on macOS no longer needs the private-API flag.
- `WebviewWindow::set_ignore_cursor_events`, `set_focusable`, `set_always_on_top` and `set_content_protected` exist in 2.12.5. macOS caveat on `set_focusable(false)`: a window that is already focused cannot be unfocused this way ([docs.rs](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html)).

### Tauri on Windows (tao source)
- `focusable: false` adds `WS_EX_NOACTIVATE` (`tao/src/platform_impl/windows/window_state.rs`, `if !self.contains(WindowFlags::FOCUSABLE) { style_ex |= WS_EX_NOACTIVATE; }`).
- Ignoring cursor events adds `WS_EX_TRANSPARENT | WS_EX_LAYERED` (same file, line ~283). These are the standard Win32 click-through flags.
- Windows created with `focus: false` are shown with `SW_SHOWNOACTIVATE` (same file, around `MARKER_DONT_FOCUS`).
- **Behaviour change in tao 0.37.1 (2026-09-26):** "Windows created with `with_focused(false)` will now receive focus when `set_visible(true)` is called later" ([release notes](https://v2.tauri.app/release/tao/v0.37.1/)). If you hide and re-show the overlay, rely on `focusable: false` (`WS_EX_NOACTIVATE`), not on `focus: false`. Test this specifically.
- The focusable/unfocusable window feature arrived in tao 0.34.1 and in tauri-runtime-wry 2.8.0 ([tao 0.34.1](https://v2.tauri.app/release/tao/v0.34.1/), [tauri-runtime-wry 2.8.0](https://v2.tauri.app/release/tauri-runtime-wry/v2.8.0/)).
- **(inference)** A layered, topmost, `WS_EX_NOACTIVATE|WS_EX_TRANSPARENT` window is DWM-composited above borderless-windowed games. It cannot draw over *exclusive* fullscreen games, which matches the map's "borderless windowed" requirement.

### Tauri on macOS
- tao's macOS window answers `canBecomeKeyWindow`/`canBecomeMainWindow` from the `focusable` flag (`tao/src/platform_impl/macos/window.rs`). But `set_visible(true)` calls `make_key_and_order_front_sync`, and the window stays a plain `NSWindow`, not a non-activating `NSPanel`.
- `tauri-nspanel` 2.1 turns a Tauri window into an `NSPanel` and exposes:
  - `StyleMask::nonactivating_panel()`
  - `CollectionBehavior::full_screen_auxiliary().can_join_all_spaces()`
  - `PanelLevel`
  - `set_ignores_mouse_events`
  - `order_front_regardless`

  Its `examples/fullscreen` combines a non-activating mask with full-screen-auxiliary and all-spaces behaviour, which is exactly the overlay case ([repo, v2.1 branch](https://github.com/ahkohd/tauri-nspanel/tree/v2.1), `src/builder.rs`, `src/lib.rs`).
- **(inference)** Many macOS games' "borderless" mode is a native full-screen Space. `full_screen_auxiliary` is what lets the overlay appear on that Space, so use `tauri-nspanel` for the overlay on macOS.

### Pure Rust (egui/eframe 0.37, built on winit)
- `ViewportBuilder::with_mouse_passthrough` ("mouse clicks pass through the window, used for non-interactable overlays"), `with_transparent`, `with_window_level` (always on top), `with_active` (initial focus only) and `with_taskbar` (Windows) all exist ([docs.rs](https://docs.rs/egui/latest/egui/viewport/struct.ViewportBuilder.html)).
- egui has no non-activating NSPanel or full-screen-auxiliary API. On macOS you would write the same raw AppKit code (`objc2`) that `tauri-nspanel` already packages. **(inference)** The overlay is no easier in pure Rust. Both stacks need some platform code, and Tauri already has a maintained crate for the hard macOS part.

## 3. Global shortcuts while the game has focus

### What Tauri uses
- `tauri-plugin-global-shortcut` wraps the `global-hotkey` crate.
- On **Windows** it calls `RegisterHotKey` with `MOD_NOREPEAT` (`global-hotkey/src/platform_impl/windows/mod.rs`).
- On **macOS** it calls Carbon `RegisterEventHotKey` (`global-hotkey/src/platform_impl/macos/mod.rs`). A `CGEventTap` is created only for media keys. **(inference)** Ordinary hotkeys therefore need no Accessibility permission. Panowww needs Accessibility anyway for input injection.

### Windows facts (Microsoft docs)
- `RegisterHotKey` "defines a system-wide hot key". On a match the system posts `WM_HOTKEY` to the registering thread or window. **F12 is reserved for the debugger** and must not be used. `MOD_WIN` combos are reserved for the OS. Registration fails if another app already holds the combo ([RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)).
- **Key risk: `RIDEV_NOHOTKEYS`.** A raw-input application can register the keyboard with `RIDEV_NOHOTKEYS`, and then "the application-defined keyboard device hotkeys are not handled. However, the system hotkeys … are still handled" ([RAWINPUTDEVICE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice)). **(inference)** A game that sets this flag while focused will silently stop Panowww's `RegisterHotKey` shortcuts. Some games do this to block the Windows key and other hotkeys.
- Raw input is non-exclusive. A background app can receive it with `RIDEV_INPUTSINK` (requires `hwndTarget`). `RIDEV_EXINPUTSINK` delivers input only when the foreground app is not itself registered for raw input ([RAWINPUTDEVICE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice), [Raw Input overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)).
- Low-level keyboard hooks (`WH_KEYBOARD_LL`) see keys before they are posted to the target queue. The hook must answer within `LowLevelHooksTimeout`, capped at 1 s since Windows 10 1709. Otherwise "the hook is silently removed". Microsoft recommends raw input over LL hooks for monitoring ([LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)).
- **(inference)** Does a raw-input or DirectInput game still *see* the keystrokes of a registered hotkey? `RegisterHotKey` swallows the legacy `WM_KEYDOWN` for the foreground app. Whether that game still gets `WM_INPUT` for it is not documented. Treat it as "the game may also react", and pick combos that no game binds. Modern DirectInput keyboard data is commonly built on raw input. This is not verified in a primary source.
- **Robust fallback (inference, built on the documented flags above):** in Panowww's Rust core, register a hidden window for keyboard raw input with `RIDEV_INPUTSINK` and match the combo yourself. This keeps working even when the game sets `RIDEV_NOHOTKEYS`, because it is not a hotkey. It also does not consume the key, so combo choice matters even more. Ignore Panowww's own injected keys (the raw input header's `hDevice` is NULL for injected input; verify). Keep `RegisterHotKey` as the primary path.

### macOS facts
- Since macOS 15 Sequoia, `RegisterEventHotKey` rejects hotkeys whose only modifiers are Option or Option+Shift. Apple staff called this intentional, to stop keyloggers. Control- or Command-based combos are unaffected ([Apple Developer Forums thread 763878](https://developer.apple.com/forums/thread/763878), [Shottr KB](https://shottr.cc/kb/sequoia)).

### Safe combo guidance (from the facts above)
- Use **Ctrl+Shift+F-key** (F12 excluded): Ctrl+Shift+F8 for start and Ctrl+Shift+F9 for stop on Windows. On macOS use Control+Shift (or Cmd+Shift) plus an F-key. **Never** Option-only or Option+Shift-only on macOS, never `Win` combos, never F12.
- Show a clear error when registration fails (combo already taken) and let users rebind.
- Note that on Macs, F-keys may need `fn` unless "Use F1, F2… as standard function keys" is on. **(inference)** Document this in the in-app tip.

## 4. Performance while a game runs

- Tauri uses the OS webview: WebView2 (Chromium) on Windows, WKWebView on macOS.
- A WebView2 process group is "a single browser process, one or more renderer processes, other helper processes, such as the GPU process" ([WebView2 process model](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)). Expect several extra processes and tens of MB of RAM over a pure-Rust UI.
- WebView2 lets an app lower memory while inactive: set `MemoryUsageTargetLevel` to `LOW` and scripts keep running. "This is useful for inactive apps that still want to run scripts", with `TrySuspend`/`Resume` as the alternative ([ICoreWebView2_19](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_19)). **(inference)** Panowww can set LOW when the game is focused. This needs access to the controller through Tauri's `with_webview`.
- **(inference)** A static web UI that is not animating costs near-zero CPU. The real CPU and GPU load during a Session comes from screen capture, PNG encoding and input timing, all in Rust, whichever shell is chosen. An immediate-mode toolkit such as egui repaints only on input or request by default, but a continuous-repaint mistake costs more there than in a DOM. Neither option is a measurable threat to game FPS if the UI is idle during a Session. Measure in a prototype.
- **Thumbnails:** Tauri says the event system "is not designed for low latency or high throughput" and uses JSON payloads. **Channels** "are designed to be fast and deliver ordered data" ([Calling the frontend](https://v2.tauri.app/develop/calling-frontend/)).
  - **(inference)** Downscale each Shot in Rust to a small JPEG or WebP (about 320 px wide).
  - Then either send its bytes or path over a Channel, or save it next to the Shot and load it through the asset protocol or a custom URI scheme.
  - At a few Shots per second (each Shot waits for "wait after move" and "wait after Shot"), this is trivial.

## 5. Native look (Sindre Sorhus-like on macOS, decent on Windows)

- Sindre Sorhus's apps are SwiftUI. No Rust option reproduces AppKit controls exactly.
- **Tauri:**
  - macOS: `titleBarStyle: "Overlay"` plus `trafficLightPosition` gives the unified, transparent title bar ([config](https://v2.tauri.app/reference/config/#windowconfig)).
  - `window-vibrancy` gives `apply_vibrancy` (NSVisualEffectView, macOS 10.10+) and `apply_liquid_glass` (macOS 26+).
  - Windows: `apply_mica` (Windows 11). Acrylic and blur are noted to perform badly while resizing or dragging on some builds ([window-vibrancy README](https://github.com/tauri-apps/window-vibrancy)).
  - **(inference)** With the system font (`-apple-system`/`system-ui`), `prefers-color-scheme` and careful CSS, a web UI gets visually closest to a Sorhus-style app of all options. It is also the easiest to iterate on.
- **Slint:** has a `cupertino` style, and `native` maps to cupertino on macOS and fluent on Windows ([Slint styles](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/)). The styles imitate the platform look rather than use native controls. Licensing: GPLv3, Royalty-free (requires disclosing Slint use, e.g. `AboutSlint` or a badge) or Commercial ([LICENSE.md](https://github.com/slint-ui/slint/blob/master/LICENSE.md)). Royalty-free fits an MIT app but adds an attribution duty.
- **egui:** custom immediate-mode look. Far from native without heavy theming. **iced:** custom look, pre-1.0 (0.14). **GPUI:** pre-1.0 with sparse docs and few crates.io releases.

## 6. Packaging and CI

- Tauri bundles a `.app`/`.dmg` on macOS and NSIS `-setup.exe` or MSI (WiX) on Windows. MSI builds only on Windows ([Windows installer](https://v2.tauri.app/distribute/windows-installer/)).
- WebView2 ships with Windows 10 (April 2018+) and Windows 11. The default `downloadBootstrapper` adds 0 MB, `embedBootstrapper` about 1.8 MB, and `offlineInstaller` about 127 MB ([same page](https://v2.tauri.app/distribute/windows-installer/)).
- Size tuning: `codegen-units = 1`, `lto = true`, `opt-level = "s"`, `panic = "abort"`, `strip = true` ([App size](https://v2.tauri.app/concept/size/)). **(inference)** Expect a few MB to around 15 MB installers. A pure-Rust app with a wgpu or glow renderer lands in a similar range, so size is not a differentiator.
- CI: `tauri-apps/tauri-action@v1` with a matrix of `macos-latest --target aarch64-apple-darwin`, `macos-latest --target x86_64-apple-darwin` and `windows-latest`. It builds and creates a draft GitHub Release ([GitHub pipeline guide](https://v2.tauri.app/distribute/pipelines/github/)). This fits "unsigned binaries on GitHub Releases". A pure-Rust app needs hand-written bundling (`cargo-bundle`/`cargo-packager`) and a release workflow.

## Recommendation

**Use Tauri 2.x (2.12.5 now; do not adopt 3.0 alphas), keeping all capture, input and timing logic in the Rust core.**

1. No pure-Rust toolkit removes the platform-specific overlay work. Tauri already exposes `focusable: false` (`WS_EX_NOACTIVATE`), click-through (`WS_EX_TRANSPARENT|WS_EX_LAYERED`), transparency and always-on-top. `tauri-nspanel` packages the non-activating, full-screen-auxiliary NSPanel that macOS needs.
2. Global shortcuts behave the same in both stacks, because both end up on `RegisterHotKey`/`RegisterEventHotKey` (the `global-hotkey` crate is toolkit-agnostic). The UI shell does not decide this question.
3. A web UI with vibrancy and an overlay title bar gets closest to the Sorhus look. `tauri-action` gives both-OS GitHub Releases for free.
4. The webview's cost is mostly RAM, which the `MemoryUsageTargetLevel` LOW setting can reduce on Windows. CPU impact is negligible if the UI stays idle while a Session runs.

Implementation guidance for the spec:
- **Overlay:** a dedicated second window, created at startup and hidden.
  - Config: `transparent`, `decorations: false`, `alwaysOnTop`, `focusable: false`, `focus: false`, `skipTaskbar`, `shadow: false`; call `set_ignore_cursor_events(true)`.
  - macOS: convert it with `tauri-nspanel` (nonactivating mask, `full_screen_auxiliary | can_join_all_spaces`, high level) and show it with `order_front_regardless`.
  - Draw the countdown and the Capture zone frame in plain HTML/CSS.
- **Shortcuts:** `tauri-plugin-global-shortcut` from Rust. Defaults: Ctrl+Shift+F8 (start) and Ctrl+Shift+F9 (stop) on Windows; Control+Shift+F8/F9 on macOS. Make them rebindable, and show a clear message when registration fails. On Windows, add a raw-input `RIDEV_INPUTSINK` watcher as a fallback for games that set `RIDEV_NOHOTKEYS`.
- **Thumbnails:** downscale in Rust, then deliver over a Channel or the asset protocol. Never send full PNG files through IPC.
- During a Session, keep the main window static (no animations); on Windows set WebView2 memory target LOW.

## Open risks (to verify in a throwaway prototype)

1. **Windows focus regression:** tao 0.37.1 makes `set_visible(true)` focus windows created with `focus: false`. Verify that `focusable: false` (`WS_EX_NOACTIVATE`) still prevents activation when the overlay is re-shown over a game.
2. **macOS overlay over a full-screen-Space game:** verify that `tauri-nspanel` with `full_screen_auxiliary` shows over at least one real game without activating Panowww, and that `ignores_mouse_events` passes clicks through.
3. **Hotkeys vs raw-input games:**
   - Unknown whether a game using raw input or DirectInput still receives the hotkey's keystrokes. Unbound combos make this harmless.
   - Games with `RIDEV_NOHOTKEYS` disable `RegisterHotKey` entirely; the raw-input sink fallback is not yet proven.
   - Elevated (admin) games and UIPI were not researched.
4. **Fallback still needed:** if 1–2 fail for some games, keep the accepted fallback (countdown in the app window, "Switch to the game").
5. **WebView2 RAM:** no first-party numbers found. Measure idle RAM and CPU of the Tauri shell on Windows next to a running game.
6. **macOS `set_focusable(false)` caveat:** an already-focused window cannot be unfocused, so create the overlay unfocusable from the start.
