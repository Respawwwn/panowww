# Research: Capture zone screenshots in Rust on macOS and Windows

Ticket: [003 – Screen zone capture in Rust on macOS and Windows](../.wayfinder/tickets/003-screen-capture.md)
Researched: 2026-10-29. Crate versions are from the crates.io API on that date.

Words follow [GLOSSARY.md](../GLOSSARY.md): a **Shot** is one screenshot of the **Capture zone** taken during a **Session**.

How much to trust each claim:
- **[primary]**: Apple or Microsoft docs, SDK headers, or crate source that I read.
- **[secondary]**: GitHub issues, forums, the press. Confirmed by more than one report but not by the vendor.
- **[estimate]**: my own reasoning, not measured. Check it in the prototype.

---

## 1. Facts

### 1.1 macOS APIs

| API | Status | Notes |
|---|---|---|
| `CGWindowListCreateImage` | **Obsoleted in macOS 15.** Deprecated in 14.0. | SDK header (MacOSX15.2.sdk, `CGWindow.h`): `SCREEN_CAPTURE_OBSOLETE(10.5,14.0,15.0)`, which expands to `availability(macos, introduced=x, deprecated=y, obsoleted=z, message="Please use ScreenCaptureKit instead.")` [primary] |
| `CGDisplayCreateImage` / `CGDisplayCreateImageForRect` | **Obsoleted in macOS 15.** Deprecated in 14.4. | `CGDirectDisplay.h`: `SCREEN_CAPTURE_OBSOLETE(10.6,14.4,15.0)` [primary] |
| `SCScreenshotManager.captureImage(contentFilter:configuration:)` | macOS 14.0+ | Returns a `CGImage`. It is BGRA when `captureDynamicRange` is SDR, and RGhA for the HDR modes (`SCScreenshotManager.h`) [primary]. [Apple docs](https://developer.apple.com/documentation/screencapturekit/scscreenshotmanager) |
| `SCScreenshotManager.captureImage(in:)` | macOS 15.2+ | "rect for the region in points on the screen space … display agnostic and supports multiple displays" (header). It has no filter, so it cannot exclude windows. [primary] [Apple docs](https://developer.apple.com/documentation/screencapturekit/scscreenshotmanager/captureimage(in:completionhandler:)) |
| `SCScreenshotManager.captureScreenshot(rect:configuration:)` + `SCScreenshotConfiguration` | macOS 26.0+ | New screenshot config with `sourceRect`, `dynamicRange`, `fileURL` and `contentType`. [primary] [Apple docs](https://developer.apple.com/documentation/screencapturekit/scscreenshotconfiguration) |
| `SCStreamConfiguration.sourceRect` | macOS 12.3+ | "The rectangle is specified in **points** in the display's logical coordinate system" (`SCStream.h`). If it is unset, the system captures the whole display. [primary] [Apple docs](https://developer.apple.com/documentation/screencapturekit/scstreamconfiguration/sourcerect) |
| `SCContentFilter.pointPixelScale`, `.contentRect` | macOS 14.0+ | Gives the point-to-pixel factor and the content rect in points. Use them to set the output `width`/`height` in **pixels** = sourceRect × scale, so the Shot is at native resolution. [primary] [Apple docs](https://developer.apple.com/documentation/screencapturekit/sccontentfilter/pointpixelscale) |
| `SCContentFilter(display:excludingWindows:)` (also `excludingApplications:exceptingWindows:`) | macOS 12.3+ | Captures a display minus the listed windows or apps. [primary] [Apple docs](https://developer.apple.com/documentation/screencapturekit/sccontentfilter/init(display:excludingwindows:)) |
| `CGPreflightScreenCaptureAccess()` / `CGRequestScreenCaptureAccess()` | macOS 10.15+ | **Not deprecated** in the 15.2 SDK: plain `API_AVAILABLE(macos(10.15))` in `CGWindow.h`. This contradicts the title of [xcap#160](https://github.com/nashaofu/xcap/issues/160). [primary] |
| `CGDisplayBounds`, global display space | — | Points. The origin is the top-left of the main display, so other displays can have negative coordinates (`CGDirectDisplay.h`). [primary] |
| `CGDisplayCreateUUIDFromDisplayID` | macOS 10.4+ | Turns a `CGDirectDisplayID` into a UUID. ColorSync, `ColorSyncDevice.h`. [primary] |

### 1.2 Windows APIs

| API | Facts |
|---|---|
| **Windows.Graphics.Capture (WGC)** | Monitor capture through `IGraphicsCaptureItemInterop::CreateForMonitor` needs Windows 10 1903 (build 18362) or later [primary, [MS](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createformonitor)]. Frames arrive as D3D11 textures. **There is no region API.** You crop after capture (`CopySubresourceRegion`). The maintainer says so in [windows-capture#80](https://github.com/NiiightmareXD/windows-capture/issues/80): "the underlying API doesn't allow capturing an area" [secondary]. With HDR on, the content "might not necessarily be B8G8R8A8"; use `R16G16B16A16_FLOAT` to avoid washed-out captures [primary, [MS screen capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)]. |
| WGC yellow border | `GraphicsCaptureSession.IsBorderRequired` needs build **20348+** (in practice Windows 11). The docs say turning the border off needs user consent via `GraphicsCaptureAccess.RequestAccessAsync(Borderless)` plus the `graphicsCaptureWithoutBorder` capability in the package manifest. If the user denies, setting false "will succeed, but the value will be ignored" [primary, [MS](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturesession.isborderrequired)]. xcap and windows-capture both call `SetIsBorderRequired(false)` from unpackaged Win32 apps and treat a failure as non-fatal (`xcap/src/windows/wgc.rs`) [primary, crate source]. Whether that works without a package manifest is **unverified**, so test it on Windows 11. On Windows 10 the border cannot be removed. Starting and stopping a session for every Shot makes the border flash ([windows-capture#158](https://github.com/NiiightmareXD/windows-capture/issues/158)) [secondary]. |
| **DXGI Desktop Duplication** | `AcquireNextFrame` returns the full output. With `DuplicateOutput` the format "is always `DXGI_FORMAT_B8G8R8A8_UNORM`" [primary, [MS](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)]. `DuplicateOutput1` can return the original back-buffer format (FP16 or R10G10B10A2), so it gets the full gamut for HDR. Other limits: one duplication per output per process, at most 4 duplicating apps by default (`DXGI_ERROR_NOT_CURRENTLY_AVAILABLE`), and `DXGI_ERROR_UNSUPPORTED` on non-DWM modes or after mode switches (re-create on `WM_DISPLAYCHANGE`) [primary, [MS](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_5/nf-dxgi1_5-idxgioutput5-duplicateoutput1)]. There is no yellow border. |
| **GDI `BitBlt`** | This is what xcap uses on Windows **by default**: `BitBlt(..., SRCCOPY)` without `CAPTUREBLT`, unless the `wgc` feature is on (`xcap/src/windows/capture.rs`, `gdi.rs`) [primary]. It is CPU-side and has no HDR path. The old Python tool used `mss`, which is also GDI on Windows. |
| **Exclusive fullscreen** | Since Windows 10, "Fullscreen Optimizations" run most FSE games "in a highly optimized borderless windowed format", and the DWM takes control again when an overlay is shown [primary, [DirectX blog](https://devblogs.microsoft.com/directx/demystifying-full-screen-optimizations/)]. True FSE bypasses DWM composition, and DWM-based capture (WGC, GDI) then commonly returns black or stale frames [secondary]. Panowww already requires borderless windowed (map Notes), so this is a documentation and tip matter, not an engineering one. |
| **DPI** | A process that is not Per-Monitor-V2 aware gets **virtualized** (scaled) coordinates from many APIs. PMv2 "sees the raw pixels of each display" [primary, [MS High DPI](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)]. The virtual-screen origin is the top-left of the primary monitor, and other monitors can be negative. |
| **Excluding our windows** | `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` (Windows 10 2004+): "the window is displayed only on a monitor. Everywhere else, the window does not appear at all". The docs give the example of recording-control windows. It must be a top-level window owned by the calling process, and it works only while DWM composes. On older Windows it falls back to `WDA_MONITOR`, which shows a black box [primary, [MS](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)]. |

### 1.3 Excluding our own overlay on macOS

- `NSWindow.sharingType = .none` is **not** reliable. Apple DTS: it "change[s] how the window content is handled at the CGWindow level, not the broader system level … virtually all screen recording … operate below the window level, which means kCGWindowSharingState has no effect. The behavior here is basically deliberate." ([forum thread 760234](https://developer.apple.com/forums/thread/760234)) [primary-ish, Apple DTS]. Developers report that ScreenCaptureKit ignores it on macOS 15.4+ ([thread 808016](https://developer.apple.com/forums/thread/808016)) [secondary].
- The supported way is the capture filter: build `SCContentFilter(display:excludingApplications:[our SCRunningApplication], exceptingWindows:[])`, or exclude the overlay's `SCWindow` [primary, header]. This needs a filter-based call (`captureImage(contentFilter:configuration:)` or an `SCStream`), **not** `captureImage(in:)`.

### 1.4 Screen Recording permission (macOS)

- Detect with `CGPreflightScreenCaptureAccess()`, which never prompts. Request with `CGRequestScreenCaptureAccess()`, which prompts once. If the user already declined, it only returns false, so the UI must deep-link to System Settings › Privacy & Security › Screen & System Audio Recording (`x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture`) [primary for the functions; the URL scheme is widely used but undocumented].
- After a grant, the process usually has to **relaunch** before capture works. The screencapturekit-rs README says "Restart the app" [secondary].
- **macOS 15 Sequoia re-prompts.** Apps that capture without going through the system `SCContentSharingPicker` get a recurring "… is requesting to bypass the system private window picker" alert. It was weekly in betas and has been **monthly** since 15.0 final / 15.1. Developers report it applies to pure-ScreenCaptureKit apps too ([9to5Mac](https://9to5mac.com/2024/08/14/macos-sequoia-screen-recording-prompt-monthly/), [TidBITS](https://tidbits.com/2024/08/19/apple-reduces-excessive-sequoia-permission-requests-shifts-to-monthly)) [secondary]. Apple has published no developer documentation on the cadence.
- TCC ties the grant to the app's code identity. For ad-hoc-signed or unsigned builds (our v1 plan), each new build probably has a different identity, so users may need to **re-grant after every update** [estimate; based on how TCC designated requirements work; verify].

### 1.5 Crates (versions as of 2026-10-29)

| Crate | Version / last release | macOS backend | Windows backend | Verdict |
|---|---|---|---|---|
| [`xcap`](https://github.com/nashaofu/xcap) | 0.9.8 / 2026-08-01, MIT/Apache, ~2.6 M downloads | **`CGWindowListCreateImage`** (`src/macos/capture.rs`), obsoleted in macOS 15. Open issue to move to SCK: [#253](https://github.com/nashaofu/xcap/issues/253) (2026-01, maintainer: "pr is welcome"). Reported hang after minutes: [#209](https://github.com/nashaofu/xcap/issues/209) | GDI by default, WGC behind the `wgc` feature (one session per call: `StartCapture` then `recv_timeout(3s)`) | Easy API (`Monitor::capture_region` → `RgbaImage`), but the macOS backend is on borrowed time. Its `capture_region` takes **points** relative to the monitor on macOS. |
| [`screencapturekit`](https://github.com/doom-fish/screencapturekit-rs) | 11.1.1 / 2026-10-26, MIT OR Apache-2.0, ~3 M downloads | Full ScreenCaptureKit: `SCScreenshotManager`, `SCStream`, filters, `sourceRect`, HDR, picker. Feature flags `macos_14_0` … `macos_26_0` | — | Best maintained. Its **build compiles a Swift bridge with SwiftPM** (`build.rs` runs `swift build`), so it needs Xcode Command Line Tools at build time (fine on GitHub macOS runners). Very active, with frequent majors (v9 to v11 in 2026). |
| [`objc2-screen-capture-kit`](https://github.com/madsmtm/objc2) | 0.3.2 / 2025-10-04 | Raw auto-generated ScreenCaptureKit bindings, no Swift | — | The pure-Rust fallback if the Swift build step becomes a problem. More `unsafe` code. |
| [`windows-capture`](https://github.com/NiiightmareXD/windows-capture) | 2.0.1 / 2026-08-08, MIT, ~2.4 M downloads | — | WGC (`GraphicsCaptureApiHandler`; `DrawBorderSettings`, `CursorCaptureSettings`, `ColorFormat::{Bgra8, Rgba8, Rgba16F}`, `Frame::buffer_crop`) **plus DXGI Desktop Duplication** since 2.0 (`dxgi_duplication_api.rs`: `DuplicateOutput1` with FP16/RGBA8/BGRA8, `acquire_next_frame`, `buffer_crop`) | Best Windows choice. One crate covers both GPU paths. |
| [`scap`](https://github.com/helmerapp/scap) | 0.0.8 / 2025-08-04, last push 2025-08-05 | Old `screencapturekit` 0.2.8 | `windows` 0.58 (WGC) | **Stale**: 15 months without a push, outdated dependencies. Avoid. |
| [`png`](https://github.com/image-rs/image-png) | 0.18.1 / 2026-02 | `Compression::{NoCompression, Fastest, Fast, Balanced, High}`. `Fast` uses the PNG-tuned [`fdeflate`](https://github.com/image-rs/fdeflate) encoder, which "outperforms libpng" ([docs.rs](https://docs.rs/png/0.18.1/png/enum.Compression.html)) [primary] | | Encoder to use. |
| [`image`](https://github.com/image-rs/image) | 0.25.10 | Wraps `png`. `CompressionType` defaults to `Fast` ([docs.rs](https://docs.rs/image/0.25.10/image/codecs/png/enum.CompressionType.html)) | | Convenient but optional. |
| [`fast_image_resize`](https://github.com/cykooz/fast_image_resize) | 6.1.0 / 2026-07 | SIMD (NEON, SSE4.1, AVX2). RGBA8 4928×3279 → 852×567: 24–27 ms Lanczos3 and 11–13 ms bilinear with SIMD, against 217 ms for `resize` (README benchmarks) [primary, crate's own benchmark] | | For thumbnails. |

### 1.6 Latency and cost per Shot

- ScreenCaptureKit single screenshot: the crate README gives a first-frame latency of **30–100 ms at 1080p and 50–150 ms at 4K** [secondary, crate authors' numbers]. `SCScreenshotManager` sets up a capture each call. A long-lived `SCStream` with `sourceRect` and a low `minimumFrameInterval` keeps the latest frame ready, which brings it down to about one frame interval [estimate].
- WGC: the first frame after `StartCapture` costs a session start-up, and restarting per Shot makes the border flash. Keep **one session for the whole Session** and read the latest frame per Shot. WGC delivers a new frame only when content changes, so keep the last received frame and use `SystemRelativeTime` (QPC) to check that it is newer than the end of the "wait after move" delay [primary for QPC timestamps; the design is an estimate].
- Desktop Duplication: `AcquireNextFrame(timeout)` likewise returns only on change. Keep the last texture.
- GPU→CPU readback of a cropped 2560×1440 BGRA texture is about 14.7 MB, a few ms [estimate].
- PNG encode, 2560×1440 RGBA, single thread: I found no published MB/s for `png`'s `Fast`. My estimate is tens of ms for `Fast` and hundreds of ms for `Balanced`/`High` [estimate; **benchmark in the prototype**]. Game frames compress worse than UI. Drop alpha (encode RGB8), because games give opaque pixels and that saves 25 % of the input.
- Thumbnail: downscale the in-memory buffer with `fast_image_resize` (about 1–5 ms for 2560×1440 → 320×180 bilinear, scaled from the benchmark above [estimate]). Send it to the UI as a small JPEG/PNG or raw bytes, and never re-read the PNG from disk.

### 1.7 Coordinates and persisting a Capture zone

- macOS: ScreenCaptureKit wants **points** in display-local or global space, and the output size is in pixels. Pixels = points × `pointPixelScale`. `CGDirectDisplayID` is not guaranteed stable across reboots or replugs; `CGDisplayCreateUUIDFromDisplayID` gives a stable UUID [primary for the API; stability is the documented intent of the UUID].
- Windows: with PMv2 (Tauri/tao already declares per-monitor-v2 awareness; check this in the prototype), monitor rects are in **physical pixels** in virtual-screen space. `HMONITOR` is not persistent. A stable key is the monitor device path from `QueryDisplayConfig` + `DisplayConfigGetDeviceInfo(DISPLAYCONFIG_TARGET_DEVICE_NAME)` [primary for the API; stability is an estimate].

---

## 2. Recommendation

1. **Build a `ZoneCapturer` per OS behind one Rust trait** (`start(monitor, zone)`, `shot() -> RgbaImage`, `stop()`), with one long-lived capture per Session. Do not adopt a cross-platform crate: `xcap` uses an obsoleted macOS API and `scap` is stale.
2. **macOS → `screencapturekit` (11.x), minimum macOS 14.** Use `SCContentFilter(display, excludingApplications: [Panowww])` so the overlay and main window never appear. Set `sourceRect` = the Capture zone in display points, output `width`/`height` = points × `pointPixelScale`, `showsCursor = false`, and SDR BGRA. Start with `SCScreenshotManager::capture_image(filter, config)` per Shot because it is simplest. Move to a persistent `SCStream` only if measured latency matters. Keep `objc2-screen-capture-kit` as the fallback if the Swift build step causes trouble.
3. **Windows → `windows-capture` (2.x), WGC monitor capture, minimum Windows 10 1903 (border-free on Windows 11).** Use one session per Session, `DrawBorderSettings::WithoutBorder`, `CursorCaptureSettings::WithoutCursor`, `Bgra8`, and `buffer_crop` to the zone. Set `WDA_EXCLUDEFROMCAPTURE` on the overlay HWND and main window HWND. Keep the crate's DXGI Desktop Duplication path as the fallback when WGC is unsupported or the border cannot be disabled. It has no border and a `DuplicateOutput1` FP16 option.
4. **Persist the Capture zone per Game on this machine** as `{ display_key, x, y, w, h }`, in display-local **logical points** on macOS and display-local **physical pixels** on Windows. Store the display's pixel size and scale with it. `display_key` is the CG display UUID on macOS and the monitor device path on Windows. On load, if the display is missing or its size or scale changed, mark the zone invalid and ask the user to redraw it. Never silently rescale.
5. **Encoding:** `png` 0.18, `Compression::Fast`, RGB8 (no alpha), on a worker thread so the next move is not blocked. This is lossless at every level, so the level only trades size for time. Produce the thumbnail from the same buffer with `fast_image_resize`.
6. **HDR:** v1 is SDR only. On Windows, detect HDR (`IDXGIOutput6::GetDesc1().ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020`) and show a tip to turn HDR off, because an 8-bit capture of an HDR desktop looks washed out. On macOS, request SDR (`captureDynamicRange = SDR`).
7. **Overlay design:** draw the Capture zone frame **outside** the zone rectangle, so it never lands in a Shot even if exclusion fails.
8. **Permission onboarding (macOS):** check with `CGPreflightScreenCaptureAccess`, request once with `CGRequestScreenCaptureAccess`, then show "Open settings" and "Relaunch Panowww". Warn in the README about the monthly Sequoia re-prompt and about re-granting after updating an unsigned build.

---

## 3. Open risks (to test in a prototype)

- **Unsigned builds and TCC:** permission may reset on every update [estimate]. That is bad UX for an unsigned v1, and it is a reason to revisit ad-hoc signing with a stable identity.
- **WGC border on unpackaged Win32 apps:** the docs require a manifest capability plus consent. The crates call it anyway. Verify on Windows 11 whether the border actually disappears. If not, use DXGI Desktop Duplication as the primary Windows path.
- **PNG encode time** at 1440p and 4K with `Fast` vs `Balanced` is unmeasured. Benchmark it on real game frames on both OSes.
- **ScreenCaptureKit latency per screenshot** (30–150 ms claimed) and whether the monthly Sequoia alert can interrupt a Session mid-run.
- **The frame must be fresh:** the Shot must come from after the camera move plus the wait. Both WGC and DD deliver only on change, so check timestamps.
- **`screencapturekit` crate churn:** it ships frequent major versions and builds Swift at compile time. Pin the major version.
- **macOS 14 floor:** `SCScreenshotManager` needs 14.0. To support 12.3–13 you would need an `SCStream` one-shot. That is probably not worth it.
- **Exclusive fullscreen and HDR games** are unsupported in v1. Cover them in the in-app tips: borderless windowed, HDR off.
- **Mixed-DPI multi-monitor setups:** drawing the zone in a Tauri overlay (logical px) and converting to capture units must be tested on 100 % + 150 % setups and on Retina + non-Retina.
