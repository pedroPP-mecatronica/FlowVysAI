//! Compressible Navier-Stokes solver — STUB.
//!
//! Full implementation in Sprint S9.

use crate::memory::arena::SimArena;
use crate::solvers::traits::*;

/// Compressible Navier-Stokes solver for high-speed aeronautical flows.
///
/// Supports RANS (k-ω SST) and future LES modes.
/// Targets Mach 0.3–5.0 (subsonic through hypersonic).
pub struct CompressibleNsSolver {
    initialized: bool,
    timestep: u64,
}

impl CompressibleNsSolver {
    pub fn new() -> Self {
        Self {
            initialized: false,
            timestep: 0,
        }
    }
}

impl Default for CompressibleNsSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for CompressibleNsSolver {
    fn init(&mut self, _config: &SolverConfig, _arena: &SimArena) -> Result<(), SolverError> {
        // TODO (S9): Allocate conservative variable arrays (ρ, ρu, ρv, ρw, ρE)
        // TODO (S9): Initialize flux arrays, Jacobian scratch space
        self.initialized = true;
        self.timestep = 0;
        Ok(())
    }

    fn step(&mut self) -> Result<StepResult, SolverError> {
        if !self.initialized {
            return Err(SolverError::InitializationFailed(
                "Solver not initialized. Call init() first.".into(),
            ));
        }
        self.timestep += 1;
        Ok(StepResult {
            timestep: self.timestep,
            time: self.timestep as f64 * 1e-4,
            dt: 1e-4,
            residual_l2: 1.0, // Stub: no actual computation
            converged: false,
        })
    }

    fn snapshot_field_data(&self, _output: &mut FieldDataBuffer) {
        // TODO (S9): Copy velocity, pressure, Mach fields to output buffer
    }

    fn finalize(&mut self) {
        self.initialized = false;
        self.timestep = 0;
    }

    fn name(&self) -> &'static str {
        "Compressible Navier-Stokes (Roe/HLLC)"
    }
}
