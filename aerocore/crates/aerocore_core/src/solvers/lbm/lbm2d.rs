//! 2D Lattice Boltzmann solver using the D2Q9 velocity model.
//!
//! Implements a lid-driven cavity simulation with:
//! - BGK (single-relaxation-time) collision operator
//! - Moving lid at the top wall: u = (U_lid, 0)
//! - Half-way bounce-back no-slip on all other walls (left, right, bottom)
//!
//! # Memory Layout
//! Distribution functions are stored in SoA (Structure of Arrays) order:
//!   `f[i * nx * ny + y * nx + x]`
//! where i ∈ [0, Q), x ∈ [0, nx), y ∈ [0, ny).
//!
//! # References
//! - Krüger et al. (2017), "The Lattice Boltzmann Method", §3 and §8.4
//! - Ladd (1994), J. Fluid Mech. 271 (momentum exchange correction)
//! - Ghia, Ghia & Shin (1982), J. Comp. Phys. 48(3), pp. 387–411

use super::d2q9;

/// 2D Lattice Boltzmann solver (D2Q9, BGK, lid-driven cavity).
pub struct LbmSolver2D {
    /// Grid width (lattice units, includes boundary nodes).
    pub nx: usize,
    /// Grid height (lattice units, includes boundary nodes).
    pub ny: usize,
    /// BGK relaxation frequency: ω = 1/τ where τ = 3ν + 0.5.
    omega: f64,
    /// Distribution functions f[i * nx*ny + y*nx + x].
    f: Vec<f64>,
    /// Temporary buffer used during streaming.
    f_tmp: Vec<f64>,
    /// Macroscopic density ρ[y*nx + x].
    pub rho: Vec<f64>,
    /// Macroscopic x-velocity [y*nx + x].
    pub ux: Vec<f64>,
    /// Macroscopic y-velocity [y*nx + x].
    pub uy: Vec<f64>,
    /// Current time step counter.
    pub timestep: u64,
}

impl LbmSolver2D {
    /// Creates a new solver for an `nx × ny` lattice with kinematic viscosity `nu`.
    ///
    /// # Parameters
    /// - `nx`, `ny`: grid dimensions including wall nodes.
    /// - `nu`: kinematic viscosity in lattice units (e.g. `U * (nx-2) / Re`).
    ///
    /// The solver is immediately ready to step; no separate `init()` is required.
    pub fn new(nx: usize, ny: usize, nu: f64) -> Self {
        let tau = 3.0 * nu + 0.5;
        let omega = 1.0 / tau;
        let n = nx * ny;
        // Initialise distributions to rest equilibrium: f_i = w_i (ρ=1, u=0).
        let mut f = vec![0.0_f64; d2q9::Q * n];
        for i in 0..d2q9::Q {
            let w = d2q9::W[i];
            for cell in 0..n {
                f[i * n + cell] = w;
            }
        }
        Self {
            nx,
            ny,
            omega,
            f,
            f_tmp: vec![0.0_f64; d2q9::Q * n],
            rho: vec![1.0_f64; n],
            ux: vec![0.0_f64; n],
            uy: vec![0.0_f64; n],
            timestep: 0,
        }
    }

    /// Flat cell index from (x, y) coordinates.
    #[inline(always)]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.nx + x
    }

    /// Performs one BGK collision + streaming step with lid-driven cavity BCs.
    ///
    /// # Boundary conditions
    /// - **Top wall** (y = ny−1): moving lid with x-velocity `u_lid`.
    ///   Applied via the Ladd momentum-exchange correction:
    ///   `f_opp[src] += f_post − 2 wᵢ ρ (eᵢ · u_wall) / cs²`
    /// - **Left, right, bottom walls**: half-way bounce-back (no-slip).
    pub fn step(&mut self, u_lid: f64) {
        let nx = self.nx;
        let ny = self.ny;
        let n = nx * ny;
        let omega = self.omega;
        // Pre-computed constants (lattice cs² = 1/3)
        const INV_CS2: f64 = 3.0; // 1/cs²
        const INV_2CS4: f64 = 4.5; // 1/(2 cs⁴) = 9/2
        const INV_2CS2: f64 = 1.5; // 1/(2 cs²) = 3/2

        // Clear streaming buffer.
        for v in self.f_tmp.iter_mut() {
            *v = 0.0;
        }

        for y in 0..ny {
            for x in 0..nx {
                let idx = self.idx(x, y);

                // ── 1. Macroscopic quantities ──────────────────────────────
                let mut rho = 0.0_f64;
                let mut ux = 0.0_f64;
                let mut uy = 0.0_f64;
                for i in 0..d2q9::Q {
                    let fi = self.f[i * n + idx];
                    rho += fi;
                    ux += d2q9::E[i][0] as f64 * fi;
                    uy += d2q9::E[i][1] as f64 * fi;
                }
                if rho > 0.0 {
                    ux /= rho;
                    uy /= rho;
                }
                self.rho[idx] = rho;
                self.ux[idx] = ux;
                self.uy[idx] = uy;

                let u_sq = ux * ux + uy * uy;

                // ── 2. BGK collision + streaming for each direction ────────
                for i in 0..d2q9::Q {
                    let ei_x = d2q9::E[i][0] as f64;
                    let ei_y = d2q9::E[i][1] as f64;
                    let wi = d2q9::W[i];
                    let ei_dot_u = ei_x * ux + ei_y * uy;

                    // BGK equilibrium distribution
                    let feq = wi
                        * rho
                        * (1.0 + ei_dot_u * INV_CS2 + ei_dot_u * ei_dot_u * INV_2CS4
                            - u_sq * INV_2CS2);

                    let f_post = self.f[i * n + idx] - omega * (self.f[i * n + idx] - feq);

                    // Streaming destination
                    let nx_new = x as i32 + d2q9::E[i][0];
                    let ny_new = y as i32 + d2q9::E[i][1];

                    if nx_new < 0 || nx_new >= nx as i32 {
                        // Left / right wall: half-way bounce-back (no-slip).
                        let opp = d2q9::OPPOSITE[i];
                        self.f_tmp[opp * n + idx] += f_post;
                    } else if ny_new < 0 {
                        // Bottom wall: half-way bounce-back (no-slip).
                        let opp = d2q9::OPPOSITE[i];
                        self.f_tmp[opp * n + idx] += f_post;
                    } else if ny_new >= ny as i32 {
                        // Top wall: moving lid.
                        // Ladd momentum-exchange correction (u_wall = (u_lid, 0)).
                        //
                        // Corner treatment: at (x=0, going right-up) or
                        // (x=nx-1, going left-up) the diagonal direction is
                        // asymmetric — the opposite lateral direction was already
                        // bounced back by the side wall.  Applying the lid
                        // correction only at one corner breaks mass conservation.
                        // Solution: use plain bounce-back at these corner diagonals.
                        let opp = d2q9::OPPOSITE[i];
                        let is_corner_diag =
                            (x == 0 && d2q9::E[i][0] == 1) || (x == nx - 1 && d2q9::E[i][0] == -1);
                        if is_corner_diag {
                            // Pure bounce-back: symmetric with the opposite corner diagonal.
                            self.f_tmp[opp * n + idx] += f_post;
                        } else {
                            let correction = 2.0 * wi * rho * ei_x * u_lid * INV_CS2;
                            self.f_tmp[opp * n + idx] += f_post - correction;
                        }
                    } else {
                        let target = self.idx(nx_new as usize, ny_new as usize);
                        self.f_tmp[i * n + target] += f_post;
                    }
                }
            }
        }

        std::mem::swap(&mut self.f, &mut self.f_tmp);
        self.timestep += 1;
    }

    // ── Analysis helpers ─────────────────────────────────────────────────────

    /// Sum of all cell densities (mass proxy).
    pub fn total_density(&self) -> f64 {
        self.rho.iter().sum()
    }

    /// Total kinetic energy: Σ ½ ρ (ux² + uy²).
    pub fn total_kinetic_energy(&self) -> f64 {
        let n = self.nx * self.ny;
        (0..n)
            .map(|i| 0.5 * self.rho[i] * (self.ux[i] * self.ux[i] + self.uy[i] * self.uy[i]))
            .sum()
    }

    /// u-velocity along the vertical centreline (x = nx/2) for all y.
    ///
    /// Returns a `Vec` of length `ny`; index 0 corresponds to y = 0 (bottom).
    pub fn centerline_u(&self) -> Vec<f64> {
        let x = self.nx / 2;
        (0..self.ny).map(|y| self.ux[self.idx(x, y)]).collect()
    }

    /// v-velocity along the horizontal centreline (y = ny/2) for all x.
    ///
    /// Returns a `Vec` of length `nx`; index 0 corresponds to x = 0 (left).
    pub fn centerline_v(&self) -> Vec<f64> {
        let y = self.ny / 2;
        (0..self.nx).map(|x| self.uy[self.idx(x, y)]).collect()
    }

    // ── Output helpers ───────────────────────────────────────────────────────

    /// Writes the full velocity field to a CSV file at the given path.
    ///
    /// Columns: `x,y,rho,ux,uy`
    ///
    /// # Errors
    /// Returns an `io::Error` if the file cannot be created or written.
    pub fn write_velocity_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "x,y,rho,ux,uy")?;
        for y in 0..self.ny {
            for x in 0..self.nx {
                let idx = self.idx(x, y);
                writeln!(
                    file,
                    "{x},{y},{:.8e},{:.8e},{:.8e}",
                    self.rho[idx], self.ux[idx], self.uy[idx]
                )?;
            }
        }
        Ok(())
    }

    /// Writes the centreline profiles (u at x=0.5, v at y=0.5) to a CSV file.
    ///
    /// Columns: `index,u_vert_centerline,v_horiz_centerline`
    ///
    /// # Errors
    /// Returns an `io::Error` if the file cannot be created or written.
    pub fn write_centerline_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let u_prof = self.centerline_u();
        let v_prof = self.centerline_v();
        let n = self.nx.max(self.ny);
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "index,u_vert_centerline,v_horiz_centerline")?;
        for i in 0..n {
            let u = if i < u_prof.len() { u_prof[i] } else { 0.0 };
            let v = if i < v_prof.len() { v_prof[i] } else { 0.0 };
            writeln!(file, "{i},{u:.8e},{v:.8e}")?;
        }
        Ok(())
    }
}
