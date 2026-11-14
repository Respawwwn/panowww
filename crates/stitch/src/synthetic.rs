//! A synthetic Session for tests: a procedural equirectangular rendered
//! into Shots through the gimbal model with a known grid and known
//! perturbations, and an in-memory [`ShotSource`] over them. Every number
//! the Stitch measures has a truth here.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    reason = "pixel coordinates and sizes move between integers and floats throughout, and colour maths reads better with short names"
)]

use rayon::prelude::*;

use panowww_core::game::StartPole;
use panowww_core::ports::{Frame, StoreError};

use crate::ShotSource;
use crate::geom::{Cam, Rot, pose};
use crate::grid::Grid;

/// How the synthetic capture departs from its grid, all in degrees.
#[derive(Debug, Clone, PartialEq)]
pub struct Perturbation {
    /// Row 1 stops this far short of its pole (positive = short).
    pub start_pole_offset: f64,
    /// Pitch between Rows; the grid assumes `180 / (rows − 1)`.
    pub pitch_step: f64,
    /// Yaw between Shots of a Row; the grid assumes `360 / shots_per_row`.
    pub turn_step: f64,
    /// Heading offset of each Row, index `row - 1`; empty means none.
    pub row_headings: Vec<f64>,
    /// `(row, col)` Shots filled with a flat colour: no control points.
    pub flat_shots: Vec<(u32, u32)>,
    /// `(row, col)`: the Shot at `col + 1` is the same image as `col`, the
    /// camera did not move for that Step; the rest of the Row lags one Step.
    pub dropped_step: Option<(u32, u32)>,
}

impl Perturbation {
    /// A capture exactly on its grid.
    pub fn none(grid: &Grid) -> Self {
        Self {
            start_pole_offset: 0.0,
            pitch_step: 180.0 / f64::from(grid.rows - 1),
            turn_step: 360.0 / f64::from(grid.shots_per_row),
            row_headings: Vec::new(),
            flat_shots: Vec::new(),
            dropped_step: None,
        }
    }

    /// The hard case of spec §16.5: start pole 4° short, turn Step 26.7°
    /// on a 14-wide grid, one Row 40° further round, two flat Shots at the
    /// zenith, one dropped Step in Row 4.
    pub fn hard(grid: &Grid) -> Self {
        let mut headings = vec![0.0; grid.rows as usize];
        if grid.rows >= 5 {
            headings[4] = 40.0;
        }
        Self {
            start_pole_offset: 4.0,
            pitch_step: 180.0 / f64::from(grid.rows - 1) - 1.0,
            turn_step: 360.0 / f64::from(grid.shots_per_row) + 1.0,
            row_headings: headings,
            flat_shots: vec![(grid.rows, 2), (grid.rows, 5)],
            dropped_step: Some((4.min(grid.rows), 7.min(grid.shots_per_row - 1))),
        }
    }
}

/// The truth behind a synthetic Session.
#[derive(Debug, Clone, PartialEq)]
pub struct Truth {
    pub f_px: f64,
    /// Pose of every Shot, in source order.
    pub poses: Vec<Rot>,
}

/// Shots in memory with their grid positions.
#[derive(Debug, Clone)]
pub struct MemorySource {
    pub shots: Vec<(u32, u32, Frame)>,
}

impl ShotSource for MemorySource {
    fn count(&self) -> usize {
        self.shots.len()
    }

    fn position(&self, index: usize) -> (u32, u32) {
        (self.shots[index].0, self.shots[index].1)
    }

    fn shot(&self, index: usize) -> Result<Frame, StoreError> {
        Ok(self.shots[index].2.clone())
    }
}

/// A hashed value in 0..1 for an integer cell.
fn hash01(cell_x: i64, cell_y: i64, seed: i64) -> f64 {
    let mut h = (cell_x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (cell_y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (seed as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    (h & 0xff_ffff) as f64 / f64::from(0xff_ffffu32)
}

/// Smooth value noise at one scale, degrees per cell.
fn noise(lon_deg: f64, lat_deg: f64, cell_deg: f64, seed: i64) -> f64 {
    let (x, y) = ((lon_deg + 180.0) / cell_deg, (lat_deg + 90.0) / cell_deg);
    let (cell_x, cell_y) = (x.floor(), y.floor());
    let (fx, fy) = (x - cell_x, y - cell_y);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (cell_x, cell_y) = (cell_x as i64, cell_y as i64);
    let top = hash01(cell_x, cell_y, seed) * (1.0 - sx) + hash01(cell_x + 1, cell_y, seed) * sx;
    let bot =
        hash01(cell_x, cell_y + 1, seed) * (1.0 - sx) + hash01(cell_x + 1, cell_y + 1, seed) * sx;
    top * (1.0 - sy) + bot * sy
}

/// The colour of the procedural world in a direction: value noise at four
/// scales that share no period, so nothing repeats; a few discs; a flat
/// sky above 70° so the zenith Row has nothing to match.
pub fn world(lon_deg: f64, lat_deg: f64) -> [u8; 3] {
    if lat_deg > 70.0 {
        return [150, 180, 230];
    }
    let n = |seed: i64| {
        0.45 * noise(lon_deg, lat_deg, 29.0, seed)
            + 0.30 * noise(lon_deg, lat_deg, 11.0, seed + 7)
            + 0.17 * noise(lon_deg, lat_deg, 3.7, seed + 13)
            + 0.08 * noise(lon_deg, lat_deg, 1.3, seed + 31)
    };
    let mut r = 30.0 + 200.0 * n(1);
    let mut g = 30.0 + 200.0 * n(2);
    let mut b = 30.0 + 200.0 * n(3);
    for (dlon, dlat, radius) in [
        (30.0, 10.0, 6.0),
        (-100.0, -20.0, 9.0),
        (150.0, 40.0, 5.0),
        (-20.0, -60.0, 8.0),
    ] {
        let d = ((lon_deg - dlon).powi(2) + (lat_deg - dlat).powi(2)).sqrt();
        if d < radius {
            r = 250.0;
            g = 240.0 - 20.0 * d;
            b = 30.0;
        }
    }
    [r as u8, g as u8, b as u8]
}

/// Renders a Session of `width × height` Shots with a horizontal field of
/// view of `hfov_deg`, through `grid` and `perturbation`.
pub fn session(
    width: u32,
    height: u32,
    hfov_deg: f64,
    grid: &Grid,
    perturbation: &Perturbation,
) -> (MemorySource, Truth) {
    let f_px = Cam::f_for_hfov(f64::from(width), hfov_deg);
    let cam = Cam::new(f_px, width, height);
    let mut poses = Vec::new();
    let mut cells: Vec<(u32, u32, Rot)> = Vec::new();
    for row in 1..=grid.rows {
        let from_start = f64::from(row - 1) * perturbation.pitch_step;
        let pitch = match grid.start_pole {
            StartPole::Nadir => -90.0 + perturbation.start_pole_offset + from_start,
            StartPole::Zenith => 90.0 - perturbation.start_pole_offset - from_start,
        };
        let heading = perturbation
            .row_headings
            .get((row - 1) as usize)
            .copied()
            .unwrap_or(0.0);
        for col in 1..=grid.shots_per_row {
            let mut turns = f64::from(col - 1);
            if let Some((r, c)) = perturbation.dropped_step
                && r == row
                && col > c
            {
                turns -= 1.0;
            }
            let yaw = heading + turns * perturbation.turn_step;
            let p = pose(yaw.to_radians(), pitch.to_radians());
            poses.push(p);
            cells.push((row, col, p));
        }
    }
    let shots: Vec<(u32, u32, Frame)> = cells
        .par_iter()
        .map(|&(row, col, p)| {
            if perturbation.flat_shots.contains(&(row, col)) {
                return (row, col, Frame::solid(width, height, [150, 180, 230]));
            }
            let mut rgb = vec![0u8; (width * height * 3) as usize];
            for y in 0..height {
                for x in 0..width {
                    let d = cam.ray(&p, f64::from(x) + 0.5, f64::from(y) + 0.5);
                    let lon = d.x.atan2(d.z).to_degrees();
                    let lat = d.y.clamp(-1.0, 1.0).asin().to_degrees();
                    let px = world(lon, lat);
                    let i = ((y * width + x) * 3) as usize;
                    rgb[i..i + 3].copy_from_slice(&px);
                }
            }
            (row, col, Frame { width, height, rgb })
        })
        .collect();
    (MemorySource { shots }, Truth { f_px, poses })
}
