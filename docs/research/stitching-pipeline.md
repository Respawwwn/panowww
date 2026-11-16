# Research: How a panorama stitcher works and why PTGui is fast

Planning ticket 001 *How a panorama stitcher works and why PTGui is fast* (map `.wayfinder/stitch`, not published).
Researched: 2026-11-13.

Words follow [GLOSSARY.md](../GLOSSARY.md): a **Session** is one capture run, a **Shot** one screenshot of the Capture zone, a **Row** the Shots at one pitch over a full turn, a **Step** one camera movement between two Shots, **Overlap** the share two neighbouring Shots have in common, a **Stitcher** the program that assembles the Shots. The map adds **Stitch** (the run inside Panowww) and **Panorama** (its output).

How much to trust each claim:
- **[primary]**: a paper, a vendor page, a manual or source code that I read (PTGui's site, Hugin's manual, OpenCV's `modules/stitching` source, Brown & Lowe 2007, Szeliski 2006, Lowe 2004, the `.pts` project PTGui saved in the first World of Warcraft Session).
- **[secondary]**: forum threads, mailing lists. Several reports agree but the vendor did not say it.
- **[estimate]**: my own reasoning or arithmetic, not measured. Check it in the spike.

Burt & Adelson 1983 could not be fetched (MIT host has an expired certificate, the Brown mirror 404s); its claims come through Szeliski's tutorial and OpenCV's implementation and are marked [secondary].

---

## 1. Facts

### 1.1 The stages of a spherical stitch and what each costs

Every general-purpose Stitcher runs the same chain. OpenCV's `Stitcher` (PANORAMA mode) is the public reference: `matchImages()` → `estimateCameraParams()` → `composePanorama()`, with features found at a **0.6 Mpx** work resolution, seams at **0.1 Mpx**, compositing at the original resolution, and a pair kept only when its confidence exceeds **1.0** [primary, `stitcher.cpp`]. Brown & Lowe's AutoStitch algorithm box is the same list [primary].

| Stage | What it does | Cost driver | Reference numbers |
|---|---|---|---|
| **Feature detection** | Scale- and rotation-invariant keypoints per image. SIFT in Brown & Lowe; a Hessian box-filter detector (Panomatic, SURF-style) in Hugin's `cpfind`; ORB by default in OpenCV. | Linear in pixels. Brown & Lowe: "Once features have been extracted from all n images (linear time)" [primary]. | "A typical image of size 500x500 pixels will give rise to about 2000 stable features" (Lowe 2004) [primary]. `cpfind` halves the image first ("scaled to their half width and height") and caps keypoints at `sieve1width × sieve1height × sieve1size` = 10 × 10 × 100 = **10 000** per image [primary]. |
| **Feature matching** | Each descriptor to its k nearest neighbours across all images (k-d tree, k = 4 in Brown & Lowe), ratio test (0.8 in Lowe; `cpfind --kdtreeseconddist 0.25`, `--kdtreesteps 200`; OpenCV `m0.distance < (1 − match_conf) × m1.distance`). | O(n log n) with a k-d tree over all features [primary, Brown & Lowe]. Without a prior on which images touch, every image is a candidate for every other. | Brown & Lowe keep the **m = 6** images with the most matches as candidates per image [primary]. `cpfind` keeps **4 to 25 control points per pair** (`--sieve2*` defaults) [primary]. |
| **Pairwise geometry (RANSAC)** | Fit a homography to 4 random matches, count inliers, repeat. | Trials × matches. Brown & Lowe: r = 4, **n = 500 trials**; `cpfind`: `--ransaciter 1000`, `--ransacdist 25` px, `--ransacmode auto` = homography below 65° hfov and roll/pitch/yaw above [primary]. | Brown & Lowe verify a pair with `n_i > α + β·n_f` (α = 8, β = 0.3); OpenCV copies it as `confidence = num_inliers / (8 + 0.3 × matches)` [primary, `matchers.cpp`]. |
| **Initial camera guess** | Focal from the homographies (median over pairs), rotations chained along a maximum spanning tree weighted by inlier count. | Negligible. | OpenCV `HomographyBasedEstimator`: `R = K_from⁻¹ · H⁻¹ · K_to`, chained `R_to = R_from · R`; fallback focal = mean of (width + height) [primary, `motion_estimators.cpp`, `autocalib.cpp`]. |
| **Bundle adjustment** | Levenberg-Marquardt over all cameras at once. | Each iteration O(N³) in the number of parameters N = 4 × images; M (measurements) ≈ 100 N [primary, Brown & Lowe §4.1]. | 57 images → "a 4 × 57 = 228 parameter optimisation problem" [primary]. PTGui's optimizer "adjusts your project in such a way that the lowest control point distance is obtained" and may report 31.5 mm for a 28 mm lens [primary, FAQ 3.6]. |
| **Straightening (wave correction)** | Post-multiply all rotations by a global rotation so camera x-axes lie in one plane. | Negligible: smallest eigenvector of Σ r_k0 r_k0ᵀ [primary, Szeliski §5.1; OpenCV `waveCorrect` uses the same scatter matrix]. | |
| **Reprojection (remapping)** | For every output pixel: equirectangular → ray → rotate → rectilinear pixel → interpolate. | Output pixels × images overlapping each pixel. "stitching time depends mostly on the output size of the panorama" [primary, PTGui FAQ 3.22]. Hugin's `nona` default interpolator is Poly3 (bicubic) [primary]. | Brown & Lowe on a 1.6 GHz Pentium M: 57 images of 2272 × 1704 "matched and registered in 60 seconds, and a further 15 minutes were taken to render the 8908×2552 (23 megapixel) output"; a 2000 × 573 preview in 57 s [primary]. |
| **Seam finding** | Choose, per output pixel, which image wins. Voronoi (nearest centre), or a graph cut on colour differences (OpenCV `GraphCutSeamFinder` at 0.1 Mpx, `enblend --primary-seam-generator graph-cut` default since 4.2, `--coarse-mask` factor 8, Dijkstra radius 25 px, annealing) [primary]. | Overlap pixels at the seam resolution. PTGui: "Calculating the best location of a seam among millions of pixels takes considerable processing time", it is sequential and **not GPU accelerated** [primary, FAQ 3.31, 3.11]. | |
| **Blending** | Multi-band (Laplacian pyramid) blend across the seams; or PTGui 12's "Zero Overlap" engine that uses only the area inside the seams. | Output pixels × levels, plus pyramid memory. | OpenCV: `num_bands = ceil(log2(blend_width)) − 1`, `blend_width = sqrt(area) × 5 / 100`, capped at `log2(max side)`; Brown & Lowe render with **5 bands, σ = 5 px**; `enblend --levels` 1 to 29, "auto" picks the maximum [primary]. |
| **Exposure / gain** | Per-image gain or block gains so overlaps agree. | Small. | Brown & Lowe gain compensation (σ_N = 10, σ_g = 0.1); OpenCV `BlocksGainCompensator` [primary]. Not needed for a game (map: out of scope). |

### 1.2 The rectilinear camera model

A pinhole camera maps a ray `(X, Y, Z)` to `x = f·X/Z, y = f·Y/Z`, with `K = diag(f, f, 1)` and `f⁻¹ = tan(θ/2)`, i.e. `θ = 2·atan(1/f)` for a unit-less focal length (half-width = 1) [primary, Szeliski eq. 9–12]. In pixels: `f_px = (w/2) / tan(hfov/2)` and `hfov = 2·atan(w / (2 f_px))`; the PanoTools wiki gives the same for sensor sizes and `vfov = 2·atan(tan(hfov/2) × h / w)` [primary].

Two rectilinear images of a rotating camera are related by the homography `H_ij = K_i R_i R_jᵀ K_j⁻¹` [primary, Brown & Lowe eq. 1; Szeliski eq. 143]. Rotations are stored as a rotation vector θ with `R = exp([θ]×)` (Brown & Lowe) or Rodrigues (OpenCV).

What the two Stitchers add on top of `f` and `R`:

- **Hugin / PanoTools lens model** [primary, Hugin manual *Lens correction model*, PTStitcher o-line]: projection `f` (0 = rectilinear, 2 = circular fisheye, 3 = full-frame fisheye, 4 = equirectangular), hfov `v`, `y p r` yaw/pitch/roll, radial polynomial `r_src = (a·r_dest³ + b·r_dest² + c·r_dest + d)·r_dest` with `d = 1 − (a + b + c)` and radius 1.0 = half the smaller side, shift `d e` in pixels, shear `g t`. "A perfect lens has a = b = c = 0.0 and d = 1.0."
- **PTGui ≥ 11** [primary, *What's new in 11*, the saved `.pts`]: focal length + sensor diagonal replace hfov ("Focal length replaces horizontal field of view as the stored lens value"); shift and crop "are normalized to the diagonal size of the image" as long-side / short-side offsets; `a b c`; shear; a fisheye factor. Per image: `yaw pitch roll` and a viewpoint correction (`vpx vpy vpd vppan vptilt`) for images not taken from the no-parallax point. The project PTGui Pro 13.10 saved for *World of Warcraft/2026-11-07_23-14-46* starts from `projection: rectilinear, focallength 50, sensordiagonal 43.27` (there is no EXIF in a PNG) with `optimizelens: "heavyplusshift"`, i.e. fov + a + b + c + shift all optimised; image 0 is the anchor (yaw/pitch/roll not optimised), every other Shot has all three free; `viewrotate: "auto"`; `blend.engine: "zerooverlap"`, `seamfinding: true`; output "relative, fraction 1" (optimum size), JPEG 95.

**What a game camera needs** [estimate, from the model above]: `a = b = c = 0`, shift 0, shear 0, no vignetting, no response curve, no viewpoint (the camera orbits one point). Parameters left: **one hfov for the Session** and **yaw, pitch, roll per Shot**. The one thing to verify in the spike is that the game really renders a centred pinhole projection (principal point at the zone centre) and that the Capture zone is not cropped asymmetrically; if it is, a fixed principal point offset (`ppx, ppy` in OpenCV's `CameraParams`) is the only extra parameter.

**What PTGui found for the World of Warcraft Session** [estimate from primary data]: the Panorama is 5234 × 2617 for 1669 × 841 Shots. PTGui's optimum size keeps the centre of the source images at 1:1, which for an equirectangular output means `W = 2π f_px`, so `f_px ≈ 833` and `hfov = 2·atan(834.5 / 833) ≈ 90°`, `vfov ≈ 54°`. Its starting guess (50 mm on a 43.27 mm diagonal) was only `2·atan((1669/1869) × 21.6 / 50) ≈ 42°`: the optimizer had to move the fov by a factor of two, which a Stitcher does only through control points.

### 1.3 What PTGui runs on the GPU, and published timings

- PTGui 10 (2014): "A new blazing fast, GPU accelerated stitching engine. Up to 10 times faster stitching!"; "For GPU processing the stitcher uses OpenCL"; "moving data between the CPU and the GPU is relatively slow and should be avoided"; CPU path "improved as well, especially on computers with 4 or more CPU cores" [primary, *What's new in 10*, version history].
- Published timing: the Gigapixel Panorama Speedtest "stitches in 22 seconds, compared to 151 seconds in PTGui 9.2" on a "6 core Intel i7, Windows 8.1, 32GB of RAM, NVidia Geforce Titan, Samsung SSD" [primary]. 11.19: "Performance improvements when using GPU acceleration (40% faster for certain projects on certain hardware)" [primary].
- What is *not* on the GPU: "the 'zero offset' blending algorithm and automatic seam placement are not GPU accelerated" (FAQ 3.11); optimum seam placement "runs sequentially" (FAQ 3.31); "Stitching and blending requires a lot of disk and memory access" (FAQ 3.15) [primary]. By elimination the GPU work is the remapping, the multiband blending and the photometric passes (the 12.23 fix "banding artifacts … if multiband blending was used and GPU acceleration enabled" confirms blending is on the GPU) [primary for the quotes, estimate for the elimination].
- PTGui 11 added OpenCL rendering for the Panorama Editor and Detail Viewer, a control-point finder "rewritten from scratch", and "generate control points only for overlapping images" when images are pre-positioned by Align to Grid, a template or a PapyWizard XML [primary]. PTGui 12 added the Zero Overlap blender ("requires no overlap at all") and automatic seam placement that puts seams "in those areas where there is the least difference between the overlapping images" [primary].
- GPU requirements: OpenCL 1.1, ≥ 512 MB VRAM, one device only ("PTGui will only use one OpenCL device"); Apple silicon supported; the FAQ warns Apple is deprecating OpenCL [primary, FAQ 8.1–8.3, 3.11].
- FAQ 3.22's order of levers: output size first, then disable optimum seams, 8-bit output, RAM limit, then "an OpenCL-capable GPU" [primary].
- Hugin: `nona` "supports GPU stitching with graphics hardware" [primary]; it uses OpenGL shaders, is experimental, crashes on some driver combinations and falls back to the CPU; speed-ups are inconsistent [secondary, hugin-ptx threads, Launchpad bug 1362170]. `enblend` stays on the CPU.

### 1.4 Pole Rows and the pole singularity

- **Equirectangular**: "the horizontal coordinate is simply longitude, and the vertical coordinate is simply latitude"; zenith and nadir "are stretched to the entire width of the image" and the polar regions are "extremely distorted" [primary, PanoTools wiki]. Brown & Lowe: "the θ, φ parameterisation suffers from singularities at the poles" [primary].
- **PTGui** has no documented special case for a Row whose Shots all look at the pole. What is documented: zenith and nadir shots are simply loaded with the rest and aligned by control points; if none are found (blue sky) the Project Assistant warns (FAQ 6.21) [primary]. Align to Grid 9.1.4 fix: "This caused problems when using Align to Grid with 360 degree panoramas … the nadir and zenith were not visible"; 9.1.7: "if one of the images ends up at the zenith or nadir, it may become misaligned"; 11.29: "straightening panorama did not work accurately if a nadir or zenith image was included" [primary, version history]. Support on roll: "Align to Grid does not introduce any roll by itself, but you're running the optimizer afterwards"; the anchor image keeps its roll, the others take "a few degrees" [secondary, PTGui group]. The `.pts` keeps `viewrotate: "auto"` per image, so the display rotation is detected, not declared [primary].
- **Hugin**: `cpfind --multirow` (default since 2024.0) matches neighbours first and falls back to "all not yet checked image pairs"; `--prealigned` "only matches images pairs, which overlaps and don't already have control point" [primary]. Yaw/pitch/roll are Euler angles; at pitch ±90° yaw and roll rotate about the same axis [estimate, geometry: `R_y(yaw)·R_x(90°) = R_x(90°)·R_z(±yaw)`].
- Consequence for Panowww [estimate]: a pole Row is N copies of one view rotated about the optic axis by `360/N` each. Their mutual Overlap is ~100 %, so a rotation-invariant detector matches them, but the only information they carry is roll and a confirmation of the pole position. Any single pole Shot covers a cap of radius `vfov/2 ≈ 27°` around the pole (World of Warcraft numbers), while the next Row at pitch 60° already reaches `60 + 27 = 87°` on its centre column: the hole at the pole is a cap of about 3° (wider between columns), which one pole Shot fills many times over.

---

## 2. What the known grid changes

Nominal yaw `Δy = 360 / shotsPerRow`, nominal pitch `Δp = 180 / (rows − 1)`, roll ≈ 0, start pole known, zone size known; unknown: one hfov and a small drift per Step. Szeliski says it in one sentence: with a pan-tilt head "pixel coordinates (x, y, f) must first be rotated using the known tilt and panning angles" and "Having a roughly known panning angle also makes it easier to compute the alignment, since the rough relative positioning of all the input images is known ahead of time, enabling a reduced search range" [primary, §2.3].

| Stage | General Stitcher | Panowww |
|---|---|---|
| Feature detection | all images, all scales, rotation-invariant | **Shrinks**: only the predicted overlap strips need features, and only one scale, since neighbours share scale and (except pole Rows) orientation. Or skip features: Szeliski's direct alternative, "dividing images into patches and creating a virtual feature correspondence for each one" [primary, §5.1], fits a known strip (phase correlation per patch). |
| Matching | O(n log n) over every feature, m = 6 candidates per image | **Skippable**: the pairs are known. Shot `(r, c)` touches `(r, c±1)` (wrapping) and `(r±1, c)`: ≤ 4 neighbours, 2n pairs. PTGui 11's "only for overlapping images" mode is this [primary]. |
| RANSAC | 500–1000 trials for a 4-point homography | **Shrinks to outlier rejection**: the homography is predicted from the grid; keep matches within a few pixels of it, no sampling needed [estimate]. `cpfind --ransacmode rpy` already uses the hfov prior instead of a free homography [primary]. |
| Initial guess | focal from homographies, spanning tree | **Skippable**: `R_(r,c) = R_y(c·Δy) · R_x(pole ± r·Δp)`, f from Overlap (below). |
| Bundle adjustment | 4 parameters per image, free | **Stays, small**: same unknown count but starts within a degree, so a handful of LM iterations [estimate]. |
| Straightening | eigenvector of camera x-axes | **Skippable**: the grid is level by construction; keep as a check that the up vector is unchanged [estimate]. |
| Seam finding | graph cut on colour | **Shrinks**: no moving objects, no exposure difference; Voronoi seams (nearest image centre) plus multi-band blending may be enough. Keep graph cut as an option for when alignment residual is visible [estimate]. |
| Exposure | gain compensation | **Skipped** (map: out of scope). |
| Reprojection and blending | full | **Unchanged**: this is what remains, and what PTGui puts on the GPU. |

### 2.1 Estimating hfov from the Overlap between neighbours

Rotating a pinhole camera by Δy about its vertical axis moves the centre pixel of Shot `c+1` to `x = f_px · tan(Δy)` in Shot `c` (Szeliski eq. 9 with the rotated ray) [estimate, derived]. So from the measured position `x_c` (pixels from centre) of neighbour's centre in a Row at pitch 0:

```
f_px = x_c / tan(Δy)          hfov = 2 · atan(w / (2 f_px))
```

A single Step measurement is enough; `shotsPerRow` of them around the equator Row average the drift out, and the vertical Step at the equator gives a second estimate `f_px = y_c / tan(Δp)`. In Overlap terms, with `o` the shared share of the width measured by *Check overlap*, `x_c ≈ (1 − o)·w`, which is why Overlap alone predicts hfov. The closed-loop check from Szeliski: if the chained Row leaves a gap angle θ_g, `f' = f·(1 − θ_g/360°)` [primary, §4.3 *Gap closing*]. OpenCV's `focalsFromHomography` gives the same from any pairwise homography: `f² = (h5² − h2²)/(h0² + h1² − h3² − h4²)` or `f² = −h2·h5/(h0·h3 + h1·h4)`, geometric mean per pair, median over pairs [primary, `autocalib.cpp`; Szeliski eq. 139–140].

World of Warcraft check [estimate]: with `f_px ≈ 833` and `Δy = 25.7°`, `x_c ≈ 401` px, i.e. the neighbour's centre sits 401 px from ours in a 1669 px wide Shot, a horizontal Overlap of about 76 % of the width; vertically `y_c = 833·tan 30° ≈ 481` px of 841, Overlap ≈ 43 %.

### 2.2 The minimal bundle adjustment

- **Parameters**: `f_px` (one scalar) and a rotation vector `θ_k ∈ ℝ³` per Shot, stored as an increment on the nominal `R_k⁰` (`R_k = exp([θ_k]×) · R_k⁰`), so pole Rows have no Euler singularity (Brown & Lowe's θ and OpenCV's Rodrigues do this) [primary for the representation]. `N = 3n + 1`; the first Shot is the gauge (θ fixed at 0), as PTGui's anchor and OpenCV's `R_inv · R` normalisation do [primary].
- **Residuals**: OpenCV's `BundleAdjusterRay` form, the difference of the two unit rays of a match scaled by `sqrt(f1·f2)` (3 residuals per match) [primary, `motion_estimators.cpp`], which Szeliski recommends because it removes the bias towards long focals (eq. 147) [primary]; or the pixel reprojection `p_ij − H·p_ji` (2 per match, Brown & Lowe eq. 15–16). Huber loss with σ = ∞ while initialising and **σ = 2 px** for the final solve [primary, Brown & Lowe].
- **Priors**: Brown & Lowe's `C_p` with `σ_θ = π/16` and `σ_f = f̄/10` [primary]; Panowww can use the calibration drift (a fraction of a degree) as σ_θ, which is the same as adding the residual `(θ_k − 0)/σ_θ` per Shot [estimate]. PTGui's Align to Grid optimizer "will attempt to keep the panorama aligned as well as possible" to the grid and moves images without control points with their neighbours [primary]: same idea.
- **Initialisation**: `R_k⁰` from the grid, `f_px` from §2.1. Brown & Lowe initialise each new image "with the same rotation and focal length as the image to which it best matches" because they have nothing better [primary]; Panowww starts every Shot within the Step drift of its final place.
- **Sparsity**: `JᵀJ` is block-sparse with 4-neighbour structure, "each iteration of bundle adjustment is O(N³)" if solved densely [primary]; for 98 Shots N = 295, a dense LM solve is milliseconds; for a 36 × 36 grid N = 3889, still seconds [estimate].

### 2.3 Mapping formulas for the spike

Rectilinear pixel → sphere (per Shot with rotation R, principal point at the zone centre) [primary, Szeliski eq. 35–39; OpenCV `SphericalProjector`]:

```
ray  = R · (x − w/2, y − h/2, f_px)                     // then normalise
lon  = atan2(ray.x, ray.z)                                // θ, −π..π
lat  = asin(ray.y / |ray|)                                // φ, −π/2..π/2
u    = (lon + π) / (2π) · W                               // equirectangular, W = 2π·f_px for 1:1 at the centre
v    = (π/2 − lat) / π · H,  H = W/2
```

Equirectangular pixel → rectilinear pixel (what the remapper runs per output pixel) [primary, same sources]:

```
lon = u/W · 2π − π;   lat = π/2 − v/H · π
d   = (sin lon · cos lat,  sin lat,  cos lon · cos lat)   // unit ray
p   = Rᵀ · d                                              // into the Shot's frame
if p.z <= 0: outside                                      // behind the camera
x   = w/2 + f_px · p.x / p.z;   y = h/2 + f_px · p.y / p.z
inside iff 0 <= x < w and 0 <= y < h
```

At the pole rows `lat = ±π/2`, `cos lat = 0`, the ray is `(0, ±1, 0)` for every `u`: one source pixel feeds a whole output row, and `atan2(0, 0)` in the forward map is undefined. Use the inverse map only, and supersample or build a mip level for the top and bottom bands [estimate].

Laplacian pyramid blending [secondary via Szeliski §6.2 and OpenCV `blenders.cpp`]: `L_l = G_l − EXPAND(G_{l+1})` per image; the mask's Gaussian pyramid gives the per-level weights; blend each level `Σ_i W_i,l · L_i,l / Σ_i W_i,l`; collapse coarse to fine with `pyrUp` and add. Number of levels for the World of Warcraft output: `blend_width = sqrt(5234 × 2617) × 0.05 ≈ 185 px`, `ceil(log2 185) − 1 = 7` bands in OpenCV's rule; Brown & Lowe used 5 [primary for the rule, estimate for the number]. Output is padded to a multiple of `2^bands` per side [primary].

---

## 3. Recommendation for a pure-Rust pipeline

Ordered by stage, each with what the spike must measure on *World of Warcraft/2026-11-07_23-14-46* (98 Shots, 14 × 7, 1669 × 841; PTGui's 5234 × 2617 Panorama is the reference):

1. **Grid prior**: build `R_k⁰` from `session.json` (`shotsPerRow`, `rows`, start pole). Measure nothing; this is bookkeeping.
2. **hfov from Overlap**: phase-correlate the predicted overlap strips of the equator Row (and the vertical Step at the equator), §2.1. Measure: the estimate against PTGui's ≈ 90°; its spread across the Row (the drift).
3. **Pairwise alignment on the 4-neighbour graph only**: per pair, either (a) direct: phase correlation or NCC on 3–5 patches of the predicted strip (virtual control points, Szeliski §5.1), or (b) features in the strip with a rotation-invariant detector for pole Rows. Start with (a); it has no detector to port and no descriptor to match. Measure: pairs with a confident peak, residual in pixels after the prior homography, time per pair.
4. **Bundle adjustment**, §2.2: LM over `f_px + 3n`, ray residuals, Huber σ = 2 px, drift prior. Measure: iterations, final RMS residual (PTGui's FAQ 3.16 treats control point distance as the quality number), time.
5. **Pole Rows**: treat each as one view: pick the Shot whose roll is nearest 0 after step 4, or blend all N with equal weight (they are the same view rotated). Measure: whether using one Shot leaves a visible seam at the cap boundary.
6. **Reprojection** with the inverse map of §2.3, bilinear first, bicubic if the reference shows a difference, `rayon` over output rows, `W = 2π f_px` rounded to the reference size. Measure: time at 5234 × 2617 on the CPU; this is the stage PTGui offloads, so it decides "CPU first, GPU later" (map, not yet specified).
7. **Seams**: Voronoi by angular distance to each Shot's centre (closed form, no search). Only if ghosting appears, a graph cut at a 0.1 Mpx scale as OpenCV does. Measure: whether any seam is visible on the reference.
8. **Blending**: Laplacian pyramid, bands from the OpenCV rule (≈ 7), 16-bit or f32 intermediates. Measure: time and peak memory against the 16-bit PTGui path; a feather blend as the fallback if memory is the limit on a 36 × 36 grid.
9. **Write** JPEG at the reference quality (95) into the Session folder, and the per-Shot `R_k`, `f_px` and residuals next to it so a later editor can start from them.

What to compare with PTGui: wall time to the saved Panorama with the project's defaults (*Align images* + *Create panorama*, optimum size), on the same machine, and the two Panoramas side by side at the seams and the poles. PTGui's number for a 760 Mpx output says nothing about a 14 Mpx one; FAQ 3.22 says output size dominates, so the comparison must be at equal output size.

Not recommended: porting SIFT or SURF for the general case (Hugin's detector is GPL, `deny.toml` excludes it), or a general matcher. Panowww does not have the problem those solve.

---

## 4. Sources

- PTGui: [What's new in 10](https://ptgui.com/whatsnew/100000), [in 9.1](https://ptgui.com/whatsnew/90100), [in 11](https://ptgui.com/whatsnew/110000), [in 12](https://ptgui.com/whatsnew/120000), [version history](https://ptgui.com/versionhistory.html), [support / FAQ](https://ptgui.com/support.html) (entries 3.6, 3.11, 3.15, 3.16, 3.22, 3.30, 3.31, 4.5, 4.11, 6.21, 8.1–8.4), [Creating gigapixel panoramas with a robotic panohead](https://ptgui.com/examples/creating_gigapixel_panoramas_with_a_robotic_panohead.html), [parsepts](https://ptgui.com/parsepts.html); the `.pts` project PTGui Pro 13.10 saved in `~/Pictures/Panowww/World of Warcraft/2026-11-07_23-14-46/` (schema `project_v55`).
- PTGui group (secondary): [Align to Grid introduces Roll?](https://groups.google.com/g/ptgui/c/OieMA_beybU).
- Hugin manual: [Cpfind](https://hugin.sourceforge.io/docs/manual/Cpfind.html), [Lens correction model](https://hugin.sourceforge.io/docs/manual/Lens_correction_model.html), [Nona](https://hugin.sourceforge.io/docs/manual/Nona.html), [Autooptimiser](https://hugin.sourceforge.io/docs/manual/Autooptimiser.html), [Hugin Stitcher tab](https://hugin.sourceforge.io/docs/manual/Hugin_Stitcher_tab.html), [Hugin Preferences](https://hugin.sourceforge.io/docs/manual/Hugin_Preferences.html); source [KeyPointDetector.cpp](https://hugin.sourceforge.io/docs/html/KeyPointDetector_8cpp_source.html) (Panomatic, Anael Orlinski, GPL).
- PanoTools wiki: [PTStitcher](https://wiki.panotools.org/PTStitcher), [Field of View](https://wiki.panotools.org/Field_of_View), [Equirectangular Projection](https://wiki.panotools.org/Equirectangular_Projection), [Rectilinear Projection](https://wiki.panotools.org/Rectilinear_Projection).
- enblend: [man page (Arch, 4.2)](https://man.archlinux.org/man/enblend.1.en); hugin-ptx: [New primary seam generator in Enblend](https://groups.google.com/g/hugin-ptx/c/UbUQoxeTXfE) (secondary); nona GPU: [Launchpad bug 1362170](https://bugs.launchpad.net/hugin/+bug/1362170), [hugin-ptx thread](https://groups.google.com/g/hugin-ptx/c/iBCV79KY3Eg/m/9GZBwVKGBwAJ) (secondary).
- OpenCV 4.x `modules/stitching`: [stitcher.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/src/stitcher.cpp), [matchers.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/src/matchers.cpp), [motion_estimators.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/src/motion_estimators.cpp), [autocalib.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/src/autocalib.cpp), [blenders.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/src/blenders.cpp), [warpers_inl.hpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/stitching/include/opencv2/stitching/detail/warpers_inl.hpp), [samples/cpp/stitching_detailed.cpp](https://raw.githubusercontent.com/opencv/opencv/4.x/samples/cpp/stitching_detailed.cpp).
- Papers: M. Brown, D. Lowe, [Automatic Panoramic Image Stitching using Invariant Features](https://www.cs.ubc.ca/~lowe/papers/07brown.pdf), IJCV 2007; D. Lowe, [Distinctive Image Features from Scale-Invariant Keypoints](https://www.cs.ubc.ca/~lowe/papers/ijcv04.pdf), IJCV 2004; R. Szeliski, [Image Alignment and Stitching: A Tutorial](https://www.microsoft.com/en-us/research/wp-content/uploads/2004/10/tr-2004-92.pdf), MSR-TR-2004-92 (§2.3, §4.3, §5.1, §6.1–6.2); P. Burt, E. Adelson, *A Multiresolution Spline With Application to Image Mosaics*, ACM ToG 2(4), 1983 (not fetched: [persci.mit.edu](https://persci.mit.edu/pub_pdfs/spline83.pdf) has an expired certificate).
