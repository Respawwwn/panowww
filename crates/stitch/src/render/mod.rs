//! The Render and Blend stages (spec §16.6 steps 6 and 7) behind one seam:
//! [`Renderer`], with [`CpuRenderer`] as the implementation and a `wgpu`
//! one possible later without touching the alignment. The Panorama is
//! produced by bands of rows under the memory limits: for each band the
//! Shots that cover it are fetched, every pixel takes one Shot (the label
//! map), and a multi-band blend on the label masks mixes the Shots only
//! across the seam lines, so nothing misaligned is averaged.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    reason = "pixel coordinates and sizes move between integers and floats throughout, image maths reads better with short names, and the loops index several planes at once"
)]

mod pyramid;
mod seams;
mod view;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;

use panowww_core::ports::{CancelFlag, Frame};

use crate::error::StitchError;
use crate::geom::{Cam, Rot};
use crate::report::{Stage, StitchProgress};
use crate::{Limits, ShotSource};
use pyramid::{ALIGN, LEVELS, Plane, expand, gaussian, laplacian};
use view::{INSIDE_MARGIN, Sampler, ShotView, direction};

/// Widest preview written next to the Panorama (spec §16.4).
pub const PREVIEW_MAX_WIDTH: usize = 8192;
/// Rows a band extends on each side so the blend sees past its edges.
const BAND_MARGIN: usize = 2 * ALIGN;
/// No Shot: a pixel nobody sees.
const NO_LABEL: u16 = u16::MAX;

/// Everything the render needs from the alignment.
pub struct RenderInput<'a> {
    pub source: &'a dyn ShotSource,
    pub shot_width: u32,
    pub shot_height: u32,
    pub poses: &'a [Rot],
    pub f_px: f64,
    pub limits: Limits,
    /// The seam finder; off, Shots meet halfway between their centres.
    pub anti_ghost: bool,
    /// Shots kept out of the Panorama: what only they saw stays black.
    pub left_out: &'a [usize],
}

/// What the render produced.
#[derive(Debug, Clone)]
pub struct Rendered {
    pub panorama: Frame,
    pub preview: Frame,
    pub render_ms: u32,
    pub blend_ms: u32,
    /// The working set the render sized itself to, for the record.
    pub peak_estimate_bytes: usize,
}

/// The seam between the alignment and the pixels.
pub trait Renderer {
    /// # Errors
    ///
    /// [`StitchError::Shot`] when a Shot cannot be fetched again,
    /// [`StitchError::Cancelled`] between bands.
    fn render(
        &self,
        input: &RenderInput,
        progress: &(dyn Fn(StitchProgress) + Sync),
        cancel: &CancelFlag,
    ) -> Result<Rendered, StitchError>;
}

/// The CPU render: `rayon` over rows, bands under the limits.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuRenderer;

/// Panorama size for a focal length: full resolution at the equator, the
/// width a multiple of twice the pyramid alignment so every level wraps.
pub fn canvas_size(f_px: f64) -> (usize, usize) {
    let raw = (std::f64::consts::TAU * f_px).floor() as usize;
    let width = (raw / (2 * ALIGN)).max(2) * (2 * ALIGN);
    (width, width / 2)
}

impl Renderer for CpuRenderer {
    fn render(
        &self,
        input: &RenderInput,
        progress: &(dyn Fn(StitchProgress) + Sync),
        cancel: &CancelFlag,
    ) -> Result<Rendered, StitchError> {
        let started = Instant::now();
        let (width, height) = canvas_size(input.f_px);
        let cam = Cam::new(input.f_px, input.shot_width, input.shot_height);
        let views: Vec<ShotView> = input
            .poses
            .iter()
            .map(|r| ShotView::new(r, &cam, height))
            .collect();
        // Band height from the limits: the band pyramids, their temporaries,
        // and the frames of the Shots that cover a band.
        let shot_bytes = (input.shot_width as usize) * (input.shot_height as usize) * 3;
        let per_row = width * (3 + 1) * 4 * 3; // accumulators, weights, temporaries, pyramid overhead
        let frames_for_band = 3 * covering_per_row(&views, height).max(1);
        let budget_for_rows = input
            .limits
            .memory_bytes
            .saturating_sub(frames_for_band * shot_bytes);
        let band_rows =
            ((budget_for_rows / per_row.max(1)) / ALIGN * ALIGN).clamp(2 * ALIGN, 16 * ALIGN);
        let peak_estimate_bytes =
            frames_for_band * shot_bytes + (band_rows + 2 * BAND_MARGIN) * per_row;
        let mut out = vec![0u8; width * height * 3];
        let mut cache: HashMap<usize, Arc<Frame>> = HashMap::new();
        let bands: Vec<(usize, usize)> = (0..height)
            .step_by(band_rows)
            .map(|b0| (b0, (b0 + band_rows).min(height)))
            .collect();
        let mut blend_ms = 0u128;
        for (k, &(b0, b1)) in bands.iter().enumerate() {
            if cancel.is_raised() {
                return Err(StitchError::Cancelled);
            }
            progress(StitchProgress::new(
                Stage::Render,
                k as f32 / bands.len() as f32,
            ));
            let e0 = b0.saturating_sub(BAND_MARGIN);
            let e1 = (b1 + BAND_MARGIN).min(height);
            let all: Vec<usize> = (0..views.len())
                .filter(|&i| views[i].rows.0 < e1 && views[i].rows.1 >= e0)
                .collect();
            let touching: Vec<usize> = all
                .iter()
                .copied()
                .filter(|i| !input.left_out.contains(i))
                .collect();
            fetch_frames(input.source, &views, &touching, e0, &mut cache)?;
            let eh = e1 - e0;
            let labels = label_band(&views, &touching, &cam, width, height, e0, eh);
            let hole = if touching.len() < all.len() {
                hole_band(&labels, &views, &all, &cam, width, height, e0, eh)
            } else {
                Vec::new()
            };
            let labels = if input.anti_ghost {
                seams::refine(
                    labels, &views, &touching, &cache, &cam, width, height, e0, eh,
                )
            } else {
                labels
            };
            let blend_started = Instant::now();
            blend_band(
                &mut out,
                &labels,
                &views,
                &touching,
                &cache,
                &cam,
                width,
                height,
                (e0, e1),
                (b0, b1),
                &hole,
            );
            blend_ms += blend_started.elapsed().as_millis();
        }
        progress(StitchProgress::new(Stage::Render, 1.0));
        progress(StitchProgress::new(Stage::Blend, 1.0));
        let panorama = Frame {
            width: width as u32,
            height: height as u32,
            rgb: out,
        };
        let preview = preview_of(&panorama);
        let total = started.elapsed().as_millis();
        Ok(Rendered {
            panorama,
            preview,
            render_ms: u32::try_from(total.saturating_sub(blend_ms)).unwrap_or(u32::MAX),
            blend_ms: u32::try_from(blend_ms).unwrap_or(u32::MAX),
            peak_estimate_bytes,
        })
    }
}

/// How many Shots can cover one row at most: the widest set of overlapping
/// row ranges.
fn covering_per_row(views: &[ShotView], height: usize) -> usize {
    let mut delta = vec![0i32; height + 1];
    for v in views {
        delta[v.rows.0] += 1;
        delta[v.rows.1 + 1] -= 1;
    }
    let mut cur = 0;
    let mut best = 0;
    for d in delta {
        cur += d;
        best = best.max(cur);
    }
    best.max(0) as usize
}

/// For every pixel of the band, the Shot whose optical axis is nearest
/// among those that see it well inside their image.
fn label_band(
    views: &[ShotView],
    touching: &[usize],
    cam: &Cam,
    width: usize,
    height: usize,
    e0: usize,
    eh: usize,
) -> Vec<u16> {
    let (f, cx, cy) = (cam.f as f32, cam.cx as f32, cam.cy as f32);
    let (w, h) = (cam.w as f32, cam.h as f32);
    let mut labels = vec![NO_LABEL; width * eh];
    labels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(j, row)| {
            let y = e0 + j;
            for x in 0..width {
                let d = direction(x, y, width, height);
                let mut best = -9f32;
                for &i in touching {
                    let v = &views[i];
                    let dot = v.dot(d);
                    if dot < v.cos_lim || dot <= best {
                        continue;
                    }
                    let l = v.local(d);
                    if l[2] <= 1e-6 {
                        continue;
                    }
                    let u = cx + f * l[0] / l[2];
                    let vv = cy - f * l[1] / l[2];
                    if u < INSIDE_MARGIN
                        || vv < INSIDE_MARGIN
                        || u > w - 1.0 - INSIDE_MARGIN
                        || vv > h - 1.0 - INSIDE_MARGIN
                    {
                        continue;
                    }
                    best = dot;
                    row[x] = i as u16;
                }
            }
        });
    labels
}

/// The frames of the Shots that touch a band: the new ones fetched in
/// parallel, the ones the band is past dropped.
fn fetch_frames(
    source: &dyn ShotSource,
    views: &[ShotView],
    touching: &[usize],
    e0: usize,
    cache: &mut HashMap<usize, Arc<Frame>>,
) -> Result<(), StitchError> {
    cache.retain(|&i, _| views[i].rows.1 >= e0);
    let missing: Vec<usize> = touching
        .iter()
        .copied()
        .filter(|i| !cache.contains_key(i))
        .collect();
    let fetched: Vec<(usize, Result<Frame, StitchError>)> = missing
        .par_iter()
        .map(|&i| {
            let (row, col) = source.position(i);
            (
                i,
                source
                    .shot(i)
                    .map_err(|source| StitchError::Shot { row, col, source }),
            )
        })
        .collect();
    for (i, frame) in fetched {
        cache.insert(i, Arc::new(frame?));
    }
    Ok(())
}

/// The pixels of the band a left-out Shot would have taken and no other
/// Shot sees: `labels` were drawn without the left-out Shots, `all` holds
/// every Shot touching the band.
#[allow(
    clippy::too_many_arguments,
    reason = "the band, its Shots and its place, named"
)]
fn hole_band(
    labels: &[u16],
    views: &[ShotView],
    all: &[usize],
    cam: &Cam,
    width: usize,
    height: usize,
    e0: usize,
    eh: usize,
) -> Vec<bool> {
    label_band(views, all, cam, width, height, e0, eh)
        .iter()
        .zip(labels)
        .map(|(&with, &without)| without == NO_LABEL && with != NO_LABEL)
        .collect()
}

/// The circular range of columns a label occupies in the band, grown by a
/// margin and aligned, as `(start, width)` with `start` possibly negative.
fn label_box(
    labels: &[u16],
    width: usize,
    eh: usize,
    i: u16,
    margin: usize,
) -> Option<(isize, usize)> {
    let mut used = vec![false; width];
    let mut any = false;
    for j in 0..eh {
        for (x, &l) in labels[j * width..(j + 1) * width].iter().enumerate() {
            if l == i {
                used[x] = true;
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    // Longest circular run of unused columns; the box is its complement.
    let first_used = used.iter().position(|&u| u).expect("some column is used");
    let (mut best_start, mut best_len, mut run_start, mut run_len) =
        (0usize, 0usize, 0usize, 0usize);
    for k in 0..width {
        let x = (first_used + k) % width;
        if used[x] {
            run_len = 0;
        } else {
            if run_len == 0 {
                run_start = x;
            }
            run_len += 1;
            if run_len > best_len {
                best_len = run_len;
                best_start = run_start;
            }
        }
    }
    let (x0, bw) = if best_len == 0 {
        (0isize, width)
    } else {
        (((best_start + best_len) % width) as isize, width - best_len)
    };
    let x0 = (x0 - margin as isize).div_euclid(ALIGN as isize) * ALIGN as isize;
    let bw = ((bw + 2 * margin).div_ceil(ALIGN) * ALIGN).min(width);
    Some((x0, bw))
}

/// Multi-band blend of one band: every Shot with pixels in it is warped
/// into the box around its label region (edge-extended), its Laplacian
/// pyramid weighted by the Gaussian pyramid of its mask and accumulated
/// into the band's pyramids, wrapping in longitude; the collapse is written
/// into `out` for the band's own rows.
#[allow(clippy::too_many_arguments, reason = "one stage, its inputs named")]
fn blend_band(
    out: &mut [u8],
    labels: &[u16],
    views: &[ShotView],
    touching: &[usize],
    cache: &HashMap<usize, Arc<Frame>>,
    cam: &Cam,
    width: usize,
    height: usize,
    extended: (usize, usize),
    own: (usize, usize),
    hole: &[bool],
) {
    let (e0, e1) = extended;
    let eh = e1 - e0;
    let (f, cx, cy) = (cam.f as f32, cam.cx as f32, cam.cy as f32);
    let mut acc: Vec<[Plane; 3]> = Vec::new();
    let mut wsum: Vec<Plane> = Vec::new();
    for l in 0..LEVELS {
        let (w, h) = (width >> l, eh >> l);
        acc.push([Plane::zeros(w, h), Plane::zeros(w, h), Plane::zeros(w, h)]);
        wsum.push(Plane::zeros(w, h));
    }
    for &i in touching {
        let Some((bx0, bw)) = label_box(labels, width, eh, i as u16, BAND_MARGIN) else {
            continue;
        };
        let v = &views[i];
        let sampler = Sampler::new(&cache[&i]);
        let mut planes = [
            Plane::zeros(bw, eh),
            Plane::zeros(bw, eh),
            Plane::zeros(bw, eh),
        ];
        let mut mask = Plane::zeros(bw, eh);
        let [p0, p1, p2] = &mut planes;
        p0.d.par_chunks_mut(bw)
            .zip(p1.d.par_chunks_mut(bw))
            .zip(p2.d.par_chunks_mut(bw))
            .zip(mask.d.par_chunks_mut(bw))
            .enumerate()
            .for_each(|(j, (((r0, r1), r2), m))| {
                let y = e0 + j;
                for k in 0..bw {
                    let x = (bx0 + k as isize).rem_euclid(width as isize) as usize;
                    m[k] = if labels[j * width + x] as usize == i {
                        1.0
                    } else {
                        0.0
                    };
                    let l = v.local(direction(x, y, width, height));
                    let lz = l[2].max(0.05);
                    let px = sampler.rgb(cx + f * l[0] / lz, cy - f * l[1] / lz);
                    r0[k] = px[0];
                    r1[k] = px[1];
                    r2[k] = px[2];
                }
            });
        let gm = gaussian(mask);
        let [pa, pb, pc] = planes;
        let laps = [
            laplacian(&gaussian(pa)),
            laplacian(&gaussian(pb)),
            laplacian(&gaussian(pc)),
        ];
        for l in 0..LEVELS {
            let (cw, lw, lh) = (width >> l, bw >> l, eh >> l);
            let ox = bx0 >> l;
            let m = &gm[l];
            for c in 0..3 {
                let lap = &laps[c][l];
                acc[l][c]
                    .d
                    .par_chunks_mut(cw)
                    .enumerate()
                    .take(lh)
                    .for_each(|(j, row)| {
                        for k in 0..lw {
                            let x = (ox + k as isize).rem_euclid(cw as isize) as usize;
                            row[x] += lap.d[j * lw + k] * m.d[j * lw + k];
                        }
                    });
            }
            wsum[l]
                .d
                .par_chunks_mut(cw)
                .enumerate()
                .take(lh)
                .for_each(|(j, row)| {
                    for k in 0..lw {
                        let x = (ox + k as isize).rem_euclid(cw as isize) as usize;
                        row[x] += m.d[j * lw + k];
                    }
                });
        }
    }
    let result = collapse(&acc, &wsum, width, eh);
    let (b0, b1) = own;
    out[b0 * width * 3..b1 * width * 3]
        .par_chunks_mut(width * 3)
        .enumerate()
        .for_each(|(jj, row)| {
            let j = b0 - e0 + jj;
            for x in 0..width {
                // What only a left-out Shot saw stays black: the coarse
                // levels of the blend would otherwise smear its neighbours
                // into the hole.
                if !hole.is_empty() && hole[j * width + x] {
                    continue;
                }
                for c in 0..3 {
                    row[x * 3 + c] = result[c].d[j * width + x].round().clamp(0.0, 255.0) as u8;
                }
            }
        });
}

/// Normalises every level of the band's pyramids by its weights and
/// collapses them into one plane per channel.
fn collapse(acc: &[[Plane; 3]], wsum: &[Plane], width: usize, eh: usize) -> Vec<Plane> {
    let mut result: Vec<Plane> = Vec::new();
    for c in 0..3 {
        let mut img: Option<Plane> = None;
        for l in (0..LEVELS).rev() {
            let (w, h) = (width >> l, eh >> l);
            let mut lvl = Plane::zeros(w, h);
            lvl.d.par_iter_mut().enumerate().for_each(|(k, v)| {
                let ws = wsum[l].d[k];
                *v = if ws > 1e-4 { acc[l][c].d[k] / ws } else { 0.0 };
            });
            img = Some(match img {
                None => lvl,
                Some(prev) => {
                    let up = expand(&prev, w, h);
                    lvl.d
                        .par_iter_mut()
                        .enumerate()
                        .for_each(|(k, v)| *v += up.d[k]);
                    lvl
                }
            });
        }
        result.push(img.expect("one level at least"));
    }
    result
}

/// The preview: the Panorama reduced by an integer factor to at most
/// [`PREVIEW_MAX_WIDTH`] wide, or a copy when it already fits.
pub fn preview_of(panorama: &Frame) -> Frame {
    let (w, h) = (panorama.width as usize, panorama.height as usize);
    let k = w.div_ceil(PREVIEW_MAX_WIDTH).max(1);
    if k == 1 {
        return panorama.clone();
    }
    let (pw, ph) = (w / k, h / k);
    let mut rgb = vec![0u8; pw * ph * 3];
    rgb.par_chunks_mut(pw * 3).enumerate().for_each(|(y, row)| {
        for x in 0..pw {
            let mut acc = [0u32; 3];
            for dy in 0..k {
                for dx in 0..k {
                    let i = ((y * k + dy) * w + x * k + dx) * 3;
                    for c in 0..3 {
                        acc[c] += u32::from(panorama.rgb[i + c]);
                    }
                }
            }
            for c in 0..3 {
                row[x * 3 + c] = ((acc[c] + (k * k / 2) as u32) / (k * k) as u32) as u8;
            }
        }
    });
    Frame {
        width: pw as u32,
        height: ph as u32,
        rgb,
    }
}
