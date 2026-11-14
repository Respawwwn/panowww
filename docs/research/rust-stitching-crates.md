# Research: Rust crates for each stitching stage

Planning ticket 002 *Rust crates for each stitching stage* (the Stitch map lives under `.wayfinder/stitch/`, not published).
Researched: 2026-11-13. Crate versions and dates are from the crates.io API on that date.

Words follow [GLOSSARY.md](../GLOSSARY.md): a **Session** is a grid of **Shots** laid out in **Rows**, each **Step** turning the camera by a nominal angle, neighbouring Shots sharing an **Overlap**. The stitch turns a Session into a **Panorama** (term to settle on the scope ticket).

How much to trust each claim:
- **[primary]**: crates.io API, docs.rs, the crate's own README or `Cargo.toml`, Debian copyright files, the vendor's docs. Read today.
- **[secondary]**: GitHub issues, search summaries, third-party benchmarks. Not confirmed by the owner.
- **[estimate]**: my own reasoning or extrapolation, not measured. Check it in the spike.

The workspace today: Rust 1.99.1, edition 2024, `image 0.25` (`core`, features `jpeg` only), `png 0.18` (`store`), `fast_image_resize 6` (`core`). All candidates below have an MSRV at or below 1.89 [primary, their `Cargo.toml`].

---

## 1. Image decoding and encoding

| Crate | Version | Licence | Last release | Pure Rust? | Notes |
|---|---|---|---|---|---|
| `png` | 0.18.1 | MIT OR Apache-2.0 | 2026-02-14 | Yes | Already in `store`. `Compression::Fast` uses `fdeflate`, "dramatically faster than the fastest mode of libpng" with a better ratio (README, no numbers) [primary]. Decoding is "on par with or better than libpng" [primary, README claim]. |
| `image` | 0.25.10 | MIT OR Apache-2.0 | 2026-03-10 | Yes | Already in `core`. `png` feature → `png 0.18`; `jpeg` feature → `zune-jpeg 0.5.5` decoder; `rayon` feature on by default (we turn defaults off) [primary, `Cargo.toml`]. Its own `JpegEncoder` is pure Rust, baseline only, 4:2:2 chroma subsampling, streams in 8×8 blocks so the whole Panorama need not be re-copied [primary, docs.rs]. MSRV 1.88. |
| `zune-image` / `zune-png` / `zune-jpeg` | 0.5.0 / 0.5.2 / 0.5.15 (0.5.16-rc3) | MIT OR Apache-2.0 OR Zlib | 2026-01-24 / 2026-03-11 / 2026-10-19 | Rust, with `unsafe` SIMD intrinsics only [primary, README] | `zune-png` **decodes only**, no encoder [primary, formats table]. Own benchmarks (Darwin, sizes unstated): baseline PNG decode `zune-png` 135 ms vs `image-rs/png` 149 ms; interlaced 140 vs 233 ms; 16-bit 186 vs 164 ms (png faster) [primary, `benchmarks/benchmark-results.md`]. Within ~10 % of `png` on the baseline case we have, so nothing to gain for Shot decoding. |
| `mozjpeg` (+ `mozjpeg-sys`) | 0.10.13 / 2.2.4-beta.1 | IJG / IJG AND Zlib AND BSD-3-Clause | 2025-02-18 | **No**: builds libjpeg-turbo/MozJPEG C via `cc`, needs `nasm` [primary, crates.io description] | Fastest baseline decoder in zune's own table (10–34 % ahead of `zune-jpeg`) [primary], but we decode PNG Shots, not JPEG. libjpeg errors surface as panics to catch [primary, README]. **IJG is not in `deny.toml`.** |
| `jpeg-encoder` | 0.7.1 | (MIT OR Apache-2.0) AND IJG | 2026-07-27 | Yes (AVX2 `simd` feature adds `unsafe`) | Baseline + progressive, optimised Huffman, no dependencies, MSRV 1.87 [primary, `Cargo.toml`, README]. No speed claims. **IJG is not in `deny.toml`.** |
| `fast_image_resize` | 6.1.0 | MIT OR Apache-2.0 | 2026-07-21 | Yes (SIMD under `unsafe`) | Already in `core`. Resize and crop only, no warps [primary, README]. SSE4.1/AVX2/NEON; `rayon` feature. 4928×3279→852×567 Lanczos3 RGB8: `image` 174 ms, fir AVX2 13 ms, single thread x86 [primary, README table]. Use it for the Display preview of the Panorama and for pyramid levels, not for reprojection. |

**Decode budget** [estimate]: a 1669×841 RGBA PNG Shot decodes in roughly 10–20 ms with `png` (`fdeflate`); 98 Shots on 8 threads with `rayon` is well under a second. Encoding a 12 000×6 000 Panorama: JPEG via `image::codecs::jpeg` ≈ 0.5–1 s; PNG `Compression::Fast` ≈ 1–2 s. Both are noise next to feature matching and blending.

## 2. Feature detection and matching

### 2.1 Crates

| Crate | Version | Licence | Last release | Pure Rust? | Notes |
|---|---|---|---|---|---|
| `akaze` (rust-cv) | 0.7.0 | MIT | **2020-06-23** | Yes | The crates.io release pins `image 0.23`, `ndarray 0.13`, `bitarray 0.2` [primary, `Cargo.toml`]: two `image` majors behind us. The monorepo `rust-cv/cv` `main` is ahead and **unpublished**: `image 0.25` (commit 2025-07-22), `ndarray 0.16`, `bitarray 0.9`, optional `rayon` (only `rayon::join` on the two Scharr derivatives), SIMD separable filters "25 %–400 %" faster (PR #68, 2023) [primary, repo]. Descriptor `BitArray<64>` (M-LDB 486 bits padded to 512) [primary, `lib.rs`]. Issue #81 "Is this project abandoned?" (2024-05): maintainers say "not very active", "at most kept up to date with new versions of dependencies" [secondary]. Using it means a `git` dependency, which **`deny.toml` forbids** (`unknown-git = "deny"`). |
| `cv-sift` | — | — | — | — | **Does not exist** on crates.io (404) nor under rust-cv (404) [primary]. rust-cv issue #7 "SIFT" open since 2020-07 [secondary]. |
| `sift-features` | 0.1.0 | MIT | 2024-09-10 | Yes | "Aims to be compatible" with OpenCV `feature2d` SIFT (Rey-Otero 2014 anatomy); deps `image 0.25.2`, `imageproc 0.25`, `ndarray 0.16`; no `rayon`; no benchmarks; dev-dep `opencv 0.92` for the comparison tests [primary, `Cargo.toml`, README]. One release, 3 808 downloads. |
| `lowe-sift` | 0.2.1 | MIT | 2026-05-29 | Yes | Detector, 128-D descriptor, exact and BBF kd-tree matching with Lowe ratio, Hough + affine verification. The README calls it "a readable reference-style implementation, not a SIMD-optimized extractor" [primary]. No threading mentioned. |
| `pure-sift` | 0.1.0 | MIT OR Apache-2.0 | **2026-11-12** | Yes, `#![forbid(unsafe_code)]` | Published yesterday, 10 downloads. SIFT + Lowe ratio + similarity/homography RANSAC + feather-blended stitching; `rayon` on descriptor extraction; deps `image 0.25`, `ndarray 0.16`, `rayon`, optional `axum`/`tokio` behind the default `server` feature (turn it off) [primary, `Cargo.toml`, README]. No timings. Inlier counts on Oxford pairs only. |
| `sift-wgpu` | 0.1.0 | ? | 2026-01-19 | — | SIFT with CPU and WebGPU backends [primary, crates.io description]. Not read further; one release. |
| `imageproc` | 0.27.0 | MIT | 2026-06-02 | Yes | `corners::corners_fast9/12`, **`corners::oriented_fast`** (intensity-centroid orientation, Rublee 2012), `binary_descriptors::brief` (Calonder 2010), `match_binary_descriptors` (brute-force Hamming under a threshold) [primary, docs.rs]. No steered (rotated) BRIEF documented, so it is "FAST + BRIEF", not a full ORB. Also `geometric_transformations::warp_with` (see §4). Optional `rayon`. |
| `tpt-cv-feature` | 0.1.0 | MIT OR Apache-2.0 | 2026-08-25 | Yes, "zero FFI" | FAST-9 with NMS, **full ORB** (intensity centroid + rotated BRIEF, 256 bits as `[u64; 4]`), brute-force Hamming with Lowe ratio, LSH matcher, pyramidal LK; `no_std` [primary, docs.rs README]. 172 downloads, one release, no benchmarks. Depends only on `tpt-cv-core`. |
| `purecv` | 0.12.0 | **LGPL-3.0-or-later** | 2026-11-02 | Rust, `unsafe` for SIMD (Miri-checked) | ORB, `BFMatcher` (Hamming, cross-check, knn), `find_homography` RANSAC, `remap`, `warp_perspective`, pyramids; `rayon` + `pulp` SIMD. `orb_detect` 512×512: 117 ms plain, 31 ms parallel+SIMD [primary, README]. "Work in progress, not yet stable". **Licence fails `deny.toml`.** |
| `hamming` | 0.1.3 | MIT/Apache-2.0 | **2015-06-01** | Yes | Popcount over byte slices. Superseded by `bitarray`. |
| `bitarray` | 0.10.0 | MIT | 2021-08-15 | Yes | Fixed-size bit arrays with Hamming distance (popcnt, optional 512-bit SIMD), implements `space::MetricPoint` [primary, space README]. `akaze` main pins 0.9. |
| `arrsac` + `sample-consensus` | 0.11.0 / 1.0.2 | MIT / MIT | 2025-08-03 / 2021-07-20 | Yes | ARRSAC with SPRT, "almost always faster than RANSAC given tuning" [primary, README]. `sample-consensus` is the `Consensus` / `Estimator` / `Model` trait crate. `arrsac` 0.11 uses `rand 0.9`. |
| `space` | 0.19.0 | MIT | 2025-08-04 | Yes | Traits `Metric`, `Knn`, `LinearSearch` / `LinearContainer` [primary, README]. `hnsw` 0.11.0 (2021-07) MIT implements ANN over it. |
| `kiddo` | 6.3.0 | MIT OR Apache-2.0 | 2026-09-06 | Yes | kd-tree for low-dimensional float points. Not for 128-D SIFT or 256-bit Hamming [estimate]. |
| `kornia-imgproc` | 0.2.0 | Apache-2.0 | 2026-09-26 | Rust; `turbojpeg` feature needs libjpeg-turbo + `nasm` | Resize, crop, rotate, colour; FAST detector in examples; no warp/remap listed in the README [primary]. CUDA-oriented roadmap. |

### 2.2 Patents

- **SIFT**: US 6,711,293 (D. G. Lowe, assignee University of British Columbia) "expired as of March 7, 2020" [primary, Lowe's page cs.ubc.ca/~lowe/keypoints]. OpenCV 4.4.0 moved SIFT into the main repository for that reason [secondary, release-note summary; the opencv.org page returned 403 today].
- **AKAZE**: the reference code is BSD-3-Clause and "can be used freely, including commercial applications" [primary, pablofdezalc/akaze README]. No patent.
- **ORB / FAST / BRIEF**: published 2011 / 2006 / 2010, in OpenCV's main module since 2.3; no patent known [secondary]. SURF is the patented one; no Rust crate offers it anyway.

### 2.3 Speed

No Rust crate above publishes a timing. What exists:
- Reference C++ A-KAZE README, Oxford "boat" pair, hardware unstated, single runs: A-KAZE ≈ 292 ms, ORB ≈ 273 ms, BRISK ≈ 188 ms per image [primary, README]. The author: A-KAZE "a bit slower than ORB and BRISK" but "much better performance".
- `purecv` ORB 512×512: 117 ms single thread, 31 ms parallel + SIMD [primary]. Scaled to 1700×850 (5.5× the pixels): roughly 0.2–0.6 s single-threaded [estimate].
- rust-cv `akaze`: no number anywhere; before PR #68 users were told to run `--release` or it is "quite slow" [secondary]. Expect 0.5–2 s per 1700×850 Shot single-threaded [estimate].
- Pure-Rust SIFT without SIMD (`lowe-sift`, `sift-features`, `pure-sift`): 1–3 s per Shot [estimate].

**What the lever buys** [estimate]: a general Stitcher matches every pair (98 Shots → 4 753 pairs) and needs scale and rotation invariance. Panowww knows the grid: each Shot overlaps only its 4 neighbours (about 200 pairs), the nominal yaw/pitch Step gives the expected offset of each neighbour to within a few percent of the Overlap, and roll ≈ 0 except on the two pole Rows. So FAST + plain BRIEF, matched brute-force in a search window around the nominal offset, is enough off the poles; the pole Rows (every Shot looks at the same spot and rotates about the view axis) need the rotated descriptor (`oriented_fast` + ORB) or no features at all: their rotation is known from the Step. Detection on ~100 Shots in parallel on 8 cores should land under 2 s total with FAST/BRIEF, under 10 s with a pure-Rust SIFT. Measure this first in the spike: it decides whether SIFT is needed.

## 3. Optimisation

| Crate | Version | Licence | Last release | Pure Rust? | Notes |
|---|---|---|---|---|---|
| `levenberg-marquardt` | 0.15.0 | MIT | 2025-08-03 | Yes | MINPACK `lmder` port, "exact LM", NaN/inf handling; edition 2024; depends on **`nalgebra 0.34`** [primary, `Cargo.toml`]. You implement `LeastSquaresProblem { set_params, params, residuals, jacobian }` with dense nalgebra storage; no sparse Jacobian, no numeric differentiation helper [primary, docs.rs]. Unit tests match MINPACK at floating-point level [primary, README]. |
| `nalgebra` | 0.35.0 | Apache-2.0 | 2026-05-24 | Yes | `Rotation3`, `UnitQuaternion`, `so(3)` exp/log maps for the per-Shot rotation. Pin **0.34** workspace-wide to share one copy with `levenberg-marquardt`, or accept a second copy (`deny.toml` has `multiple-versions = "warn"`). |
| `argmin` | 0.11.0 | MIT OR Apache-2.0 | 2025-09-28 | Yes | Gauss-Newton, trust region, quasi-Newton, Nelder-Mead… **no Levenberg-Marquardt solver** [primary, docs.rs solver list]. Heavier framework; skip. |
| `cv-core` / `cv-pinhole` | 0.15.0 / 0.6.0 | MIT / MIT | **2020-06-23** | Yes | Camera and pose traits, `CameraIntrinsicsK1Distortion`; pinned to `nalgebra 0.30` [primary, `Cargo.toml`]. Three nalgebra majors behind; not worth the conversion glue. |
| `faer` | 0.24.4 | MIT | 2026-06-24 | Yes | Fast dense linear algebra; only needed if the normal equations get large. Not for the spike. |

**Problem size** [estimate]: per Shot one rotation (3 parameters), plus one horizontal field of view and maybe one radial `k1` per Session: 98 Shots → ~300 unknowns. With ~200 neighbour pairs × 30–100 matches × 2 residuals ≈ 10–40 k residuals, the dense Jacobian is 40 k × 300 `f64` ≈ 100 MB at the top end; fine for `levenberg-marquardt`, and the nominal grid is such a good initial guess that a handful of iterations converge. A 36 × 36 Session (1 296 Shots, ~3 900 unknowns, ~500 k residuals) would need a sparse or Schur-complement solver: out of scope, cap the Session size for the experimental Stitch or solve Row by Row [estimate].

## 4. Reprojection, seams and blending

Searched crates.io for "equirectangular", "panorama", "stitching", "laplacian pyramid blending", and the web for multi-band blending in Rust [primary, crates.io search API]. Nothing covers a pinhole grid → equirectangular Panorama:

| Crate | Version | Licence | Last release | Pure Rust? | Notes |
|---|---|---|---|---|---|
| `imageproc::geometric_transformations` | 0.27.0 | MIT | 2026-06-02 | Yes | `warp_with(image, mapping: Fn(f32, f32) -> (f32, f32) + Sync + Send, Interpolation, Border)` defines the pre-image of each output pixel with a closure [primary, docs.rs]: exactly an inverse equirectangular map (output pixel → yaw/pitch → ray → Shot pixel). Nearest/bilinear/bicubic. `rayon` optional. Whether it is as fast as a hand loop is for the spike. |
| `imgpyr` | 0.2.1 | MIT | 2026-07-31 | Yes | Gaussian and Laplacian pyramids on `f32` planes with OpenCV `pyrDown`/`pyrUp` conventions, `collapse`, odd sizes, `rayon` feature; **no blending API**, 55 downloads [primary, README]. A multi-band blend is ~100 lines on top of it [estimate]. |
| `insta360-stitch` | 0.0.3 | MIT OR Apache-2.0 | 2026-10-29 | Yes (ffmpeg for video only) | Dual-fisheye only, linear ramp blending, DIS flow for parallax. Useful as a speed reference: an 11 904 × 5 952 photo in 3.6 s on 24 threads, 25 s on one, 230 MB [primary, README]. |
| `opencv-no-c-stitching` | 0.1.0 | Apache-2.0 | 2026-10-20 | Yes | "Pure-Rust port of OpenCV 4.15" with warpers, blenders, panorama pipeline; 2 stars, 3 commits; README admits `features2d` detectors and `VoronoiSeamFinder` are numerically unverified [primary, README]. Reads as machine-translated. Do not depend on it; maybe read it. |
| `oximedia-align` / `oximedia-cv` | 0.2.1 | ? | 2026-08-12 | Yes | Linear alpha blending only [secondary, source search]. |
| `rayon` | 1.12.0 | MIT OR Apache-2.0 | 2026-04-14 | Yes | Row-parallel remap and pyramid passes. |
| `ndarray` | 0.17.2 | MIT OR Apache-2.0 | 2026-01-10 | Yes | Optional; `image::ImageBuffer` + slices + `rayon` is enough for the spike [estimate]. `pure-sift` and `sift-features` pin 0.16. |
| `wgpu` | 30.0.2 | MIT OR Apache-2.0 | 2026-11-12 | Yes | Headless compute for the remap and the pyramid later. Releases every 1–3 months (29.0.4 Jul, 30.0.1 Aug, 30.0.2 Nov 2026) [primary]. Not needed if the CPU path meets the PTGui target. |

So reprojection, seams and blending are **hand-written on `image` + `rayon`**:
- Remap: inverse map per output pixel, bilinear, writing each Shot's warped tile plus a weight mask. 12 000 × 6 000 output ≈ 72 M pixels; the whole Panorama once over at ~50–100 Mpx/s per core → under 1 s on 8 cores [estimate].
- Seams: the in-game camera rotates about a fixed point, so there is no parallax and the Overlap matches once aligned. Voronoi (nearest-Shot-centre) seams are enough; a graph-cut seam finder is a later refinement [estimate].
- Blend: 4–6-level Laplacian pyramid multi-band blend (Burt–Adelson), the standard that `enblend` and OpenCV `MultiBandBlender` use. Memory: full-size `f32` RGB pyramids of the Panorama ≈ 72 M × 3 × 4 B × 1.33 ≈ 1.1 GB; blend per band in tiles or in `f16`/`u16` to stay under a few hundred MB [estimate].

## 5. The non-pure options

### 5.1 The `opencv` crate

`opencv` 0.102.0, MIT, 2026-11-08, OpenCV 4.x and 5.x, MSRV 1.88 [primary]. Bindings are pre-generated, but the build still needs **libclang/LLVM** and a system OpenCV found through pkg-config, CMake or vcpkg [primary, README/INSTALL]. The `stitching` module is bound: `Stitcher`, `Detail_SphericalWarper`, `Detail_BundleAdjusterRay`, `Detail_GraphCutSeamFinder`, `Detail_MultiBandBlender` [primary, docs.rs].

- **macOS**: the documented route is `brew install opencv`, dynamic dylibs; the formula pulls ~25 runtime dependencies (ffmpeg, tbb, openvino, vtk, tesseract, protobuf…) [primary, formulae.brew.sh]. Shipping that inside a Tauri bundle means relinking and bundling the whole tree, or a custom static OpenCV build with only `core`, `imgproc`, `features2d`, `calib3d`, `stitching`.
- **Windows**: Chocolatey (dynamic) or vcpkg; the crate's own CI uses the `x64-windows` (dynamic) triplet [primary, `ci/install-windows-vcpkg.sh`]. A static build is `x64-windows-static-md` + `OPENCV_MSVC_CRT` [primary, README env vars].
- **Static linking** is "supported and tested at least on Linux"; the crate's CI runs a static job only on Ubuntu [primary, INSTALL.md, workflow]. macOS and Windows static builds would be ours to debug.
- **Size**: the `opencv-python-headless` 5.0.0.93 wheel, which embeds a full OpenCV, is 48.3 MB on macOS arm64 and 43.8 MB on win_amd64 [primary, PyPI]. A trimmed static build is 15–30 MB on top of the app [estimate].
- **CI cost**: the crate's CI caches `~/build` and uses `sccache`; the old Travis script noted >50 min without a cached OpenCV [secondary]. Expect 20–40 min per OS the first time, minutes after, plus two new toolchain installs (LLVM, vcpkg/brew) in `ci.yml` [estimate].
- It is `unsafe` FFI by nature, which the new crate's `#![forbid(unsafe_code)]` rule excludes from `core`-style crates; it would need an adapter crate like `input` / `capture`.

### 5.2 Hugin's `nona` / `enblend` / `libpano13`

| Tool | Licence | Source |
|---|---|---|
| hugin (incl. `nona`) | GPL-2+ ("`doc/nona.pod`: licensed under the same terms as the hugin package itself") | [primary, Debian copyright file] |
| libpano13 | GPL-2+ (plus BSD/MIT/MINPACK-licensed files; `lmdif.c` is MINPACK) | [primary, Debian copyright file; SourceForge `COPYING`] |
| enblend-enfuse | GPL-2+ | [primary, SourceForge `COPYING`; Debian copyright file] |

An MIT app may not **link** GPL code without becoming GPL. Shipping the GPL executables beside the app and calling them with `exec` + command-line arguments is "mere aggregation" in the FSF's reading, as long as the two stay separate programs (pipes, sockets and command-line arguments are the mechanisms "normally used between two separate programs") [secondary: GNU GPL FAQ "MereAggregation"; gnu.org rate-limited me today, quoted from memory, verify before relying on it]. Even then Panowww would be distributing GPL binaries: it must ship their licence texts and the corresponding source or a written offer, on both OSes, and the Hugin tree weighs more than the app. The map's rule is "no GPL", and *Open in PTGui / Autopano Giga* already covers the external-Stitcher path. Not a fallback.

## 6. Licence check against `deny.toml`

Allowed today: Apache-2.0 (± LLVM exception), BSD-2/3, BSL-1.0, CC0, CDLA-Permissive-2.0, ISC, MIT, MPL-2.0, OFL-1.1, Unicode-3.0, Zlib. Also `unknown-git = "deny"`, `wildcards = "deny"`, `multiple-versions = "warn"`.

| Candidate | Licence expression | Passes? |
|---|---|---|
| `png`, `image`, `fast_image_resize`, `rayon`, `ndarray`, `wgpu`, `kiddo`, `tpt-cv-feature`, `pure-sift` | MIT OR Apache-2.0 | Yes |
| `zune-*` | MIT OR Apache-2.0 OR Zlib | Yes |
| `imageproc`, `akaze`, `arrsac`, `sample-consensus`, `space`, `bitarray`, `levenberg-marquardt`, `sift-features`, `lowe-sift`, `imgpyr`, `faer`, `opencv` (bindings) | MIT | Yes |
| `nalgebra`, `kornia-*`, `opencv-no-c-*` | Apache-2.0 | Yes |
| `jpeg-encoder` | (MIT OR Apache-2.0) **AND IJG** | **No** until `IJG` is added to `allow` (it is a permissive notice-licence; adding it is defensible, but needs a reason in `deny.toml`) |
| `mozjpeg` / `mozjpeg-sys` | IJG / IJG AND Zlib AND BSD-3-Clause | **No** (IJG), and not pure Rust |
| `purecv` | LGPL-3.0-or-later | **No** |
| rust-cv `akaze` from the monorepo | MIT, but a `git` source | **No**: `unknown-git = "deny"`; the published 0.7.0 is unusable with `image 0.25` |
| Hugin tools | GPL-2+ | **No** (not a crate, but the same rule) |
| OpenCV library itself | Apache-2.0 | Not seen by cargo-deny; fine on paper |

Everything on the pure-Rust shortlist in §7 passes `deny.toml` unchanged.

## 7. Recommendation

Shortlist for the pure-Rust spike (one throwaway binary in the workspace, `#![forbid(unsafe_code)]` where the map rule applies):

1. **Decode / encode**: `png 0.18` (decode the Shots; encode with `Compression::Fast` if the Panorama is PNG), `image 0.25` with `png` + `jpeg` features and `image::codecs::jpeg::JpegEncoder` for a JPEG Panorama, `fast_image_resize 6` for the preview. Nothing new. Decode all Shots in a `rayon` pool.
2. **Features**: `imageproc 0.27` `corners_fast9` + `binary_descriptors::brief`, brute-force Hamming limited to the 4 grid neighbours inside a window around the nominal Step offset (no kd-tree, no `space`). On the pole Rows use `oriented_fast` + the ORB descriptor of `tpt-cv-feature 0.1`, or skip features and trust the Step. Outliers: a tiny RANSAC on a 2-parameter offset model, or `arrsac 0.11` + `sample-consensus 1.0` if a full rotation model is fitted per pair. **Fallback inside the spike**: `sift-features 0.1` or `pure-sift 0.1` (both pure Rust, MIT-compatible) if binary descriptors fail on repetitive game textures; expect several seconds more.
3. **Optimisation**: `levenberg-marquardt 0.15` with `nalgebra 0.34` (pin 0.34 to keep one copy). Unknowns: one rotation per Shot (so(3) tangent around the nominal grid pose) + one horizontal field of view; residuals = reprojection error of each match on the unit sphere. Analytic or finite-difference Jacobian, dense.
4. **Reprojection, seams, blend**: hand-written. Inverse equirectangular map per Shot with `imageproc::warp_with` or a `rayon` row loop, Voronoi seams from the grid, multi-band blend on `imgpyr 0.2` pyramids (or ~100 lines of own `pyrDown`/`pyrUp`). Keep the blend tiled or 16-bit to hold memory. `wgpu 30` is the GPU path if the CPU remap + blend misses the PTGui target.
5. **Fallback outside pure Rust**: the `opencv 0.102` crate statically linked, in a separate adapter crate, only if the spike cannot reach PTGui-class time or quality. Costs: LLVM + OpenCV on both CI runners, ~20–40 MB more per installer, `unsafe` FFI, and the macOS/Windows static link is untested upstream. Hugin is not a fallback (GPL).

Blockers found: `cv-sift` does not exist; rust-cv is dormant and its usable `akaze` is git-only (blocked by `deny.toml`); every fast JPEG encoder outside `image` carries the IJG term; no crate does equirectangular remapping or multi-band blending, so stage 4 is ours to write; no Rust feature crate publishes a timing, so the spike must measure detection first.
