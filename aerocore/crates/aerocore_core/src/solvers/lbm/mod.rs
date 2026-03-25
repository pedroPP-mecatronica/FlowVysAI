pub mod d3q19;
pub mod collision;
pub mod streaming;
pub mod boundary;

use crate::memory::arena::SimArena;
use crate::math_core::precision::FloatPrecision;
use crate::solvers::traits::*;

pub struct LbmSolver<'a, T: FloatPrecision> {
    nx: usize,
    ny: usize,
    nz: usize,
    _tau: T,        // Adicionado underline para evitar warning
    omega: T,
    f_in: Vec<&'a mut [T]>,
    f_out: Vec<&'a mut [T]>,
    rho: &'a mut [T],
    u_x: &'a mut [T],
    u_y: &'a mut [T],
    u_z: &'a mut [T],
    timestep: u64,
    initialized: bool,
}

impl<'a, T: FloatPrecision> LbmSolver<'a, T> {
    pub fn new(nx: usize, ny: usize, nz: usize, viscosity: T) -> Self {
        let three = T::from_f64(3.0).unwrap();
        let half = T::from_f64(0.5).unwrap();
        let tau = three * viscosity + half;
        Self {
            nx, ny, nz, 
            _tau: tau, 
            omega: T::ONE / tau,
            f_in: Vec::with_capacity(d3q19::Q),
            f_out: Vec::with_capacity(d3q19::Q),
            rho: &mut [],
            u_x: &mut [], u_y: &mut [], u_z: &mut [],
            timestep: 0,
            initialized: false,
        }
    }
    #[inline(always)]
    fn get_index(&self, x: usize, y: usize, z: usize) -> usize { (z * self.ny + y) * self.nx + x }
}

impl<'a, T: FloatPrecision> Solver<'a> for LbmSolver<'a, T> {
    fn init(&mut self, _config: &SolverConfig, arena: &'a SimArena) -> Result<(), SolverError> {
        let num_cells = self.nx * self.ny * self.nz;
        self.f_in.clear();
        self.f_out.clear();
        for _ in 0..d3q19::Q {
            self.f_in.push(arena.alloc_aligned_slice(num_cells, T::ZERO));
            self.f_out.push(arena.alloc_aligned_slice(num_cells, T::ZERO));
        }
        self.rho = arena.alloc_aligned_slice(num_cells, T::ONE);
        self.u_x = arena.alloc_aligned_slice(num_cells, T::ZERO);
        self.u_y = arena.alloc_aligned_slice(num_cells, T::ZERO);
        self.u_z = arena.alloc_aligned_slice(num_cells, T::ZERO);
        self.initialized = true;
        Ok(())
    }

    fn step(&mut self) -> Result<StepResult, SolverError> {
        if !self.initialized { return Err(SolverError::InitializationFailed("Não init".into())); }
        let q = d3q19::Q;
        let cs2 = T::from_f64(d3q19::CS2).unwrap();
        let inv_2cs4 = T::ONE / (T::TWO * cs2 * cs2);
        let inv_2cs2 = T::ONE / (T::TWO * cs2);

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.get_index(x, y, z);
                    let mut r = T::ZERO;
                    let mut ux = T::ZERO; let mut uy = T::ZERO; let mut uz = T::ZERO;
                    for i in 0..q {
                        let fi = self.f_in[i][idx];
                        r += fi;
                        ux += T::from_i32(d3q19::E[i][0]).unwrap() * fi;
                        uy += T::from_i32(d3q19::E[i][1]).unwrap() * fi;
                        uz += T::from_i32(d3q19::E[i][2]).unwrap() * fi;
                    }
                    if r > T::ZERO { ux /= r; uy /= r; uz /= r; }
                    self.rho[idx] = r; self.u_x[idx] = ux; self.u_y[idx] = uy; self.u_z[idx] = uz;
                    let u_sq = ux*ux + uy*uy + uz*uz;
                    for i in 0..q {
                        let ei = d3q19::E[i];
                        let wi = T::from_f64(d3q19::W[i]).unwrap();
                        let dot = T::from_i32(ei[0]).unwrap() * ux + T::from_i32(ei[1]).unwrap() * uy + T::from_i32(ei[2]).unwrap() * uz;
                        let feq = wi * r * (T::ONE + dot / cs2 + (dot * dot) * inv_2cs4 - u_sq * inv_2cs2);
                        let f_post = self.f_in[i][idx] - self.omega * (self.f_in[i][idx] - feq);
                        let nx_new = (x as i32 + ei[0]).rem_euclid(self.nx as i32) as usize;
                        let ny_new = (y as i32 + ei[1]).rem_euclid(self.ny as i32) as usize;
                        let nz_new = (z as i32 + ei[2]).rem_euclid(self.nz as i32) as usize;
                        let target_idx = self.get_index(nx_new, ny_new, nz_new);
                        self.f_out[i][target_idx] = f_post;
                    }
                }
            }
        }
        std::mem::swap(&mut self.f_in, &mut self.f_out);
        self.timestep += 1;
        Ok(StepResult { timestep: self.timestep, time: 0.0, dt: 0.001, residual_l2: 0.0, converged: false })
    }

    fn snapshot_field_data(&self, output: &mut FieldDataBuffer) {
        output.num_points = self.nx * self.ny * self.nz;
        output.pressure = self.rho.as_ptr() as *const f64;
        output.velocity_x = self.u_x.as_ptr() as *const f64;
        output.velocity_y = self.u_y.as_ptr() as *const f64;
        output.velocity_z = self.u_z.as_ptr() as *const f64;
        output.generation += 1;
    }
    fn finalize(&mut self) { self.initialized = false; }
    fn name(&self) -> &'static str { "LBM D3Q19" }
}