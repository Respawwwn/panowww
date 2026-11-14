//! The Align stage (spec §16.6 step 5): a bundle adjustment on the gimbal
//! model, one pitch per Row, one yaw per Shot, one focal length; then a
//! small free rotation per Shot. Levenberg–Marquardt on Huber-weighted
//! ray-space residuals, numeric Jacobian per match, dense normal
//! equations: a few hundred unknowns, milliseconds.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    reason = "least-squares code: h, g, j and r are the textbook names, the parameter columns are indexed by Shot, and each solver reads as one piece"
)]

use nalgebra::{DMatrix, DVector};
use rayon::prelude::*;

use crate::geom::{Cam, Rot, V3};
use crate::grid::GridParams;
use crate::matching::Corr;

/// Pixel error that counts as one sigma.
const SIGMA_PX: f64 = 4.0;
/// Residuals beyond this many sigmas are down-weighted (Huber).
const HUBER: f64 = 3.0;
/// Damping priors around the pass's starting estimate (ADR 0002).
const SIGMA_PITCH_DEG: f64 = 6.0;
const SIGMA_YAW_DEG: f64 = 3.0;
/// On pole Rows flat ground cannot pin a roll: the grid's turn holds.
const SIGMA_YAW_POLE_DEG: f64 = 0.3;
const POLE_DEG: f64 = 75.0;
/// The free per-Shot stage stays this close to the gimbal solution.
const SIGMA_FREE_DEG: f64 = 0.6;

/// How a least-squares pass went.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub rms_px: f64,
    pub cost_start: f64,
    pub cost_end: f64,
    pub iterations: usize,
}

fn huber_cost(r: f64) -> f64 {
    if r <= HUBER {
        0.5 * r * r
    } else {
        HUBER * (r - 0.5 * HUBER)
    }
}

fn huber_weight(r: f64) -> f64 {
    if r <= HUBER { 1.0 } else { (HUBER / r).sqrt() }
}

/// One Levenberg–Marquardt loop over a parameter vector, given the per-match
/// Jacobian blocks and the prior terms. `eval(delta)` applies a step and
/// returns the new cost; `commit()` keeps the last evaluated step.
struct Lm {
    lambda: f64,
    iterations: usize,
    converged: bool,
}

impl Lm {
    fn new() -> Self {
        Self {
            lambda: 1e-3,
            iterations: 0,
            converged: false,
        }
    }

    /// Solves `(H + λ diag H) δ = −g`, trying larger damping until a step
    /// lowers the cost; returns the accepted step or `None` when none does.
    fn step(
        &mut self,
        h: &DMatrix<f64>,
        g: &DVector<f64>,
        cost: f64,
        eval: &mut dyn FnMut(&DVector<f64>) -> f64,
    ) -> Option<(DVector<f64>, f64)> {
        let m = h.nrows();
        for _ in 0..8 {
            let mut hl = h.clone();
            for d in 0..m {
                hl[(d, d)] += self.lambda * (h[(d, d)].abs() + 1e-9);
            }
            let Some(chol) = hl.cholesky() else {
                self.lambda *= 10.0;
                continue;
            };
            let delta = chol.solve(&(-g));
            let new_cost = eval(&delta);
            if new_cost < cost {
                let gain = cost - new_cost;
                self.lambda = (self.lambda / 3.0).max(1e-7);
                self.iterations += 1;
                if gain < 1e-4 * cost {
                    self.converged = true;
                }
                return Some((delta, new_cost));
            }
            self.lambda *= 5.0;
        }
        None
    }
}

/// Accumulates one match's Jacobian block into the normal equations.
fn accumulate(
    h: &mut DMatrix<f64>,
    g: &mut DVector<f64>,
    cols: &[usize],
    j: &DMatrix<f64>,
    r: &V3,
) {
    let jtj = j.transpose() * j;
    let jtr = j.transpose() * r;
    for (p, &cp) in cols.iter().enumerate() {
        g[cp] += jtr[p];
        for (q, &cq) in cols.iter().enumerate() {
            h[(cp, cq)] += jtj[(p, q)];
        }
    }
}

/// Refines `params` (pitch per Row, yaw per Shot, focal length) on `corrs`,
/// the anchor Shot's yaw fixed. The prior damps around the incoming
/// estimate, never towards the raw grid.
pub fn bundle_adjust(
    params: &mut GridParams,
    corrs: &[Corr],
    anchor: usize,
    width: u32,
    height: u32,
) -> Fit {
    let n = params.yaw.len();
    let rows = params.pitch.len();
    let sigma_pitch = SIGMA_PITCH_DEG.to_radians();
    let pole: Vec<bool> = params
        .pitch
        .iter()
        .map(|p| p.abs() > POLE_DEG.to_radians())
        .collect();
    let sigma_yaw: Vec<f64> = (0..n)
        .map(|i| {
            if pole[params.row_of[i]] {
                SIGMA_YAW_POLE_DEG
            } else {
                SIGMA_YAW_DEG
            }
            .to_radians()
        })
        .collect();
    // Layout: 0 = f, 1..=rows = pitch per Row, then one yaw per Shot but the anchor.
    let pitch_col = |r: usize| 1 + r;
    let yaw_col: Vec<Option<usize>> = {
        let mut k = 1 + rows;
        (0..n)
            .map(|i| {
                if i == anchor {
                    None
                } else {
                    k += 1;
                    Some(k - 1)
                }
            })
            .collect()
    };
    let m = 1 + rows + (n - 1);
    let nominal = params.clone();
    let residual = |g: &GridParams, c: &Corr| -> V3 {
        let cam = Cam::new(g.f_px, width, height);
        (cam.ray(&g.pose(c.ia), c.a.0, c.a.1) - cam.ray(&g.pose(c.ib), c.b.0, c.b.1)) * cam.f
            / SIGMA_PX
    };
    let total_cost = |g: &GridParams| -> f64 {
        let data: f64 = corrs
            .iter()
            .map(|c| huber_cost(residual(g, c).norm()))
            .sum();
        let prior_pitch: f64 = (0..rows)
            .map(|r| 0.5 * ((g.pitch[r] - nominal.pitch[r]) / sigma_pitch).powi(2))
            .sum();
        let prior_yaw: f64 = (0..n)
            .map(|i| 0.5 * ((g.yaw[i] - nominal.yaw[i]) / sigma_yaw[i]).powi(2))
            .sum();
        data + prior_pitch + prior_yaw
    };
    let apply = |g: &GridParams, delta: &DVector<f64>| -> GridParams {
        let mut gn = g.clone();
        gn.f_px += delta[0];
        for r in 0..rows {
            gn.pitch[r] += delta[pitch_col(r)];
        }
        for i in 0..n {
            if let Some(col) = yaw_col[i] {
                gn.yaw[i] += delta[col];
            }
        }
        gn
    };
    let cost_start = total_cost(params);
    let mut cost = cost_start;
    let mut lm = Lm::new();
    let (eps_a, eps_f) = (1e-5, 2e-2);
    for _ in 0..30 {
        let g_now = params.clone();
        let mut h = DMatrix::<f64>::zeros(m, m);
        let mut grad = DVector::<f64>::zeros(m);
        let blocks: Vec<(Vec<usize>, DMatrix<f64>, V3)> = corrs
            .par_iter()
            .map(|c| {
                let r0 = residual(&g_now, c);
                let w = huber_weight(r0.norm());
                let mut cols = vec![0usize];
                let mut jac: Vec<V3> = Vec::new();
                let mut gf = g_now.clone();
                gf.f_px += eps_f;
                jac.push((residual(&gf, c) - r0) / eps_f);
                let mut touched = vec![g_now.row_of[c.ia]];
                if g_now.row_of[c.ib] != g_now.row_of[c.ia] {
                    touched.push(g_now.row_of[c.ib]);
                }
                for r in touched {
                    let mut gp = g_now.clone();
                    gp.pitch[r] += eps_a;
                    jac.push((residual(&gp, c) - r0) / eps_a);
                    cols.push(pitch_col(r));
                }
                for idx in [c.ia, c.ib] {
                    if let Some(col) = yaw_col[idx] {
                        let mut gy = g_now.clone();
                        gy.yaw[idx] += eps_a;
                        jac.push((residual(&gy, c) - r0) / eps_a);
                        cols.push(col);
                    }
                }
                let mut j = DMatrix::<f64>::zeros(3, cols.len());
                for (q, col) in jac.iter().enumerate() {
                    j.set_column(q, &(col * w));
                }
                (cols, j, r0 * w)
            })
            .collect();
        for (cols, j, r) in &blocks {
            accumulate(&mut h, &mut grad, cols, j, r);
        }
        for r in 0..rows {
            h[(pitch_col(r), pitch_col(r))] += 1.0 / (sigma_pitch * sigma_pitch);
            grad[pitch_col(r)] += (g_now.pitch[r] - nominal.pitch[r]) / (sigma_pitch * sigma_pitch);
        }
        for i in 0..n {
            if let Some(col) = yaw_col[i] {
                let s = sigma_yaw[i];
                h[(col, col)] += 1.0 / (s * s);
                grad[col] += (g_now.yaw[i] - nominal.yaw[i]) / (s * s);
            }
        }
        let mut candidate: Option<GridParams> = None;
        let mut eval = |delta: &DVector<f64>| -> f64 {
            let gn = apply(&g_now, delta);
            let c = total_cost(&gn);
            candidate = Some(gn);
            c
        };
        match lm.step(&h, &grad, cost, &mut eval) {
            Some((_, new_cost)) => {
                *params = candidate.take().expect("a step was evaluated");
                cost = new_cost;
            }
            None => break,
        }
        if lm.converged {
            break;
        }
    }
    let cam = Cam::new(params.f_px, width, height);
    Fit {
        rms_px: crate::matching::residual_px(corrs, &params.poses(), &cam),
        cost_start,
        cost_end: cost,
        iterations: lm.iterations,
    }
}

/// A small free rotation per Shot on top of the gimbal solution, the
/// anchor fixed, the focal length fixed: absorbs what the gimbal model
/// cannot, within a tight prior.
pub fn refine_shots(poses: &mut [Rot], corrs: &[Corr], anchor: usize, cam: &Cam) -> Fit {
    let n = poses.len();
    let sigma = SIGMA_FREE_DEG.to_radians();
    let col_of: Vec<Option<usize>> = {
        let mut k = 0;
        (0..n)
            .map(|i| {
                if i == anchor {
                    None
                } else {
                    k += 3;
                    Some(k - 3)
                }
            })
            .collect()
    };
    let m = 3 * (n - 1);
    let base: Vec<Rot> = poses.to_vec();
    let residual = |poses: &[Rot], c: &Corr| -> V3 {
        (cam.ray(&poses[c.ia], c.a.0, c.a.1) - cam.ray(&poses[c.ib], c.b.0, c.b.1)) * cam.f
            / SIGMA_PX
    };
    let total_cost = |poses: &[Rot]| -> f64 {
        let data: f64 = corrs
            .iter()
            .map(|c| huber_cost(residual(poses, c).norm()))
            .sum();
        let prior: f64 = (0..n)
            .map(|i| {
                0.5 * (poses[i] * base[i].inverse()).scaled_axis().norm_squared() / (sigma * sigma)
            })
            .sum();
        data + prior
    };
    let apply = |poses: &[Rot], delta: &DVector<f64>| -> Vec<Rot> {
        (0..n)
            .map(|i| match col_of[i] {
                Some(c0) => {
                    Rot::from_scaled_axis(V3::new(delta[c0], delta[c0 + 1], delta[c0 + 2]))
                        * poses[i]
                }
                None => poses[i],
            })
            .collect()
    };
    let cost_start = total_cost(poses);
    let mut cost = cost_start;
    let mut lm = Lm::new();
    let eps = 1e-5;
    for _ in 0..20 {
        let now: Vec<Rot> = poses.to_vec();
        let mut h = DMatrix::<f64>::zeros(m, m);
        let mut grad = DVector::<f64>::zeros(m);
        let blocks: Vec<(Vec<usize>, DMatrix<f64>, V3)> = corrs
            .par_iter()
            .map(|c| {
                let r0 = residual(&now, c);
                let w = huber_weight(r0.norm());
                let mut cols = Vec::new();
                let mut jac: Vec<V3> = Vec::new();
                for idx in [c.ia, c.ib] {
                    if let Some(c0) = col_of[idx] {
                        for k in 0..3 {
                            let mut om = V3::zeros();
                            om[k] = eps;
                            let rp = Rot::from_scaled_axis(om) * now[idx];
                            let r1 = if idx == c.ia {
                                (cam.ray(&rp, c.a.0, c.a.1) - cam.ray(&now[c.ib], c.b.0, c.b.1))
                                    * cam.f
                                    / SIGMA_PX
                            } else {
                                (cam.ray(&now[c.ia], c.a.0, c.a.1) - cam.ray(&rp, c.b.0, c.b.1))
                                    * cam.f
                                    / SIGMA_PX
                            };
                            jac.push((r1 - r0) / eps);
                            cols.push(c0 + k);
                        }
                    }
                }
                let mut j = DMatrix::<f64>::zeros(3, cols.len());
                for (q, col) in jac.iter().enumerate() {
                    j.set_column(q, &(col * w));
                }
                (cols, j, r0 * w)
            })
            .collect();
        for (cols, j, r) in &blocks {
            accumulate(&mut h, &mut grad, cols, j, r);
        }
        for i in 0..n {
            if let Some(c0) = col_of[i] {
                let dev = (now[i] * base[i].inverse()).scaled_axis();
                for k in 0..3 {
                    h[(c0 + k, c0 + k)] += 1.0 / (sigma * sigma);
                    grad[c0 + k] += dev[k] / (sigma * sigma);
                }
            }
        }
        let mut candidate: Option<Vec<Rot>> = None;
        let mut eval = |delta: &DVector<f64>| -> f64 {
            let next = apply(&now, delta);
            let c = total_cost(&next);
            candidate = Some(next);
            c
        };
        match lm.step(&h, &grad, cost, &mut eval) {
            Some((_, new_cost)) => {
                poses.copy_from_slice(&candidate.take().expect("a step was evaluated"));
                cost = new_cost;
            }
            None => break,
        }
        if lm.converged {
            break;
        }
    }
    Fit {
        rms_px: crate::matching::residual_px(corrs, poses, cam),
        cost_start,
        cost_end: cost,
        iterations: lm.iterations,
    }
}
