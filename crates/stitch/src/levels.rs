//! The Read stage: every Shot fetched once, reduced to grey copies at 1/2,
//! 1/4 and 1/8 for the alignment, the full frame dropped at once so the
//! working set stays under the limits (spec §16.5).

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "pixel coordinates and sizes move between integers and floats throughout"
)]

use rayon::prelude::*;

use panowww_core::ports::{CancelFlag, Frame};

use crate::error::StitchError;
use crate::grid::Grid;
use crate::report::{Stage, StitchProgress};
use crate::{Limits, ShotSource};

/// One grey copy of a Shot: `scale` full-resolution pixels per pixel.
#[derive(Debug, Clone)]
pub struct Level {
    pub w: usize,
    pub h: usize,
    pub scale: f64,
    pub g: Vec<u8>,
}

impl Level {
    /// Bilinear sample, `None` outside the image.
    pub fn sample(&self, x: f64, y: f64) -> Option<f32> {
        if !(x >= 0.0 && y >= 0.0 && x < (self.w - 1) as f64 && y < (self.h - 1) as f64) {
            return None;
        }
        let (x0, y0) = (x as usize, y as usize);
        let (fx, fy) = ((x - x0 as f64) as f32, (y - y0 as f64) as f32);
        let p = |xx: usize, yy: usize| f32::from(self.g[yy * self.w + xx]);
        let top = p(x0, y0) * (1.0 - fx) + p(x0 + 1, y0) * fx;
        let bot = p(x0, y0 + 1) * (1.0 - fx) + p(x0 + 1, y0 + 1) * fx;
        Some(top * (1.0 - fy) + bot * fy)
    }

    /// Mean of 2 × 2 blocks.
    fn halved(&self) -> Level {
        let (w, h) = (self.w / 2, self.h / 2);
        let mut g = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let i = (2 * y) * self.w + 2 * x;
                let sum = u32::from(self.g[i])
                    + u32::from(self.g[i + 1])
                    + u32::from(self.g[i + self.w])
                    + u32::from(self.g[i + self.w + 1]);
                g[y * w + x] = ((sum + 2) / 4) as u8;
            }
        }
        Level {
            w,
            h,
            scale: self.scale * 2.0,
            g,
        }
    }
}

/// The grey copies of one Shot and where it sits in the grid.
#[derive(Debug, Clone)]
pub struct ShotLevels {
    pub row: u32,
    pub col: u32,
    pub half: Level,
    pub quarter: Level,
    pub eighth: Level,
}

impl ShotLevels {
    pub fn level(&self, scale: u32) -> &Level {
        match scale {
            8 => &self.eighth,
            4 => &self.quarter,
            _ => &self.half,
        }
    }
}

/// Every Shot's grey copies, in source order, plus the Shots' size.
#[derive(Debug, Clone)]
pub struct Pyramids {
    shots: Vec<ShotLevels>,
    pub width: u32,
    pub height: u32,
}

impl Pyramids {
    /// Reads every Shot in parallel. Fails on a Shot that cannot be read,
    /// on Shots of different sizes, when the copies alone exceed the
    /// limits, or when the flag is raised.
    pub fn build(
        source: &dyn ShotSource,
        grid: &Grid,
        limits: Limits,
        progress: &(dyn Fn(StitchProgress) + Sync),
        cancel: &CancelFlag,
    ) -> Result<Self, StitchError> {
        let count = source.count();
        let _ = grid;
        let first = source.shot(0).map_err(|error| {
            let (row, col) = source.position(0);
            StitchError::Shot {
                row,
                col,
                source: error,
            }
        })?;
        let (width, height) = (first.width, first.height);
        let per_shot = (width as usize / 2) * (height as usize / 2) * 21 / 16;
        let needed = per_shot * count;
        if needed > limits.memory_bytes {
            return Err(StitchError::Memory {
                needed_mb: (needed / (1024 * 1024)) as u64,
                budget_mb: (limits.memory_bytes / (1024 * 1024)) as u64,
            });
        }
        progress(StitchProgress::new(Stage::Read, 0.0));
        let done = std::sync::atomic::AtomicUsize::new(0);
        let shots: Vec<ShotLevels> = (0..count)
            .into_par_iter()
            .map(|index| -> Result<ShotLevels, StitchError> {
                if cancel.is_raised() {
                    return Err(StitchError::Cancelled);
                }
                let (row, col) = source.position(index);
                let frame = if index == 0 {
                    first.clone()
                } else {
                    source.shot(index).map_err(|error| StitchError::Shot { row, col, source: error })?
                };
                if frame.width != width || frame.height != height || !frame.is_well_formed() {
                    return Err(StitchError::Grid(format!(
                        "Shot r{row:02}_c{col:02} is {} × {} where the Session's Shots are {width} × {height}.",
                        frame.width, frame.height
                    )));
                }
                let half = half_grey(&frame);
                let quarter = half.halved();
                let eighth = quarter.halved();
                let n = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if n.is_multiple_of(8) || n == count {
                    progress(StitchProgress::new(Stage::Read, n as f32 / count as f32));
                }
                Ok(ShotLevels {
                    row,
                    col,
                    half,
                    quarter,
                    eighth,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            shots,
            width,
            height,
        })
    }

    pub fn count(&self) -> usize {
        self.shots.len()
    }

    pub fn position(&self, index: usize) -> (u32, u32) {
        (self.shots[index].row, self.shots[index].col)
    }

    pub fn shot(&self, index: usize) -> &ShotLevels {
        &self.shots[index]
    }

    pub fn shots(&self) -> &[ShotLevels] {
        &self.shots
    }

    /// Index of the Shot at `(row, col)`, if any.
    pub fn index_of(&self, row: u32, col: u32) -> Option<usize> {
        self.shots.iter().position(|s| s.row == row && s.col == col)
    }

    /// The right neighbour in the Row (wrapping) and the Shot in the next Row.
    pub fn neighbour_pairs(&self, grid: &Grid) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (i, s) in self.shots.iter().enumerate() {
            let next_col = if s.col == grid.shots_per_row {
                1
            } else {
                s.col + 1
            };
            if let Some(j) = self.index_of(s.row, next_col)
                && j != i
            {
                out.push((i, j));
            }
            if let Some(j) = self.index_of(s.row + 1, s.col) {
                out.push((i, j));
            }
        }
        out
    }
}

/// Grey at half resolution: luma of each 2 × 2 block.
fn half_grey(frame: &Frame) -> Level {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let (gw, gh) = (w / 2, h / 2);
    let px = &frame.rgb;
    let mut g = vec![0u8; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let mut acc = 0f32;
            for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let i = ((2 * y + dy) * w + (2 * x + dx)) * 3;
                acc += 0.299 * f32::from(px[i])
                    + 0.587 * f32::from(px[i + 1])
                    + 0.114 * f32::from(px[i + 2]);
            }
            g[y * gw + x] = (acc * 0.25).round().clamp(0.0, 255.0) as u8;
        }
    }
    Level {
        w: gw,
        h: gh,
        scale: 2.0,
        g,
    }
}
