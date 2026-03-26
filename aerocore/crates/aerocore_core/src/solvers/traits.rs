use crate::memory::arena::SimArena;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StepResult {
    pub timestep: u64,
    pub time: f64,
    pub dt: f64,
    pub residual_l2: f64,
    pub converged: bool,
}

#[derive(Debug, Clone, Default)] // Adicionado Default
pub struct SolverConfig {
    pub max_iterations: u64,
    pub convergence_threshold: f64,
    pub dt: f64,
    pub use_gpu: bool,
    pub precision: PrecisionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] // Adicionado Default
#[repr(C)]
pub enum PrecisionMode {
    #[default]
    FP64,
    FP32,
}

#[derive(Debug)]
pub enum SolverError {
    InitializationFailed(String),
    NumericalDivergence { timestep: u64, residual: f64 },
    GpuError(String),
    InvalidConfig(String),
}

/// Contrato principal que todo solver deve implementar.
pub trait Solver<'a>: Send + Sync {
    fn init(&mut self, config: &SolverConfig, arena: &'a SimArena) -> Result<(), SolverError>;
    fn step(&mut self) -> Result<StepResult, SolverError>;
    fn snapshot_field_data(&self, output: &mut FieldDataBuffer);
    fn finalize(&mut self);
    fn name(&self) -> &'static str;
}

#[repr(C)]
pub struct FieldDataBuffer {
    pub num_points: usize,
    pub positions_x: *const f64,
    pub positions_y: *const f64,
    pub positions_z: *const f64,
    pub velocity_x: *const f64,
    pub velocity_y: *const f64,
    pub velocity_z: *const f64,
    pub pressure: *const f64,
    pub temperature: *const f64,
    pub mach: *const f64,
    pub generation: u64,
}

#[derive(Debug, Clone)]
pub enum SimulationCommand {
    Start(SolverConfig),
    Pause,
    Resume,
    Stop,
    RequestSnapshot,
}
