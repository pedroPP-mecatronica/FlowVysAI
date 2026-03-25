//! C-compatible types for the FFI boundary.
//!
//! All types use `#[repr(C)]` for stable ABI layout.
//! These mirror the Rust-side types in `aerocore_core::solvers::traits`.

use aerocore_core::solvers::traits::{PrecisionMode, SolverConfig};

/// C-compatible solver configuration.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FfiSolverConfig {
    pub max_iterations: u64,
    pub convergence_threshold: f64,
    pub dt: f64,
    pub use_gpu: bool,
    pub precision: FfiPrecisionMode,
}

impl FfiSolverConfig {
    pub fn from_rust(config: &SolverConfig) -> Self {
        Self {
            max_iterations: config.max_iterations,
            convergence_threshold: config.convergence_threshold,
            dt: config.dt,
            use_gpu: config.use_gpu,
            precision: match config.precision {
                PrecisionMode::FP64 => FfiPrecisionMode::FP64,
                PrecisionMode::FP32 => FfiPrecisionMode::FP32,
            },
        }
    }
}

/// C-compatible precision mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum FfiPrecisionMode {
    FP64 = 0,
    FP32 = 1,
}

/// C-compatible simulation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum FfiSimulationState {
    Idle = 0,
    Initializing = 1,
    Running = 2,
    Paused = 3,
    Converged = 4,
    Diverged = 5,
    Error = 6,
}

/// C-compatible step result.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FfiStepResult {
    pub timestep: u64,
    pub time: f64,
    pub dt: f64,
    pub residual_l2: f64,
    pub converged: bool,
}
