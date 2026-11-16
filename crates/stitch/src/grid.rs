//! The Session's grid, the starting guess of every pose (spec §16.6), and
//! the gimbal parameters the alignment refines: one pitch per Row, one yaw
//! per Shot, one focal length.

use panowww_core::game::{Game, StartPole};

use crate::ShotSource;
use crate::error::StitchError;
use crate::geom::{Rot, pose};
use crate::levels::Pyramids;

/// What the Session says about its own layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    /// Rows from pole to pole, poles included, at least 2.
    pub rows: u32,
    /// Shots in one full turn, at least 3.
    pub shots_per_row: u32,
    /// The pole Row 1 looks at.
    pub start_pole: StartPole,
}

impl Grid {
    pub fn from_game(game: &Game) -> Self {
        Self {
            rows: game.rows,
            shots_per_row: game.shots_per_row,
            start_pole: game.start_pole,
        }
    }

    /// Nominal pitch of a Row in radians: the poles at the ends, equal
    /// Steps between them.
    pub fn nominal_pitch(&self, row: u32) -> f64 {
        let step = std::f64::consts::PI / f64::from(self.rows - 1);
        let from_start = f64::from(row - 1) * step;
        match self.start_pole {
            StartPole::Nadir => -std::f64::consts::FRAC_PI_2 + from_start,
            StartPole::Zenith => std::f64::consts::FRAC_PI_2 - from_start,
        }
    }

    /// Nominal yaw of a column in radians: column 1 is the start heading.
    pub fn nominal_yaw(&self, col: u32) -> f64 {
        f64::from(col - 1) * std::f64::consts::TAU / f64::from(self.shots_per_row)
    }

    /// The Row nearest the horizon, 1-based.
    pub fn horizon_row(&self) -> u32 {
        (1..=self.rows)
            .min_by(|a, b| {
                self.nominal_pitch(*a)
                    .abs()
                    .total_cmp(&self.nominal_pitch(*b).abs())
            })
            .unwrap_or(1)
    }

    /// The source must hold every cell of the grid exactly once.
    pub(crate) fn check(&self, source: &dyn ShotSource) -> Result<(), StitchError> {
        if self.rows < 2 || self.shots_per_row < 3 {
            return Err(StitchError::Grid(format!(
                "A grid of {} Rows × {} Shots cannot be stitched.",
                self.rows, self.shots_per_row
            )));
        }
        let expected = (self.rows * self.shots_per_row) as usize;
        if source.count() != expected {
            return Err(StitchError::Grid(format!(
                "The Session holds {} Shots where its grid of {} Rows × {} needs {expected}.",
                source.count(),
                self.rows,
                self.shots_per_row
            )));
        }
        let mut seen = vec![false; expected];
        for index in 0..expected {
            let (row, col) = source.position(index);
            if row < 1 || row > self.rows || col < 1 || col > self.shots_per_row {
                return Err(StitchError::Grid(format!(
                    "Shot r{row:02}_c{col:02} is outside the grid."
                )));
            }
            let cell = ((row - 1) * self.shots_per_row + (col - 1)) as usize;
            if seen[cell] {
                return Err(StitchError::Grid(format!(
                    "Shot r{row:02}_c{col:02} appears twice."
                )));
            }
            seen[cell] = true;
        }
        Ok(())
    }
}

/// The gimbal model behind the grid (ADR 0002): angles in radians.
#[derive(Debug, Clone, PartialEq)]
pub struct GridParams {
    pub f_px: f64,
    /// One pitch per Row, index `row - 1`.
    pub pitch: Vec<f64>,
    /// One yaw per Shot, by Shot index.
    pub yaw: Vec<f64>,
    /// Row index (`row - 1`) of every Shot.
    pub row_of: Vec<usize>,
}

impl GridParams {
    /// The grid's own guess, with the Shots after a dropped Step pulled
    /// back by one Step each (`drops[i]` dropped Steps precede Shot `i` in
    /// its Row).
    pub fn nominal(pyramids: &Pyramids, grid: &Grid, drops: &[u32], f_px: f64) -> Self {
        let pitch = (1..=grid.rows).map(|r| grid.nominal_pitch(r)).collect();
        let yaw = (0..pyramids.count())
            .map(|i| {
                let (_, col) = pyramids.position(i);
                grid.nominal_yaw(col - drops[i])
            })
            .collect();
        let row_of = (0..pyramids.count())
            .map(|i| (pyramids.position(i).0 - 1) as usize)
            .collect();
        Self {
            f_px,
            pitch,
            yaw,
            row_of,
        }
    }

    pub fn pose(&self, i: usize) -> Rot {
        pose(self.yaw[i], self.pitch[self.row_of[i]])
    }

    pub fn poses(&self) -> Vec<Rot> {
        (0..self.yaw.len()).map(|i| self.pose(i)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nadir_grid_runs_from_minus_ninety_to_ninety() {
        let g = Grid {
            rows: 7,
            shots_per_row: 14,
            start_pole: StartPole::Nadir,
        };
        assert!((g.nominal_pitch(1).to_degrees() + 90.0).abs() < 1e-9);
        assert!((g.nominal_pitch(4).to_degrees()).abs() < 1e-9);
        assert!((g.nominal_pitch(7).to_degrees() - 90.0).abs() < 1e-9);
        assert_eq!(g.horizon_row(), 4);
        assert!((g.nominal_yaw(2).to_degrees() - 360.0 / 14.0).abs() < 1e-9);
    }

    #[test]
    fn zenith_grid_starts_at_the_top() {
        let g = Grid {
            rows: 6,
            shots_per_row: 13,
            start_pole: StartPole::Zenith,
        };
        assert!((g.nominal_pitch(1).to_degrees() - 90.0).abs() < 1e-9);
        assert!((g.nominal_pitch(6).to_degrees() + 90.0).abs() < 1e-9);
        assert!(g.horizon_row() == 3 || g.horizon_row() == 4);
    }
}
