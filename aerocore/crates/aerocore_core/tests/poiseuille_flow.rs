//! Poiseuille flow integration test — analytical validation of the LBM solver.
//!
//! # Physics
//! Poiseuille flow is the steady laminar flow of a viscous fluid between two
//! infinite parallel plates driven by a constant body force (pressure gradient).
//!
//! The analytical solution is a parabolic velocity profile:
//!
//!   u(y) = (F / (2ν)) × y × (H - y)
//!
//! where:
//!   F = body force (pressure gradient substitute in LBM)
//!   ν = kinematic viscosity (lattice units)
//!   H = channel height in lattice units
//!   y = distance from bottom wall
//!
//! # Validation
//! After sufficient iterations to reach steady state, the maximum relative
//! error between the LBM velocity profile and the analytical solution must be
//! below 10% (the half-way bounce-back introduces an O(Δx²) wall-position
//! offset that causes ~5% error on coarse grids).
//!
//! # References
//! - Krüger et al. (2017), "The Lattice Boltzmann Method", §5.2.1
//! - Succi (2018), "The Lattice Boltzmann Equation", §8.3

use aerocore_core::memory::arena::SimArena;
use aerocore_core::solvers::lbm::LbmSolver;
use aerocore_core::solvers::traits::{Solver, SolverConfig};

/// Number of lattice units in the x-direction (periodic: length doesn't matter).
const NX: usize = 4;
/// Number of lattice units in the y-direction (channel height, including walls).
const NY: usize = 12;
/// Number of lattice units in the z-direction.
const NZ: usize = 4;

/// Kinematic viscosity in lattice units.
const VISCOSITY: f64 = 0.1;

/// Number of iterations to reach a well-converged steady state.
const ITERATIONS: u64 = 2_000;

/// Body force applied in the x-direction to drive the flow.
/// Must be small enough to satisfy the low-Mach approximation (Ma << 1).
const BODY_FORCE: f64 = 1e-5;

#[test]
fn poiseuille_flow_analytical_validation() {
    // ── set up arena and solver ───────────────────────────────────────────────
    // 20 f64 fields × NX×NY×NZ cells × 8 bytes, with generous headroom
    let arena_bytes = 32 * NX * NY * NZ * std::mem::size_of::<f64>() * 20;
    let arena = SimArena::new(arena_bytes);

    let config = SolverConfig {
        max_iterations: ITERATIONS,
        dt: 1.0, // lattice time step = 1 (LBM convention)
        ..Default::default()
    };

    let mut solver: LbmSolver<f64> = LbmSolver::new(NX, NY, NZ, VISCOSITY);

    solver.init(&config, &arena).expect("LBM init failed");

    // ── run iterations ────────────────────────────────────────────────────────
    for _ in 0..ITERATIONS {
        solver
            .step_with_body_force(BODY_FORCE, 0.0, 0.0)
            .expect("LBM step failed");
    }

    // ── extract velocity profile ──────────────────────────────────────────────
    let ux_profile = solver.ux_profile_y(NX / 2, NZ / 2);

    // ── analytical solution ───────────────────────────────────────────────────
    // With the half-way bounce-back scheme the effective no-slip wall is placed
    // at y = -0.5 and y = NY - 0.5.  The exact Poiseuille profile is therefore:
    //
    //   u_x(y) = F/(2ν) × (y + 0.5) × (NY - 0.5 - y)
    //
    // Reference: Krüger et al. (2017), Eq. 5.37
    let nu = VISCOSITY;
    let f = BODY_FORCE;
    let ny_f = NY as f64;

    // Maximum relative error over all interior nodes
    // Use a generous tolerance: the half-way bounce-back shifts the effective
    // wall position by 0.5 lattice units, so small domains show ~5% error.
    let tolerance = 0.10; // 10%
    let mut max_rel_err = 0.0_f64;
    for (j, &u_lbm) in ux_profile.iter().enumerate().skip(1).take(NY - 2) {
        let y = j as f64;
        // Half-way bounce-back analytical formula
        let u_analytical = (f / (2.0 * nu)) * (y + 0.5) * (ny_f - 0.5 - y);

        if u_analytical.abs() > 1e-15 {
            let rel_err = ((u_lbm - u_analytical) / u_analytical).abs();
            if rel_err > max_rel_err {
                max_rel_err = rel_err;
            }
        }
    }

    assert!(
        max_rel_err < tolerance,
        "Poiseuille flow: max relative error {:.4} exceeds tolerance {:.4}.\n\
         LBM profile:        {:?}\n",
        max_rel_err,
        tolerance,
        &ux_profile[1..NY - 1],
    );
}
