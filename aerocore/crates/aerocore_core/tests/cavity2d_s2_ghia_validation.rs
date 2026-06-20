//! Sprint S2 — Quantitative validation of the 2D lid-driven cavity solver
//! against the reference data of Ghia, Ghia & Shin (1982) for Re = 100 and
//! Re = 400.
//!
//! # Benchmark description
//! The lid-driven cavity is a 2D square domain whose top wall moves at a
//! constant velocity U_lid.  The Reynolds number is defined as:
//!   Re = U_lid × L_fluid / ν
//! where L_fluid = nx − 2 is the number of interior nodes.
//!
//! # Validation strategy
//! 1. Run the `LbmSolver2D` solver on a **128 × 128** lattice for a fixed,
//!    deterministic number of steps.
//! 2. Extract the centreline velocity profiles:
//!    - u(x = 0.5, y)  — vertical centreline
//!    - v(x, y = 0.5)  — horizontal centreline
//! 3. Load reference data from the embedded Ghia 1982 CSV tables
//!    (normalised by U_lid and cavity length).
//! 4. For each reference point, linearly interpolate the solver profile at
//!    the same normalised coordinate, then normalise by U_lid.
//! 5. Compute RMSE and maximum absolute error against the Ghia reference.
//! 6. Assert that errors stay within the documented tolerances.
//!
//! # Step counts and tolerances
//! The step counts below are chosen to yield a well-developed flow.  A single
//! convective time unit is:
//!   T_conv = (nx − 2) / U_lid = 126 / 0.05 = 2 520 steps.
//!
//! - Re = 100: 20 000 steps ≈ 7.9 T_conv.  At this Re the primary vortex
//!   reaches quasi-steady state after ~5 T_conv, so the comparison is
//!   quantitatively meaningful.  Tolerance: RMSE ≤ 0.08, max-abs ≤ 0.15.
//!
//! - Re = 400: 40 000 steps ≈ 15.9 T_conv.  Convergence is slower and
//!   secondary eddies may still be evolving; tolerances are therefore wider:
//!   RMSE ≤ 0.12, max-abs ≤ 0.20.
//!
//! # CI performance note
//! Each test below is marked `#[ignore]` because the step counts above
//! require ~7 minutes in a debug build.  They are designed for release-mode
//! execution:
//!
//! ```text
//! cargo test --release -- --include-ignored
//! ```
//!
//! This produces results in under 30 seconds and is the recommended way to
//! run the full S2 validation suite in CI.  Standard `cargo test` (debug,
//! no `--include-ignored`) passes immediately because ignored tests are not
//! counted as failures.
//!
//! # Determinism
//! The solver contains no randomness; results are bit-identical across runs
//! given the same inputs (IEEE-754 double arithmetic, sequential loop).
//!
//! # References
//! - Ghia, Ghia & Shin (1982), J. Comput. Phys. 48(3), pp. 387–411

use aerocore_core::solvers::lbm::lbm2d::LbmSolver2D;

// ── Simulation parameters ─────────────────────────────────────────────────

const NX: usize = 128;
const NY: usize = 128;
const U_LID: f64 = 0.05;

// Steps per case.  Chosen so that:
//   Re=100 → ~7.9 convective time units  (quasi-steady)
//   Re=400 → ~15.9 convective time units (well-developed primary vortex)
const STEPS_RE100: usize = 20_000;
const STEPS_RE400: usize = 40_000;

// ── Error tolerances (in u/U_lid or v/U_lid units) ───────────────────────
//
// These values account for:
//   • LBM compressibility artefacts  (Ma ≈ 0.087)
//   • Incomplete convergence at finite step counts
//   • Minor grid-size difference vs Ghia's 129 × 129 Navier-Stokes solution
//
// Re = 100
const RMSE_TOL_RE100: f64 = 0.08; // root-mean-square error
const MAX_ABS_TOL_RE100: f64 = 0.15; // maximum absolute error

// Re = 400 (wider because convergence is slower at higher Re)
const RMSE_TOL_RE400: f64 = 0.12;
const MAX_ABS_TOL_RE400: f64 = 0.20;

// ── Embedded reference data ───────────────────────────────────────────────

const GHIA_RE100_U: &str = include_str!("data/ghia_re100_u_centerline.csv");
const GHIA_RE100_V: &str = include_str!("data/ghia_re100_v_centerline.csv");
const GHIA_RE400_U: &str = include_str!("data/ghia_re400_u_centerline.csv");
const GHIA_RE400_V: &str = include_str!("data/ghia_re400_v_centerline.csv");

// ── Helpers ───────────────────────────────────────────────────────────────

/// Parses a 2-column CSV (with optional `#`-comment header lines and a
/// named-column header) and returns a `Vec` of `(coord, value)` pairs.
///
/// Lines starting with `#` are silently skipped.  The first non-comment line
/// is treated as the column-name header and is also skipped.  All remaining
/// lines are split on `,` and parsed as `f64`.
fn parse_csv(src: &str) -> Vec<(f64, f64)> {
    let mut pairs = Vec::new();
    let mut header_seen = false;
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !header_seen {
            // Skip the column-name header (e.g. "y,u" or "x,v").
            header_seen = true;
            continue;
        }
        let mut parts = trimmed.splitn(2, ',');
        let coord: f64 = parts
            .next()
            .expect("missing first column")
            .trim()
            .parse()
            .expect("invalid float in first column");
        let val: f64 = parts
            .next()
            .expect("missing second column")
            .trim()
            .parse()
            .expect("invalid float in second column");
        pairs.push((coord, val));
    }
    pairs
}

/// Linearly interpolates `profile[0..n]` at a normalised coordinate `coord`
/// in [0, 1], where profile index `i` corresponds to `i / (n − 1)`.
fn interp_at(profile: &[f64], coord: f64) -> f64 {
    let n = profile.len();
    assert!(n >= 2, "profile must have at least 2 points");
    let idx_f = coord * (n - 1) as f64;
    let i0 = (idx_f.floor() as usize).min(n - 2);
    let i1 = i0 + 1;
    let t = idx_f - i0 as f64;
    (1.0 - t) * profile[i0] + t * profile[i1]
}

/// Root-mean-square error between two equal-length slices.
fn rmse(predicted: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(predicted.len(), reference.len());
    let n = predicted.len() as f64;
    let sum_sq: f64 = predicted
        .iter()
        .zip(reference.iter())
        .map(|(p, r)| (p - r) * (p - r))
        .sum();
    (sum_sq / n).sqrt()
}

/// Maximum absolute error between two equal-length slices.
fn max_abs_error(predicted: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(predicted.len(), reference.len());
    predicted
        .iter()
        .zip(reference.iter())
        .map(|(p, r)| (p - r).abs())
        .fold(0.0_f64, f64::max)
}

/// Samples the solver centreline profile at the normalised coordinates given
/// in `reference`, normalises by `u_lid`, and returns (solver_vals, ref_vals).
fn sample_profile(profile: &[f64], reference: &[(f64, f64)], u_lid: f64) -> (Vec<f64>, Vec<f64>) {
    let solver_vals: Vec<f64> = reference
        .iter()
        .map(|(coord, _)| interp_at(profile, *coord) / u_lid)
        .collect();
    let ref_vals: Vec<f64> = reference.iter().map(|(_, v)| *v).collect();
    (solver_vals, ref_vals)
}

// ── Test: Re = 100 ────────────────────────────────────────────────────────

/// Quantitative validation against Ghia 1982 Table I & II at Re = 100.
///
/// Marked `#[ignore]` because the 20 000-step run takes ~2 min in debug mode.
/// Run with:
///   cargo test --release -- --include-ignored cavity2d_s2_ghia_re100
#[test]
#[ignore = "slow: run with `cargo test --release -- --include-ignored`"]
fn cavity2d_s2_ghia_re100() {
    let re = 100.0_f64;
    let nu = U_LID * (NX - 2) as f64 / re;
    let mut solver = LbmSolver2D::new(NX, NY, nu);

    for _ in 0..STEPS_RE100 {
        solver.step(U_LID);
    }

    // ── u profile (vertical centreline, x = NX/2) ─────────────────────
    let ref_u = parse_csv(GHIA_RE100_U);
    let u_profile = solver.centerline_u();
    let (solver_u, ghia_u) = sample_profile(&u_profile, &ref_u, U_LID);

    let rmse_u = rmse(&solver_u, &ghia_u);
    let max_u = max_abs_error(&solver_u, &ghia_u);

    assert!(
        rmse_u <= RMSE_TOL_RE100,
        "Re=100 u-profile RMSE {rmse_u:.4} exceeds tolerance {RMSE_TOL_RE100:.4}\n\
         solver: {solver_u:.4?}\n\
         ghia:   {ghia_u:.4?}"
    );
    assert!(
        max_u <= MAX_ABS_TOL_RE100,
        "Re=100 u-profile max-abs {max_u:.4} exceeds tolerance {MAX_ABS_TOL_RE100:.4}"
    );

    // ── v profile (horizontal centreline, y = NY/2) ────────────────────
    let ref_v = parse_csv(GHIA_RE100_V);
    let v_profile = solver.centerline_v();
    let (solver_v, ghia_v) = sample_profile(&v_profile, &ref_v, U_LID);

    let rmse_v = rmse(&solver_v, &ghia_v);
    let max_v = max_abs_error(&solver_v, &ghia_v);

    assert!(
        rmse_v <= RMSE_TOL_RE100,
        "Re=100 v-profile RMSE {rmse_v:.4} exceeds tolerance {RMSE_TOL_RE100:.4}\n\
         solver: {solver_v:.4?}\n\
         ghia:   {ghia_v:.4?}"
    );
    assert!(
        max_v <= MAX_ABS_TOL_RE100,
        "Re=100 v-profile max-abs {max_v:.4} exceeds tolerance {MAX_ABS_TOL_RE100:.4}"
    );
}

// ── Test: Re = 400 ────────────────────────────────────────────────────────

/// Quantitative validation against Ghia 1982 Table I & II at Re = 400.
///
/// Marked `#[ignore]` because the 40 000-step run takes ~5 min in debug mode.
/// Run with:
///   cargo test --release -- --include-ignored cavity2d_s2_ghia_re400
#[test]
#[ignore = "slow: run with `cargo test --release -- --include-ignored`"]
fn cavity2d_s2_ghia_re400() {
    let re = 400.0_f64;
    let nu = U_LID * (NX - 2) as f64 / re;
    let mut solver = LbmSolver2D::new(NX, NY, nu);

    for _ in 0..STEPS_RE400 {
        solver.step(U_LID);
    }

    // ── u profile (vertical centreline, x = NX/2) ─────────────────────
    let ref_u = parse_csv(GHIA_RE400_U);
    let u_profile = solver.centerline_u();
    let (solver_u, ghia_u) = sample_profile(&u_profile, &ref_u, U_LID);

    let rmse_u = rmse(&solver_u, &ghia_u);
    let max_u = max_abs_error(&solver_u, &ghia_u);

    assert!(
        rmse_u <= RMSE_TOL_RE400,
        "Re=400 u-profile RMSE {rmse_u:.4} exceeds tolerance {RMSE_TOL_RE400:.4}\n\
         solver: {solver_u:.4?}\n\
         ghia:   {ghia_u:.4?}"
    );
    assert!(
        max_u <= MAX_ABS_TOL_RE400,
        "Re=400 u-profile max-abs {max_u:.4} exceeds tolerance {MAX_ABS_TOL_RE400:.4}"
    );

    // ── v profile (horizontal centreline, y = NY/2) ────────────────────
    let ref_v = parse_csv(GHIA_RE400_V);
    let v_profile = solver.centerline_v();
    let (solver_v, ghia_v) = sample_profile(&v_profile, &ref_v, U_LID);

    let rmse_v = rmse(&solver_v, &ghia_v);
    let max_v = max_abs_error(&solver_v, &ghia_v);

    assert!(
        rmse_v <= RMSE_TOL_RE400,
        "Re=400 v-profile RMSE {rmse_v:.4} exceeds tolerance {RMSE_TOL_RE400:.4}\n\
         solver: {solver_v:.4?}\n\
         ghia:   {ghia_v:.4?}"
    );
    assert!(
        max_v <= MAX_ABS_TOL_RE400,
        "Re=400 v-profile max-abs {max_v:.4} exceeds tolerance {MAX_ABS_TOL_RE400:.4}"
    );
}
