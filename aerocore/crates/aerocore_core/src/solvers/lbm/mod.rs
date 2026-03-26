pub mod boundary;
pub mod collision;
pub mod d2q9;
pub mod d3q19;
pub mod lbm2d;
pub mod streaming;

use crate::math_core::precision::FloatPrecision;
use crate::memory::arena::SimArena;
use crate::solvers::traits::*;

pub struct LbmSolver<'a, T: FloatPrecision> {
    nx: usize,
    ny: usize,
    nz: usize,
    _tau: T, // Adicionado underline para evitar warning
    omega: T,
    f_in: Vec<&'a mut [T]>,
    f_out: Vec<&'a mut [T]>,
    rho: &'a mut [T],
    u_x: &'a mut [T],
    u_y: &'a mut [T],
    u_z: &'a mut [T],
    /// Per-cell solid mask.  `true` = solid (bounce-back); `false` = fluid.
    /// Pre-allocated in `init()`; never grows inside `step*` functions.
    is_boundary: Vec<bool>,
    timestep: u64,
    initialized: bool,
}

impl<'a, T: FloatPrecision> LbmSolver<'a, T> {
    pub fn new(nx: usize, ny: usize, nz: usize, viscosity: T) -> Self {
        let three = T::from_f64(3.0).unwrap();
        let half = T::from_f64(0.5).unwrap();
        let tau = three * viscosity + half;
        Self {
            nx,
            ny,
            nz,
            _tau: tau,
            omega: T::ONE / tau,
            f_in: Vec::with_capacity(d3q19::Q),
            f_out: Vec::with_capacity(d3q19::Q),
            rho: &mut [],
            u_x: &mut [],
            u_y: &mut [],
            u_z: &mut [],
            is_boundary: Vec::new(),
            timestep: 0,
            initialized: false,
        }
    }

    /// Overrides the solid-cell mask used for per-cell bounce-back.
    ///
    /// `mask[idx]` must be `true` for every solid cell, where
    /// `idx = (z * ny + y) * nx + x`.  The mask is cloned from the caller's
    /// `Vec<bool>` so that it can be built from STL geometry
    /// (see [`crate::io_mesh::voxelize::stl_to_boundary_mask`]).
    ///
    /// Call this **after** [`Solver::init`] and **before** the first `step*`.
    /// The mask length must equal `nx * ny * nz`; a length mismatch is
    /// silently ignored to keep the hot path allocation-free.
    pub fn set_boundary_mask(&mut self, mask: Vec<bool>) {
        let expected = self.nx * self.ny * self.nz;
        if mask.len() == expected {
            self.is_boundary = mask;
        }
    }

    #[inline(always)]
    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.ny + y) * self.nx + x
    }

    // ── analysis helpers ──────────────────────────────────────────────────────

    /// Returns the x-velocity profile along the y-axis at the given (x, z) column.
    ///
    /// The returned `Vec` has length `ny`; index 0 corresponds to y = 0.
    pub fn ux_profile_y(&self, x: usize, z: usize) -> Vec<f64> {
        (0..self.ny)
            .map(|y| {
                let idx = self.get_index(x, y, z);
                self.u_x[idx].to_f64().unwrap_or(0.0)
            })
            .collect()
    }

    /// Returns the sum of all cell densities (mass proxy in LBM).
    pub fn total_density(&self) -> f64 {
        self.rho.iter().map(|r| r.to_f64().unwrap_or(0.0)).sum()
    }

    /// Returns the total kinetic energy: Σ ½ρ(u² + v² + w²).
    pub fn total_kinetic_energy(&self) -> f64 {
        let n = self.nx * self.ny * self.nz;
        (0..n)
            .map(|i| {
                let rho = self.rho[i].to_f64().unwrap_or(0.0);
                let ux = self.u_x[i].to_f64().unwrap_or(0.0);
                let uy = self.u_y[i].to_f64().unwrap_or(0.0);
                let uz = self.u_z[i].to_f64().unwrap_or(0.0);
                0.5 * rho * (ux * ux + uy * uy + uz * uz)
            })
            .sum()
    }

    // ── physics step variants ─────────────────────────────────────────────────

    /// BGK step with a uniform body force (Guo forcing scheme) and half-way
    /// bounce-back no-slip walls at y = 0 and y = ny − 1.
    ///
    /// Periodic boundaries are applied in x and z.
    ///
    /// # Guo Forcing Scheme
    /// Adds a forcing distribution to the post-collision populations:
    ///
    ///   F_i = wᵢ (1 − ω/2) [ (eᵢ·F − u·F)/cs² + (eᵢ·u)(eᵢ·F)/cs⁴ ]
    ///
    /// Reference: Krüger et al. (2017), Eq. 5.14
    pub fn step_with_body_force(&mut self, fx: T, fy: T, fz: T) -> Result<StepResult, SolverError> {
        if !self.initialized {
            return Err(SolverError::InitializationFailed("not initialized".into()));
        }

        let q = d3q19::Q;
        let cs2 = T::from_f64(d3q19::CS2).unwrap(); // 1/3
        let inv_cs2 = T::ONE / cs2; // 3
        let inv_cs4 = inv_cs2 * inv_cs2; // 9
        let inv_2cs4 = inv_cs4 / T::TWO; // 9/2
        let inv_2cs2 = inv_cs2 / T::TWO; // 3/2
        let half_omega = T::from_f64(0.5).unwrap() * self.omega;

        // Zero output buffer
        for i in 0..q {
            for v in self.f_out[i].iter_mut() {
                *v = T::ZERO;
            }
        }

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.get_index(x, y, z);

                    // 1. Macroscopic density and velocity
                    let mut r = T::ZERO;
                    let mut ux = T::ZERO;
                    let mut uy = T::ZERO;
                    let mut uz = T::ZERO;
                    for i in 0..q {
                        let fi = self.f_in[i][idx];
                        r += fi;
                        ux += T::from_i32(d3q19::E[i][0]).unwrap() * fi;
                        uy += T::from_i32(d3q19::E[i][1]).unwrap() * fi;
                        uz += T::from_i32(d3q19::E[i][2]).unwrap() * fi;
                    }
                    let inv_r = if r > T::ZERO { T::ONE / r } else { T::ZERO };
                    ux *= inv_r;
                    uy *= inv_r;
                    uz *= inv_r;

                    // Guo: effective velocity = u + F/(2ρ)
                    let inv_2r = if r > T::ZERO {
                        T::ONE / (T::TWO * r)
                    } else {
                        T::ZERO
                    };
                    let ux_eff = ux + fx * inv_2r;
                    let uy_eff = uy + fy * inv_2r;
                    let uz_eff = uz + fz * inv_2r;

                    self.rho[idx] = r;
                    self.u_x[idx] = ux_eff;
                    self.u_y[idx] = uy_eff;
                    self.u_z[idx] = uz_eff;

                    let u_sq = ux_eff * ux_eff + uy_eff * uy_eff + uz_eff * uz_eff;
                    let u_dot_f = ux_eff * fx + uy_eff * fy + uz_eff * fz;

                    for i in 0..q {
                        let ei = d3q19::E[i];
                        let wi = T::from_f64(d3q19::W[i]).unwrap();
                        let eix = T::from_i32(ei[0]).unwrap();
                        let eiy = T::from_i32(ei[1]).unwrap();
                        let eiz = T::from_i32(ei[2]).unwrap();

                        let ei_dot_u = eix * ux_eff + eiy * uy_eff + eiz * uz_eff;
                        let ei_dot_f = eix * fx + eiy * fy + eiz * fz;

                        // BGK equilibrium with effective velocity
                        let feq = wi
                            * r
                            * (T::ONE + ei_dot_u * inv_cs2 + ei_dot_u * ei_dot_u * inv_2cs4
                                - u_sq * inv_2cs2);

                        // Guo forcing distribution (Krüger Eq. 5.14)
                        let fi_force = wi
                            * (T::ONE - half_omega)
                            * ((ei_dot_f - u_dot_f) * inv_cs2 + ei_dot_u * ei_dot_f * inv_cs4);

                        let f_post =
                            self.f_in[i][idx] - self.omega * (self.f_in[i][idx] - feq) + fi_force;

                        // Streaming: periodic in x/z, bounce-back in y
                        let nx_new = (x as i32 + ei[0]).rem_euclid(self.nx as i32) as usize;
                        let ny_new = y as i32 + ei[1];
                        let nz_new = (z as i32 + ei[2]).rem_euclid(self.nz as i32) as usize;

                        if ny_new < 0 || ny_new >= self.ny as i32 {
                            // Half-way bounce-back: reverse direction, accumulate at source
                            let opp = d3q19::OPPOSITE[i];
                            self.f_out[opp][idx] += f_post;
                        } else {
                            let target = self.get_index(nx_new, ny_new as usize, nz_new);
                            if self.is_boundary[target] {
                                // Solid interior cell: per-cell bounce-back
                                let opp = d3q19::OPPOSITE[i];
                                self.f_out[opp][idx] += f_post;
                            } else {
                                self.f_out[i][target] += f_post;
                            }
                        }
                    }
                }
            }
        }

        std::mem::swap(&mut self.f_in, &mut self.f_out);
        self.timestep += 1;
        Ok(StepResult {
            timestep: self.timestep,
            time: self.timestep as f64,
            dt: 1.0,
            residual_l2: 0.0,
            converged: false,
        })
    }

    /// BGK step with a moving lid at y = ny − 1 (velocity u_lid in the
    /// x-direction) and a no-slip bounce-back wall at y = 0.
    ///
    /// Periodic boundaries are applied in x and z.
    ///
    /// The moving wall uses the standard momentum-exchange correction:
    ///
    ///   f_opp[source] += 2 wᵢ ρ_wall (eᵢ · u_wall) / cs²
    ///
    /// Reference: Ladd (1994), J. Fluid Mech. 271
    pub fn step_with_lid(&mut self, u_lid: T) -> Result<StepResult, SolverError> {
        if !self.initialized {
            return Err(SolverError::InitializationFailed("not initialized".into()));
        }

        let q = d3q19::Q;
        let cs2 = T::from_f64(d3q19::CS2).unwrap();
        let inv_cs2 = T::ONE / cs2;
        let inv_2cs2 = T::ONE / (T::TWO * cs2);
        let inv_2cs4 = T::ONE / (T::TWO * cs2 * cs2);
        let two = T::TWO;

        // Zero output buffer
        for i in 0..q {
            for v in self.f_out[i].iter_mut() {
                *v = T::ZERO;
            }
        }

        let ny_max = self.ny as i32 - 1;

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.get_index(x, y, z);

                    // 1. Macroscopic density and velocity
                    let mut r = T::ZERO;
                    let mut ux = T::ZERO;
                    let mut uy = T::ZERO;
                    let mut uz = T::ZERO;
                    for i in 0..q {
                        let fi = self.f_in[i][idx];
                        r += fi;
                        ux += T::from_i32(d3q19::E[i][0]).unwrap() * fi;
                        uy += T::from_i32(d3q19::E[i][1]).unwrap() * fi;
                        uz += T::from_i32(d3q19::E[i][2]).unwrap() * fi;
                    }
                    let inv_r = if r > T::ZERO { T::ONE / r } else { T::ZERO };
                    ux *= inv_r;
                    uy *= inv_r;
                    uz *= inv_r;

                    self.rho[idx] = r;
                    self.u_x[idx] = ux;
                    self.u_y[idx] = uy;
                    self.u_z[idx] = uz;

                    let u_sq = ux * ux + uy * uy + uz * uz;

                    for i in 0..q {
                        let ei = d3q19::E[i];
                        let wi = T::from_f64(d3q19::W[i]).unwrap();
                        let eix = T::from_i32(ei[0]).unwrap();
                        let eiy = T::from_i32(ei[1]).unwrap();
                        let eiz = T::from_i32(ei[2]).unwrap();

                        let ei_dot_u = eix * ux + eiy * uy + eiz * uz;
                        let feq = wi
                            * r
                            * (T::ONE + ei_dot_u * inv_cs2 + ei_dot_u * ei_dot_u * inv_2cs4
                                - u_sq * inv_2cs2);

                        let f_post = self.f_in[i][idx] - self.omega * (self.f_in[i][idx] - feq);

                        // Streaming
                        let nx_new = (x as i32 + ei[0]).rem_euclid(self.nx as i32) as usize;
                        let ny_new = y as i32 + ei[1];
                        let nz_new = (z as i32 + ei[2]).rem_euclid(self.nz as i32) as usize;

                        if ny_new < 0 {
                            // No-slip bottom wall (y = 0): standard bounce-back
                            let opp = d3q19::OPPOSITE[i];
                            self.f_out[opp][idx] += f_post;
                        } else if ny_new > ny_max {
                            // Moving lid (y = ny-1): moving wall bounce-back
                            let opp = d3q19::OPPOSITE[i];
                            // Momentum exchange correction: 2 w_i ρ (e_i · u_wall) / cs²
                            let ei_dot_uw = eix * u_lid; // u_wall = (u_lid, 0, 0)
                            let correction = two * wi * r * ei_dot_uw * inv_cs2;
                            self.f_out[opp][idx] += f_post - correction;
                        } else {
                            let target = self.get_index(nx_new, ny_new as usize, nz_new);
                            if self.is_boundary[target] {
                                // Solid interior cell: per-cell bounce-back
                                let opp = d3q19::OPPOSITE[i];
                                self.f_out[opp][idx] += f_post;
                            } else {
                                self.f_out[i][target] += f_post;
                            }
                        }
                    }
                }
            }
        }

        std::mem::swap(&mut self.f_in, &mut self.f_out);
        self.timestep += 1;
        Ok(StepResult {
            timestep: self.timestep,
            time: self.timestep as f64,
            dt: 1.0,
            residual_l2: 0.0,
            converged: false,
        })
    }
}

impl<'a, T: FloatPrecision> Solver<'a> for LbmSolver<'a, T> {
    fn init(&mut self, _config: &SolverConfig, arena: &'a SimArena) -> Result<(), SolverError> {
        let num_cells = self.nx * self.ny * self.nz;
        self.f_in.clear();
        self.f_out.clear();

        // Initialise each distribution to the equilibrium at rest (ρ=1, u=0):
        //   f_i^eq = w_i  (since ρ=1 and u=0)
        for i in 0..d3q19::Q {
            let w = T::from_f64(d3q19::W[i]).unwrap();
            self.f_in.push(arena.alloc_aligned_slice(num_cells, w));
            self.f_out
                .push(arena.alloc_aligned_slice(num_cells, T::ZERO));
        }
        self.rho = arena.alloc_aligned_slice(num_cells, T::ONE);
        self.u_x = arena.alloc_aligned_slice(num_cells, T::ZERO);
        self.u_y = arena.alloc_aligned_slice(num_cells, T::ZERO);
        self.u_z = arena.alloc_aligned_slice(num_cells, T::ZERO);
        // Pre-allocate boundary mask: all fluid by default.
        // set_boundary_mask() can override this before the first step.
        self.is_boundary.clear();
        self.is_boundary.resize(num_cells, false);
        self.initialized = true;
        Ok(())
    }

    fn step(&mut self) -> Result<StepResult, SolverError> {
        if !self.initialized {
            return Err(SolverError::InitializationFailed("Não init".into()));
        }
        let q = d3q19::Q;
        let cs2 = T::from_f64(d3q19::CS2).unwrap();
        let inv_2cs4 = T::ONE / (T::TWO * cs2 * cs2);
        let inv_2cs2 = T::ONE / (T::TWO * cs2);
        let inv_cs2 = T::ONE / cs2;

        // Zero output buffer (required for += accumulation and bounce-back)
        for i in 0..q {
            for v in self.f_out[i].iter_mut() {
                *v = T::ZERO;
            }
        }

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.get_index(x, y, z);
                    let mut r = T::ZERO;
                    let mut ux = T::ZERO;
                    let mut uy = T::ZERO;
                    let mut uz = T::ZERO;
                    for i in 0..q {
                        let fi = self.f_in[i][idx];
                        r += fi;
                        ux += T::from_i32(d3q19::E[i][0]).unwrap() * fi;
                        uy += T::from_i32(d3q19::E[i][1]).unwrap() * fi;
                        uz += T::from_i32(d3q19::E[i][2]).unwrap() * fi;
                    }
                    let inv_r = if r > T::ZERO { T::ONE / r } else { T::ZERO };
                    ux *= inv_r;
                    uy *= inv_r;
                    uz *= inv_r;

                    self.rho[idx] = r;
                    self.u_x[idx] = ux;
                    self.u_y[idx] = uy;
                    self.u_z[idx] = uz;

                    let u_sq = ux * ux + uy * uy + uz * uz;
                    for i in 0..q {
                        let ei = d3q19::E[i];
                        let wi = T::from_f64(d3q19::W[i]).unwrap();
                        let dot = T::from_i32(ei[0]).unwrap() * ux
                            + T::from_i32(ei[1]).unwrap() * uy
                            + T::from_i32(ei[2]).unwrap() * uz;
                        let feq = wi
                            * r
                            * (T::ONE + dot * inv_cs2 + dot * dot * inv_2cs4 - u_sq * inv_2cs2);
                        let f_post = self.f_in[i][idx] - self.omega * (self.f_in[i][idx] - feq);
                        // Streaming: periodic in x/y/z with per-cell bounce-back
                        let nx_new = (x as i32 + ei[0]).rem_euclid(self.nx as i32) as usize;
                        let ny_new = (y as i32 + ei[1]).rem_euclid(self.ny as i32) as usize;
                        let nz_new = (z as i32 + ei[2]).rem_euclid(self.nz as i32) as usize;
                        let target_idx = self.get_index(nx_new, ny_new, nz_new);
                        if self.is_boundary[target_idx] {
                            // Solid neighbor: half-way bounce-back to opposite direction
                            let opp = d3q19::OPPOSITE[i];
                            self.f_out[opp][idx] += f_post;
                        } else {
                            self.f_out[i][target_idx] += f_post;
                        }
                    }
                }
            }
        }
        std::mem::swap(&mut self.f_in, &mut self.f_out);
        self.timestep += 1;
        Ok(StepResult {
            timestep: self.timestep,
            time: self.timestep as f64,
            dt: 1.0,
            residual_l2: 0.0,
            converged: false,
        })
    }

    fn snapshot_field_data(&self, output: &mut FieldDataBuffer) {
        output.num_points = self.nx * self.ny * self.nz;
        output.pressure = self.rho.as_ptr() as *const f64;
        output.velocity_x = self.u_x.as_ptr() as *const f64;
        output.velocity_y = self.u_y.as_ptr() as *const f64;
        output.velocity_z = self.u_z.as_ptr() as *const f64;
        output.generation += 1;
    }

    fn finalize(&mut self) {
        self.initialized = false;
    }

    fn name(&self) -> &'static str {
        "LBM D3Q19"
    }
}
