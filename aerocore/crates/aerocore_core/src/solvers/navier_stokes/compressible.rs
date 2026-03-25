use crate::memory::arena::SimArena;
use crate::solvers::traits::*;

pub struct CompressibleNsSolver {
    initialized: bool,
    _timestep: u64, // Adicionado underline para evitar warning
}

impl Default for CompressibleNsSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressibleNsSolver {
    pub fn new() -> Self { 
        Self { initialized: false, _timestep: 0 } 
    }
}

impl<'a> Solver<'a> for CompressibleNsSolver {
    fn init(&mut self, _config: &SolverConfig, _arena: &'a SimArena) -> Result<(), SolverError> {
        self.initialized = true;
        Ok(())
    }

    fn step(&mut self) -> Result<StepResult, SolverError> {
        Ok(StepResult { 
            timestep: 0, 
            time: 0.0, 
            dt: 0.0, 
            residual_l2: 0.0, 
            converged: false 
        })
    }

    fn snapshot_field_data(&self, _output: &mut FieldDataBuffer) {}
    fn finalize(&mut self) { self.initialized = false; }
    fn name(&self) -> &'static str { "Compressible Navier-Stokes (Stub)" }
}