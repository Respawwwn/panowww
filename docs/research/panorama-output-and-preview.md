# Research: Panorama file format and the 360° preview in the webview

Stitch planning ticket 003 *Panorama file format and the 360° preview in the webview* (the planning map is not published).
Researched: 2026-11-13. Crate and package versions are from crates.io, docs.rs and jsDelivr on that date.

Words follow [GLOSSARY.md](../GLOSSARY.md): a **Session** is one capture run; its **Shots** are the source images. The **Panorama** is the equirectangular image a Stitch produces from them (new term, settled on the scope ticket).

How much to trust each claim:
- **[primary]**: Google, Adobe, W3C, Khronos, Apple, Microsoft, Tauri or viewer docs and source that I read, the human's own repositories, or a file on this Mac.
- **[secondary]**: GitHub issues, forums, surveys, the press. Confirmed by more than one report but not by the vendor.
- **[estimate]**: my own reasoning or a quick measurement with other tools, not the final code. Check it in the spike.

---

## 1. The Panorama file

### 1.1 Shape and size

- A full photo sphere is 360° × 180°, so the file is **2:1**. Google's example is 4000 × 2000. "If an image is scaled, the aspect ratio must be preserved" or it must not be shown as a sphere [primary, [Google](https://developers.google.com/streetview/spherical-metadata)]. Photo Sphere Viewer states the same rule: "The Full panorama width / Full panorama height ratio must always be 2:1" [primary, [PSV](https://photo-sphere-viewer.js.org/guide/adapters/equirectangular.html)].
- **What PTGui produced from a real Session.** `~/Pictures/Panowww/World of Warcraft/2026-11-07_23-14-46/r01_c01 Panorama.jpg` (98 Shots of 1669 × 841) is a **5234 × 2617 JPEG of 5.3 MB** [primary, `sips` + `exiftool` on the file]. That is PTGui's "optimum size" for this grid; 10 000 × 5 000 is an upper bound for a larger grid, not the norm.

### 1.2 JPEG vs PNG at 10 000 × 5 000

Measured with Pillow 12 (libjpeg-turbo / libpng) on an Apple M3 Max, on a 10 000 × 5 000 canvas tiled from 42 real World of Warcraft Shots. Not the Rust crates, so the ratios matter more than the absolute times [estimate]:

| Encode | Size | Time |
|---|---|---|
| JPEG quality 90, 4:4:4 | 16.3 MB | 0.22 s |
| JPEG quality 85, 4:2:0 | 9.7 MB | 0.10 s |
| PNG default (zlib 6) | 68.6 MB | 2.42 s |
| PNG fast (zlib 1) | 101.0 MB | 1.12 s |
| Resize to 8192 × 4096 (Lanczos) + JPEG 85 | 6.9 MB | 0.53 s + 0.08 s |

- Decoded RGBA at 10 000 × 5 000 is **200 MB**; 8192 × 4096 is 134 MB; 4096 × 2048 is 34 MB [estimate, arithmetic].
- The pure-Rust `image::codecs::jpeg::JpegEncoder` (already used by `app/src/runner.rs` at quality 85) is slower than libjpeg-turbo; expect a few seconds, not 0.2 s, at this size. `png 0.18` with `Compression::Fast` (already used by `crates/store/src/png.rs`) is the PNG row above [estimate].
- PNG is lossless but 4–7× larger; JPEG at quality 90 loses nothing visible on game renders and is what PTGui and the sharing targets below expect. Nothing in the targets below reads PNG metadata.

### 1.3 The GPano XMP block

Namespace `http://ns.google.com/photos/1.0/panorama/`, prefix `GPano`, serialised as Adobe XMP inside the image [primary, [Google](https://developers.google.com/streetview/spherical-metadata)].

| Property | Required | Value for a Panowww full sphere |
|---|---|---|
| `ProjectionType` | yes; Google supports only `equirectangular` | `equirectangular` |
| `CroppedAreaImageWidthPixels`, `CroppedAreaImageHeightPixels` | yes; must equal the actual size | W, H |
| `FullPanoWidthPixels`, `FullPanoHeightPixels` | yes | W, H |
| `CroppedAreaLeftPixels`, `CroppedAreaTopPixels` | yes | 0, 0 |
| `UsePanoramaViewer` | no, default True | `True` |
| `PoseHeadingDegrees` | no ("needed for Google Maps display"), 0 ≤ h < 360 | omit (no compass in a game) |
| `InitialViewHeadingDegrees` / `PitchDegrees` / `RollDegrees` | no, default 0 | the view the Session started from, or 0 |
| `InitialHorizontalFOVDegrees` | no | optional |
| `StitchingSoftware`, `CaptureSoftware` | no | `Panowww <version>` |
| `SourcePhotosCount`, `FirstPhotoDate`, `LastPhotoDate` | no | Shots count, first and last Shot time (Panowww knows them) |

PTGui writes exactly this subset: `ProjectionType`, `UsePanoramaViewer`, the four `CroppedArea*`, the two `FullPano*`, the three `InitialView*` (heading 180) and `StitchingSoftware = "PTGui Pro 13.10 (www.ptgui.com)"`; no `PoseHeadingDegrees` [primary, `exiftool -XMP-GPano:all` on the file above]. Google's own example (trimmed to the required and useful lines) [primary]:

```xml
<rdf:Description rdf:about="" xmlns:GPano="http://ns.google.com/photos/1.0/panorama/">
  <GPano:UsePanoramaViewer>True</GPano:UsePanoramaViewer>
  <GPano:StitchingSoftware>Photo Sphere</GPano:StitchingSoftware>
  <GPano:ProjectionType>equirectangular</GPano:ProjectionType>
  <GPano:CroppedAreaLeftPixels>0</GPano:CroppedAreaLeftPixels>
  <GPano:CroppedAreaTopPixels>0</GPano:CroppedAreaTopPixels>
  <GPano:CroppedAreaImageWidthPixels>4000</GPano:CroppedAreaImageWidthPixels>
  <GPano:CroppedAreaImageHeightPixels>2000</GPano:CroppedAreaImageHeightPixels>
  <GPano:FullPanoWidthPixels>4000</GPano:FullPanoWidthPixels>
  <GPano:FullPanoHeightPixels>2000</GPano:FullPanoHeightPixels>
</rdf:Description>
```

The block sits in the usual wrapper: `<?xpacket begin="﻿" id="W5M0MpCehiHzreSzNTczkc9d"?><x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">…</rdf:RDF></x:xmpmeta><?xpacket end="w"?>` [primary, XMP Part 3 §1.1.3.1: "only the standard portion contains a complete packet wrapper"; PTGui's file carries `<x:xmpmeta xmlns:x="adobe:ns:meta/" …>`].

### 1.4 Embedding the block

**JPEG** [primary, [XMP Specification Part 3](https://github.com/adobe/xmp-docs/raw/master/XMPSpecifications/XMPSpecificationPart3.pdf) §1.1.3, Table 6]:
- Segment `APP1` (`0xFFE1`), length = `2 + 29 + packet`, then the null-terminated namespace `http://ns.adobe.com/xap/1.0/` (29 bytes with the null), then the packet.
- "The XMP APP1 … must be placed before the first SOF marker." Recommended order: Exif APP1, XMP APP1, PSIR APP13, then the rest. Panowww writes no Exif, so: SOI, the encoder's APP0 (JFIF), XMP APP1, the rest.
- One segment holds at most **65 502 bytes** of packet; "typically, it is around 2 KB". Ours is about 1 KB, so no ExtendedXMP.

**PNG** [primary, [W3C PNG 3rd ed.](https://www.w3.org/TR/png-3/) §11.3.3.1 and §11.3.3.4; XMP Part 3 §1.1.5]: one `iTXt` chunk, keyword `XML:com.adobe.xmp`, compression flag 0, compression method 0, empty language tag, empty translated keyword, the packet as UTF-8. The PNG spec lists this keyword among its predefined ones and recommends exactly those flag values. `png::Encoder::add_itxt_chunk(keyword, text)` writes it from the `Info` struct, so no hand-splicing for PNG [primary, [docs.rs png](https://docs.rs/png/latest/png/struct.Encoder.html)].

**Crates** (licences all pass `deny.toml`):

| Crate | Version | Licence | Verdict |
|---|---|---|---|
| `img-parts` | 0.4.2 | MIT OR Apache-2.0 | Low-level JPEG/PNG container: `Jpeg::from_bytes`, `segments_mut()`, `JpegSegment::new_with_contents(marker, bytes)`, `Png::chunks_mut()` (a `Vec<PngChunk>`). Helpers only for EXIF and ICC, **no XMP helper**: build the APP1 contents (namespace + null + packet) by hand and insert the segment. Deps: `bytes`, `crc32fast`, `miniz_oxide` [primary, docs.rs]. |
| `xmp-writer` (typst) | 0.3.x | MIT OR Apache-2.0 | Serialises a packet, no dependencies, forbids unsafe. Does not touch files. Custom namespaces possible but GPano is 12 fixed lines; `format!` is simpler [primary, README]. |
| `xmpkit` | 0.1.6 (2026-07) | MIT OR Apache-2.0 | Pure-Rust port of Adobe's toolkit, reads and writes XMP in JPEG and PNG. Young, 0.1 [primary, docs.rs]. |
| `xmp_toolkit` | 1.12.1 | MIT OR Apache-2.0 | Bindings to Adobe's **C++** SDK, built with `cc`, `links = "xmp_toolkit"`, MSVC only on Windows. Against the no-C-library spirit of the map [primary, Cargo.toml + README]. |
| `little_exif` | 0.6.23 | MIT OR Apache-2.0 | EXIF only; no XMP anywhere in its docs [primary, docs.rs]. `kamadak-exif` is read-only. |

Verdict: hand-write the ~1 KB packet with `format!`, insert it with `img-parts` for JPEG (one segment) and with `png`'s `add_itxt_chunk` if a PNG is ever written. About 60 lines, no new transitive weight beyond `img-parts` [estimate].

### 1.5 What the sharing targets actually check

| Target | What it does | Trust |
|---|---|---|
| **Facebook** | Graph API `photo` accepts `spherical_metadata` (`ProjectionType`, the four `CroppedArea*`, the two `FullPano*` required; pose and initial-view fields optional); "If omitted, Facebook tries to generate metadata from the image's embedded data", with `allow_spherical_photo` "only if the server can interpret the photo as spherical, such as through Photosphere XMP". So the GPano block is what the web upload reads. | [primary, [Graph API](https://developers.facebook.com/docs/graph-api/reference/photo/)] |
| Facebook size limits | Reports disagree: 6000 × 3000 max in most guides, 16 000 × 8 000 and "< 30 000 px" in others, 30 MB. Unverified; PTGui's 5234 px is under every figure. | [secondary] |
| **Google Photos** | No help page found. Users report GPano JPEGs are shown as spheres and non-tagged ones flat. Google products support only `equirectangular`. | [secondary] |
| **Finder / Quick Look / Preview**, **Explorer / Photos** | No evidence that either reads GPano; both show the flat 2:1 image. Third-party viewers exist for both OSes. | [secondary, no primary statement either way] |
| **Pannellum**, **Photo Sphere Viewer** | Both parse the GPano block of a JPEG (`FullPano*`, `CroppedArea*`, pose, initial view) and set the crop and the initial view from it; both have a switch to ignore it. | [primary, source] |
| **Respawwwn** | See §2.5: tiles cut with ImageMagick, so the block is never read for the tiled path; the flat fallback path reads it via Photo Sphere Viewer. | [primary, the human's repo] |

So: the block costs nothing, is required for Facebook and Google Photos, and is harmless elsewhere.

## 2. The preview in the webview

### 2.1 Engines

wry (0.57 in `Cargo.lock`) uses WebKit on macOS ("WebKit is native on macOS") and "WebView2 provided by Microsoft Edge Chromium" on Windows [primary, [wry README](https://github.com/tauri-apps/wry)]. WebView2 "uses Microsoft Edge as the rendering engine", Evergreen runtime, Windows 10 1709+ [primary, [MS](https://learn.microsoft.com/en-us/microsoft-edge/webview2/)].

**WebGL on macOS**: "WebKit now supports WebGL2 … the WebGL implementation now runs on top of Metal" since Safari 15 (macOS Catalina and later) [primary, [WebKit blog](https://webkit.org/blog/11989/new-webkit-features-in-safari-15/)]. WKWebView shares the system WebKit, so the macOS 14+ floor is comfortably inside that [estimate]. A WebKit regression report saw WebGL fail on GitHub's virtual macOS-13 runners (no Metal device) but work on macOS-14 ones [secondary]; a 2026 forum thread reports `linkProgram` failing in a *Catalyst* WKWebView on a 27.3 beta, which does not apply to an AppKit app [secondary]. **WebGL on Windows**: Chromium; works with a GPU; the SwiftShader software fallback is being removed (context creation fails instead from Chromium 139 / Edge 144 where it would have used SwiftShader), so a GPU-less VM gets **no WebGL** and the window must fall back to a flat image [secondary, Samsung/Edge policy pages]. The map already lists Windows as unverified.

**Texture limits**: `MAX_TEXTURE_SIZE` must be at least 64 in OpenGL ES 2.0 / WebGL 1 and at least 2048 in ES 3.0 / WebGL 2 [primary, [Khronos ES 2.0](https://registry.khronos.org/OpenGL-Refpages/es2.0/xhtml/glGet.xml), [ES 3.0](https://registry.khronos.org/OpenGL-Refpages/es3.0/html/glGet.xhtml)]. In practice Safari reports ≥ 8192 in 100 % and ≥ 16 384 in 99 % of surveyed sessions; Chromium on desktop GPUs is 16 384 [secondary, [web3dsurvey](https://web3dsurvey.com/webgl/parameters/MAX_TEXTURE_SIZE)]. A 10 000-wide texture therefore fits on every target machine, but costs 200 MB of GPU memory plus the 200 MB decoded bitmap in the webview process [estimate].

### 2.2 Candidate viewers

Sizes are the minified files on jsDelivr, measured with `curl` [primary]. All licences pass the `deny.toml` list.

| Viewer | Version | Licence | Size | Renderer | Loads one equirectangular image | Keyboard / drag | Reads GPano |
|---|---|---|---|---|---|---|---|
| **Pannellum** | 2.5.6 | MIT | 56 KB JS + 9.7 KB CSS, no deps | WebGL 1 only (`getContext('experimental-webgl')`); throws `{type: 'no webgl'}` for equirectangular images, the CSS 3D fallback exists only for multires and cubemaps. Splits an image wider than `MAX_TEXTURE_SIZE` into two textures; errors if `max(width/2, height)` still exceeds it. | `pannellum.viewer(el, {type: 'equirectangular', panorama: url, autoLoad: true, showControls: false})`; `setHfov`, `lookAt`, `on('load')`, `on('error')`, `destroy()` | Arrows and WASD pan, `+`/`-` zoom; `draggable`, `mouseZoom` | yes (`ignoreGPanoXMP`) |
| **Photo Sphere Viewer** `@photo-sphere-viewer/core` | 5.15.1 | MIT | 203 KB ESM + 12 KB CSS, depends on `three ^0.185` (392 KB `three.module.min.js`; a Vite build tree-shakes it [estimate]) | **WebGL 2 required** (`getContext('webgl2')`, else throws "WebGL 2 is not supported."). An image wider than `MAX_TEXTURE_SIZE` is downscaled on an `OffscreenCanvas` to `maxCanvasWidth`. | `new Viewer({container, panorama: url, navbar: false, keyboard: 'always'})`; `rotate`, `zoom`, `destroy()`, `panorama-error` event | Arrows rotate, PageUp/Down and `+`/`-` zoom; `keyboard` defaults to `'fullscreen'` so set `'always'`; `mousemove`, `mousewheel`, `moveInertia` | yes (`useXmpData`) |
| **Marzipano** | 0.10.2 | Apache-2.0 (deps bowser, gl-matrix, hammerjs, all MIT) | 216 KB | WebGL | Tile-oriented (`ImageUrlSource.fromString('{z}/{f}/{y}/{x}.jpg')`, `EquirectGeometry([{width}])`); single-image use is possible but the docs show only cubemaps | `mouseViewMode: 'drag'`; no keyboard controls documented | no |
| **three.js** directly | 0.187.0 | MIT | 392 KB minified module (less tree-shaken) | WebGL 2 (`WebGLRenderer`) | The official example is ~80 lines: `SphereGeometry(500, 60, 40).scale(-1, 1, 1)`, `TextureLoader`, `texture.colorSpace = SRGBColorSpace`, `MeshBasicMaterial({map})`, pointer deltas × 0.1 into lon/lat, wheel into `camera.fov` clamped 10–75 [primary, [example](https://github.com/mrdoob/three.js/blob/dev/examples/webgl_panorama_equirectangular.html)] | write it yourself | no; and no texture-size guard |
| Hand-written WebGL | — | — | ~0 KB, ~200 lines | WebGL 1 | A full-screen quad whose fragment shader maps view direction to (lon, lat) to UV; keyboard, drag, inertia and resize by hand | write it yourself | no |

All candidates run from the Vite bundle: no CDN, no network, no `script-src` change. Pannellum ships only a browser global (`build/pannellum.js`, no ESM or types), so it needs a side-effect import and a hand-written `.d.ts` [estimate from its package.json].

### 2.3 Content Security Policy

Current: `default-src 'self'; img-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:` (`app/tauri.conf.json`) [primary]. Tauri adds its own script hashes and style nonces at build time, so only app-specific sources are listed [primary, [Tauri CSP](https://v2.tauri.app/security/csp/)].

- **How the viewers fetch the image matters.** Pannellum requests it with an `XMLHttpRequest` (`responseType = 'blob'`), reads the XMP with a `FileReader`, then shows it via `URL.createObjectURL`; if the blob image is blocked it falls back to the plain URL [primary, source]. Photo Sphere Viewer's `loadTexture` does `textureLoader.loadFile(…)` → Blob → `blobToImage` [primary, source]. XHR and `fetch` are governed by **`connect-src`**, which falls back to `default-src 'self'`, so the file's origin must be added there, not only to `img-src`. three.js `TextureLoader` goes through an `<img>` element, so `img-src` alone suffices [secondary for the CSP rule, primary for the loaders].
- Tauri's documented example for the asset protocol is `img-src 'self' asset: http://asset.localhost blob: data:` [primary, [convertFileSrc](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc), [CSP page](https://v2.tauri.app/security/csp/)].

### 2.4 Getting the file into the webview

**(a) Asset protocol.** `convertFileSrc(path)` returns `asset://localhost/<encoded path>` on macOS and `http://asset.localhost/<encoded path>` on Windows (`https://` if `app.windows[].useHttpsScheme` is on, which also moves localStorage and cookies, so do not flip it) [primary, Tauri `scripts/core.js` and config reference]. Needs:
- the `protocol-asset` Cargo feature on `tauri` (the app has `features = []` today) [primary, docs.rs features; `Manager::asset_protocol_scope` is gated on it];
- `app.security.assetProtocol.enable = true` plus a `scope`. The scope is global, not per window. The Sessions folder is user-chosen (`choose_sessions_folder`), so a static glob cannot cover it: either `tauri-plugin-persisted-scope` with its `protocol-asset` feature, or widen it at runtime from Rust with `app.asset_protocol_scope().allow_file(&panorama)` / `allow_directory(folder, false)` [primary, [asset protocol scope](https://v2.tauri.app/security/asset-protocol/), [`Scope`](https://docs.rs/tauri/latest/tauri/scope/fs/struct.Scope.html)];
- `img-src` and `connect-src` extended with `asset: http://asset.localhost` (§2.3).
The handler checks the scope, sniffs the MIME from the first 8 KB, and answers a plain GET by reading the **whole file into a `Vec<u8>`** (fine at 2–16 MB); `Range` requests are served in chunks of at most 1000 KiB with 206 [primary, `crates/tauri/src/protocol/asset.rs`]. The webview can then cache and decode the file itself.

**(b) A command returning bytes.** The existing pattern is `session_thumbnail` → `data_url("image/jpeg", bytes)`: a base64 string through the JSON IPC (+33 %, one more copy in JS). For a 2 MB preview copy that is ~2.7 MB of string, acceptable; for a 16 MB full file it is not [estimate]. Tauri 2 can also return raw bytes (`tauri::ipc::Response`) to an `ArrayBuffer`, wrapped in a `Blob` and `URL.createObjectURL`; a `blob:` URL then needs `blob:` in `connect-src` for Pannellum and Photo Sphere Viewer (§2.3). Pannellum alone accepts a pre-decoded `Image`/`ImageBitmap` as `panorama`, which bypasses CSP entirely [primary, source]. No new permission, no scope, no Cargo feature.

**(c) A reduced copy for the preview.** Whatever the transport, the viewer should not decode 200 MB. Write a second JPEG next to the Panorama, **8192 × 4096** (134 MB decoded, 6.9 MB on disk at quality 85 in the Pillow run) or 4096 × 2048 (34 MB, ~1.7 MB) when the full file is wider, else the full file. In the 1200 × 760 main window a 60° view spans 1365 preview pixels at 8192 wide and 683 at 4096, so 8192 stays crisp and 4096 looks slightly soft [estimate]. Resizing costs about half a second in Pillow; comparable in Rust with `image::imageops::resize` + rayon [estimate]. Photo Sphere Viewer would downscale a too-wide image itself, but only against `MAX_TEXTURE_SIZE`, not against memory.

### 2.5 What Respawwwn expects

The site (https://respawwwn.com) is public, a Vite / React Router app; the fetched markup only lists `/assets/*.js` chunks and Google Tag Manager, so the viewer is invisible from outside [primary, `curl`]. The human's own checkout tells the rest [primary, `~/www/gameguessr/gameguessr-website/app/components/PanoramaImage.tsx`, `gameguessr-api/src/AssetResolver/Riddle/Panorama/*`, `MessageHandler/Asset/Panorama/*`, `config/packages/liip_imagine.yaml`]:
- Viewer: `@photo-sphere-viewer/core` **5.13.4** with `@photo-sphere-viewer/equirectangular-tiles-adapter`; options `navbar: false, minFov: 30, maxFov: 80, defaultZoomLvl: 20, moveSpeed 1 (1.5 on mobile), zoomSpeed: 2, mousemove: true, moveInertia: true`, an idle auto-rotation of 0.0001 rad per frame and an animated cursor hint until the first interaction.
- Tiles: the API cuts the uploaded equirectangular file with ImageMagick `convert <image> +repage -crop 16x8@ +repage tile_%03d.webp` (16 columns × 8 rows = 128 WebP tiles, numbered row-major) after a nadir watermark is composited; the front-end declares `width: cols × 512 = 8192` and `baseUrl: placeholder` (the thumbnail). The non-tiled fallback loads the Liip `webp` derivative (quality 95) or the `original` (quality 100).
- So Respawwwn expects a plain **2:1 equirectangular upload**; an 8192 × 4096 Panorama matches the 512-px tile grid exactly, larger files are cut into larger tiles and still work (the declared width only drives level choice) [estimate]. No metadata is read on the tiled path. Using Photo Sphere Viewer in Panowww's preview gives the same look and drag feel as the site.

## 3. Recommendation

**File.** One `Panorama` JPEG, 2:1, at the size the Stitch computes (PTGui's optimum for a 14 × 7 Session was 5234 × 2617), quality 90, written with the `image` encoder already in the tree; let the spike decide chroma subsampling. PNG only if the spec insists on lossless, at 4–7× the size and seconds of encode time. Embed the GPano block from §1.3 in an XMP APP1 segment right after the APP0 segment: hand-written packet, inserted with `img-parts` (PNG: `png::Encoder::add_itxt_chunk`). Fields: `ProjectionType`, `UsePanoramaViewer`, the four `CroppedArea*`, the two `FullPano*`, `InitialViewHeadingDegrees/Pitch/Roll`, `StitchingSoftware = "Panowww <version>"`, `SourcePhotosCount`, `FirstPhotoDate`, `LastPhotoDate`; no `PoseHeadingDegrees`. That is PTGui's set plus what Panowww knows, and it satisfies Facebook's and Google's required list. Also write a preview copy, 8192 px wide at most, quality 85, next to it.

**Viewer.** `@photo-sphere-viewer/core` (MIT, 203 KB + three.js, WebGL 2): the same viewer and options as Respawwwn, drag with inertia, keyboard with `keyboard: 'always'`, GPano-aware, and it clamps textures to `MAX_TEXTURE_SIZE` by itself. Pannellum (MIT, 56 KB, WebGL 1, no ESM build) is the lean alternative if three.js weight bothers; Marzipano and a hand-written shader do not earn their keep. Handle the no-WebGL case (Windows VMs, remote desktops) by catching the viewer's error and showing the preview copy flat with a short string from `strings.ts`.

**Transport.** Keep the existing command pattern for the first spike: a `panorama_preview` command returning the preview copy as a `data:` URL, exactly like `session_thumbnail`, needs no Cargo feature, scope, CSP change or new permission beyond `allow-panorama-preview`, and 2–7 MB of base64 is fine. Switch to the asset protocol (`protocol-asset` feature, `assetProtocol.enable`, `asset_protocol_scope().allow_file()` at Stitch end, `img-src` and `connect-src` extended with `asset: http://asset.localhost`) only if the preview grows past that or the window later becomes an editor that reloads the full file. Either way the Stitch window's label must join `windows` in `app/capabilities/default.json` (or get its own capability) before it can call commands.

## 4. Sources

- Google, [Photo Sphere XMP Metadata](https://developers.google.com/streetview/spherical-metadata).
- Adobe, [XMP Specification Part 3: Storage in Files](https://github.com/adobe/xmp-docs/raw/master/XMPSpecifications/XMPSpecificationPart3.pdf) (2020), §1.1.3 JPEG, §1.1.5 PNG.
- W3C, [PNG Specification (Third Edition)](https://www.w3.org/TR/png-3/), §11.3.3.1, §11.3.3.4.
- Facebook, [Graph API Photo reference](https://developers.facebook.com/docs/graph-api/reference/photo/), `spherical_metadata`, `allow_spherical_photo`.
- crates.io / docs.rs: [img-parts](https://docs.rs/img-parts/latest/img_parts/), [png Encoder](https://docs.rs/png/latest/png/struct.Encoder.html), [little_exif](https://docs.rs/little_exif/latest/little_exif/), [xmpkit](https://docs.rs/crate/xmpkit/latest), [xmp_toolkit](https://github.com/adobe/xmp-toolkit-rs), [xmp-writer](https://github.com/typst/xmp-writer).
- Tauri 2: [convertFileSrc](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc), [Asset protocol scope](https://v2.tauri.app/security/asset-protocol/), [Content Security Policy](https://v2.tauri.app/security/csp/), [config reference](https://v2.tauri.app/reference/config/) (`assetProtocol`, `useHttpsScheme`), [`Manager::asset_protocol_scope`](https://docs.rs/tauri/latest/tauri/trait.Manager.html), [`scope::fs::Scope`](https://docs.rs/tauri/latest/tauri/scope/fs/struct.Scope.html), source `crates/tauri/src/protocol/asset.rs` and `crates/tauri/scripts/core.js`; [wry README](https://github.com/tauri-apps/wry).
- Viewers: [Pannellum](https://pannellum.org/documentation/reference/) and its `src/js/pannellum.js`, `libpannellum.js`, `readme.md`; [Photo Sphere Viewer](https://photo-sphere-viewer.js.org/guide/config.html), [equirectangular adapter](https://photo-sphere-viewer.js.org/guide/adapters/equirectangular.html), source `packages/core/src/data/system.ts`, `utils/three-utils.ts`, `adapters/EquirectangularAdapter.ts`; [Marzipano](https://www.marzipano.net/docs.html) and its `package.json`; [three.js](https://github.com/mrdoob/three.js) `package.json` and `examples/webgl_panorama_equirectangular.html`. Sizes from jsDelivr.
- WebGL: [WebKit, New WebKit Features in Safari 15](https://webkit.org/blog/11989/new-webkit-features-in-safari-15/); Khronos ES 2.0 and ES 3.0 `glGet` reference pages; [web3dsurvey MAX_TEXTURE_SIZE](https://web3dsurvey.com/webgl/parameters/MAX_TEXTURE_SIZE); [Microsoft, Introduction to WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/); [WebView2 browser flags](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags); SwiftShader removal, Edge policy `EnableUnsafeSwiftShader` (secondary).
- Respawwwn: https://respawwwn.com (public, JS-rendered) and the human's checkout `~/www/gameguessr/` (website `app/components/PanoramaImage.tsx`, API `src/AssetResolver/Riddle/Panorama/`, `src/MessageHandler/Asset/Panorama/`, `config/packages/liip_imagine.yaml`).
- Local: `~/Pictures/Panowww/World of Warcraft/2026-11-07_23-14-46/r01_c01 Panorama.jpg` (PTGui Pro 13.10 output, inspected with `sips` and `exiftool`); Pillow 12 encode timings on an Apple M3 Max.
