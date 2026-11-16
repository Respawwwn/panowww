//! Dropped Steps (spec §16.6, ADR 0002): two consecutive Shots of a Row
//! that are the same image mean the camera did not move for that Step.
//! Every later Shot of the Row is one Step behind the grid's plan.

#![allow(clippy::cast_precision_loss, reason = "pixel sums become means")]

use crate::levels::{Level, Pyramids};

/// Normalised correlation at zero shift above which two Shots are the
/// same image.
const SAME_IMAGE: f32 = 0.985;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dropped {
    /// Per Shot, how many dropped Steps precede it in its Row.
    pub drops: Vec<u32>,
    /// `(twin, duplicate)` Shot indices, the duplicate being the later one.
    pub pairs: Vec<(usize, usize)>,
}

/// Compares every Shot with its left neighbour on the 1/8 copies.
pub fn detect(pyramids: &Pyramids) -> Dropped {
    let mut drops = vec![0u32; pyramids.count()];
    let mut pairs = Vec::new();
    let rows = pyramids.shots().iter().map(|s| s.row).max().unwrap_or(0);
    let cols = pyramids.shots().iter().map(|s| s.col).max().unwrap_or(0);
    for row in 1..=rows {
        let mut count = 0;
        for col in 2..=cols {
            let (Some(i), Some(j)) = (pyramids.index_of(row, col - 1), pyramids.index_of(row, col))
            else {
                continue;
            };
            if same_image(&pyramids.shot(i).eighth, &pyramids.shot(j).eighth) {
                count += 1;
                pairs.push((i, j));
            }
            drops[j] = count;
        }
    }
    Dropped { drops, pairs }
}

fn same_image(a: &Level, b: &Level) -> bool {
    if a.g.len() != b.g.len() || a.g.is_empty() {
        return false;
    }
    let n = a.g.len() as f32;
    let (ma, mb) = (
        a.g.iter().map(|&v| f32::from(v)).sum::<f32>() / n,
        b.g.iter().map(|&v| f32::from(v)).sum::<f32>() / n,
    );
    let (mut cov, mut va, mut vb) = (0f32, 0f32, 0f32);
    for k in 0..a.g.len() {
        let (da, db) = (f32::from(a.g[k]) - ma, f32::from(b.g[k]) - mb);
        cov += da * db;
        va += da * da;
        vb += db * db;
    }
    va > 0.0 && vb > 0.0 && cov / (va * vb).sqrt() > SAME_IMAGE
}
