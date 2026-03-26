//! Sprint S1 — 2D Lid-Driven Cavity validation test (D2Q9 LBM).
//!
//! # Physics
//! The lid-driven cavity is a canonical CFD benchmark: a square cavity whose
//! top wall moves at a prescribed velocity U_lid while all other walls are
//! stationary no-slip surfaces.  At low Reynolds numbers (Re ≲ 400) a single
//! primary vortex occupies most of the cavity.
//!
//! # Validation checks
//! 1. **Energy build-up**: total kinetic energy rises from zero and is positive
//!    after convergence.
//! 2. **Mass conservation**: total density stays within 0.1% of its initial
//!    value throughout the run.
//! 3. **Primary vortex (qualitative)**: the u-velocity along the vertical
//!    centreline (x = nx/2) must change sign — positive near the lid
//!    (dragged by the moving wall) and negative somewhere below (return flow).
//!    This confirms that a recirculating primary vortex has formed.
//! 4. **Lid speed satisfied**: the u-velocity at the node just below the lid
//!    has the correct sign and a non-negligible magnitude proportional to U_lid.
//!
//! # Determinism
//! The solver contains no randomness and operates only on IEEE-754 double
//! arithmetic in a purely sequential loop, guaranteeing bit-identical results
//! across platforms given the same inputs.
//!
//! # References
//! - Ghia, Ghia & Shin (1982), J. Comp. Phys. 48(3), pp. 387–411
//! - Krüger et al. (2017), "The Lattice Boltzmann Method", §8.4

use aerocore_core::solvers::lbm::lbm2d::LbmSolver2D;

/// Cavity side length in lattice units (includes boundary/wall nodes).
/// N=32 keeps CI run-time short while still resolving the primary vortex.
const N: usize = 32;

/// Lid velocity (x-direction). Ma = U_lid / cs ≈ 0.087 ≪ 1 ✓
const U_LID: f64 = 0.05;

/// Re = U_lid × (N − 2) / ν ≈ 100
const VISCOSITY: f64 = 0.01450; // ≈ U_LID * (N-2) / 100

/// Number of steps — large enough for the flow to reach a quasi-steady state.
const STEPS: u64 = 8_000;

/// Mass-conservation tolerance (relative).
const MASS_TOL: f64 = 0.001; // 0.1%

#[test]
fn cavity2d_s1_energy_and_mass() {
    let mut solver = LbmSolver2D::new(N, N, VISCOSITY);
    let initial_mass = solver.total_density();

    let mut ke_early = 0.0_f64;
    let mut ke_late = 0.0_f64;

    for step in 1..=STEPS {
        solver.step(U_LID);
        if step == 200 {
            ke_early = solver.total_kinetic_energy();
        }
        if step == STEPS {
            ke_late = solver.total_kinetic_energy();
        }
    }

    // ── Check 1: positive kinetic energy builds up ────────────────────────
    assert!(
        ke_early > 0.0,
        "Expected positive KE after 200 steps, got {ke_early}"
    );
    assert!(
        ke_late > ke_early,
        "Expected KE to grow from early ({ke_early:.4e}) to late ({ke_late:.4e})"
    );

    // ── Check 2: mass conservation ────────────────────────────────────────
    let final_mass = solver.total_density();
    let mass_err = ((final_mass - initial_mass) / initial_mass).abs();
    assert!(
        mass_err < MASS_TOL,
        "Mass conservation violated: initial={initial_mass:.6}, final={final_mass:.6}, rel_err={mass_err:.2e}"
    );
}

#[test]
fn cavity2d_s1_primary_vortex() {
    let mut solver = LbmSolver2D::new(N, N, VISCOSITY);

    for _ in 0..STEPS {
        solver.step(U_LID);
    }

    // ── Check 3: primary vortex (sign change in vertical centreline) ──────
    // The u-velocity along the vertical centreline (x = N/2) should be:
    //   - Positive near the lid (y ≈ N−1): fluid dragged in +x direction.
    //   - Negative somewhere in the interior: return flow of the vortex.
    let u_profile = solver.centerline_u();

    // Skip the boundary nodes (y=0 and y=N-1).
    let interior = &u_profile[1..N - 1];

    let u_near_lid = interior[interior.len() - 1]; // node just below lid
    let has_positive = interior.iter().any(|&u| u > 0.0);
    let has_negative = interior.iter().any(|&u| u < 0.0);

    assert!(
        u_near_lid > 0.0,
        "Node just below lid should have u > 0 (lid drags fluid); got {u_near_lid:.4e}"
    );
    assert!(
        has_positive,
        "Vertical centreline u-profile has no positive values — vortex not formed?\n{interior:?}"
    );
    assert!(
        has_negative,
        "Vertical centreline u-profile has no negative values — sign change expected for Re≈100\n{interior:?}"
    );

    // ── Check 4: lid velocity fraction ───────────────────────────────────
    // The node just below the lid should feel a significant fraction of U_lid.
    assert!(
        u_near_lid > 0.1 * U_LID,
        "u just below lid ({u_near_lid:.4e}) is less than 10% of U_lid ({U_LID:.4e})"
    );

    // ── Check 5: mass conservation (duplicate guard) ──────────────────────
    let n_cells = N * N;
    let mass: f64 = solver.rho.iter().sum();
    let expected = n_cells as f64; // ρ₀ = 1 everywhere → total = N²
    let mass_err = ((mass - expected) / expected).abs();
    assert!(
        mass_err < MASS_TOL,
        "Mass drift after vortex check: {mass_err:.2e} > {MASS_TOL}"
    );
}

#[test]
fn cavity2d_s1_csv_output() {
    // Smoke test: run a tiny simulation and verify CSV files are created and
    // contain the expected header and a non-trivial number of rows.
    let n = 8_usize;
    let mut solver = LbmSolver2D::new(n, n, 0.1);
    for _ in 0..100 {
        solver.step(0.05);
    }

    let dir = std::env::temp_dir().join("cavity2d_s1_csv_test");
    std::fs::create_dir_all(&dir).expect("Cannot create temp dir");

    let vel_path = dir.join("velocity.csv");
    let cl_path = dir.join("centerline.csv");

    solver
        .write_velocity_csv(&vel_path)
        .expect("write_velocity_csv failed");
    solver
        .write_centerline_csv(&cl_path)
        .expect("write_centerline_csv failed");

    // Velocity CSV: header + n*n data rows.
    let vel_content = std::fs::read_to_string(&vel_path).expect("Cannot read velocity.csv");
    let vel_lines: Vec<&str> = vel_content.lines().collect();
    assert_eq!(
        vel_lines[0], "x,y,rho,ux,uy",
        "Unexpected velocity CSV header"
    );
    assert_eq!(
        vel_lines.len(),
        n * n + 1,
        "Expected {} data rows + 1 header in velocity CSV",
        n * n
    );

    // Centreline CSV: header + max(nx, ny) rows.
    let cl_content = std::fs::read_to_string(&cl_path).expect("Cannot read centerline.csv");
    let cl_lines: Vec<&str> = cl_content.lines().collect();
    assert_eq!(
        cl_lines[0], "index,u_vert_centerline,v_horiz_centerline",
        "Unexpected centreline CSV header"
    );
    assert_eq!(
        cl_lines.len(),
        n + 1,
        "Expected {n} data rows + 1 header in centreline CSV"
    );
}
