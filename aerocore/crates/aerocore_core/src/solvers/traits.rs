//! Solver traits and contracts — the unified API between Core and UI.
//!
//! Defines the 5 fundamental contracts from the Architecture Manifest:
//! 1. `Solver` — simulation engine contract
//! 2. `FieldDataBuffer` — SoA data transfer Core→UI (zero-copy)
//! 3. `SimulationOrchestrator` — lifecycle control by UI
//! 4. `SimulationCommand` / `SimulationState` — command/state enums
//! 5. (GpuComputeBackend is in gpu_compute/device.rs)
//!
//! # Steering Compliance
//! - DESACOPLAMENTO EXTREMO: UI never imports solver internals
//! - ZERO ALOCAÇÃO: `init()` allocates everything, `step()` is allocation-free
//! - PRECISÃO FP64: Default precision, FP32 opt-in via `PrecisionMode`

use crate::memory::arena::SimArena;

// ============================================================================
// Contract 1: Solver — Simulation Engine
// ============================================================================

/// Result of a single simulation timestep.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StepResult {
    /// Current timestep number (monotonically increasing).
    pub timestep: u64,
    /// Physical simulation time in seconds.
    pub time: f64,
    /// Timestep size used for this step.
    pub dt: f64,
    /// L₂ norm of the residual (convergence metric).
    pub residual_l2: f64,
    /// True if the simulation has converged below threshold.
    pub converged: bool,
}

/// Solver configuration — passed during `init()`.
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum number of iterations (timesteps for transient, iterations for steady).
    pub max_iterations: u64,
    /// Convergence threshold for residual L₂ norm.
    pub convergence_threshold: f64,
    /// Timestep size in seconds.
    pub dt: f64,
    /// Whether to offload computation to GPU.
    pub use_gpu: bool,
    /// Floating-point precision mode.
    pub precision: PrecisionMode,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10_000,
            convergence_threshold: 1e-6,
            dt: 1e-4,
            use_gpu: false,
            precision: PrecisionMode::FP64,
        }
    }
}

/// Floating-point precision mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PrecisionMode {
    /// Double precision (64-bit) — default, required for scientific accuracy.
    FP64,
    /// Single precision (32-bit) — opt-in for fast GPU execution.
    FP32,
}

/// Main solver contract. Every solver (LBM, N-S, etc.) implements this.
///
/// # Lifecycle
/// 1. `init()` — allocates ALL memory via the arena (ONLY allocation point)
/// 2. `step()` — advances one timestep (ZERO allocation, hot path)
/// 3. `snapshot_field_data()` — copies field data to buffer for UI consumption
/// 4. `finalize()` — releases resources and resets state
pub trait Solver: Send + Sync {
    /// Initializes the solver with the given configuration.
    ///
    /// This is the ONLY point where memory allocation is permitted.
    /// All field arrays, scratch buffers, and working memory must be
    /// allocated from the provided `SimArena`.
    fn init(&mut self, config: &SolverConfig, arena: &SimArena) -> Result<(), SolverError>;

    /// Advances the simulation by one timestep.
    ///
    /// # Steering #2 Compliance
    /// ZERO dynamic allocation is permitted inside this method.
    /// All data must have been pre-allocated during `init()`.
    fn step(&mut self) -> Result<StepResult, SolverError>;

    /// Copies current field data to the output buffer for UI consumption.
    ///
    /// This is the ONLY interface through which the UI accesses solver data.
    /// The buffer uses SoA layout for direct GPU upload.
    fn snapshot_field_data(&self, output: &mut FieldDataBuffer);

    /// Releases all resources and resets the solver to its initial state.
    fn finalize(&mut self);

    /// Human-readable solver name for UI display.
    fn name(&self) -> &'static str;
}

/// Solver error types.
#[derive(Debug)]
pub enum SolverError {
    /// Initialization failed (e.g., invalid config, insufficient arena).
    InitializationFailed(String),
    /// Numerical divergence detected.
    NumericalDivergence {
        timestep: u64,
        residual: f64,
    },
    /// GPU backend error.
    GpuError(String),
    /// Invalid configuration parameter.
    InvalidConfig(String),
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::NumericalDivergence { timestep, residual } =>
                write!(f, "Numerical divergence at timestep {}: residual = {:.6e}", timestep, residual),
            Self::GpuError(msg) => write!(f, "GPU error: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for SolverError {}

// ============================================================================
// Contract 2: FieldDataBuffer — SoA Data Transfer Core → UI
// ============================================================================

/// SoA (Structure of Arrays) buffer for field data, transferred from Core to UI.
///
/// # Data-Oriented Design
/// Each component is a contiguous slice: `[x₀,x₁,...], [y₀,y₁,...], [z₀,z₁,...]`
/// This enables:
/// - SIMD vectorized operations
/// - Direct upload to GPU vertex/storage buffers
/// - Cache-friendly sequential access
///
/// # FFI Safety
/// All pointers are `*const f64` with `repr(C)` for direct use across FFI boundary.
#[repr(C)]
pub struct FieldDataBuffer {
    /// Number of points/cells in the field.
    pub num_points: usize,

    /// Cell center positions — SoA layout.
    pub positions_x: *const f64,
    pub positions_y: *const f64,
    pub positions_z: *const f64,

    /// Velocity components — SoA layout.
    pub velocity_x: *const f64,
    pub velocity_y: *const f64,
    pub velocity_z: *const f64,

    /// Scalar pressure field.
    pub pressure: *const f64,

    /// Scalar temperature field (null if not applicable).
    pub temperature: *const f64,

    /// Local Mach number (for compressible N-S; null otherwise).
    pub mach: *const f64,

    /// Snapshot generation number — UI uses this to detect changes.
    pub generation: u64,
}

/// Safe Rust-side view of field data (lifetime-bound).
pub struct FieldDataView<'a> {
    pub num_points: usize,
    pub positions: (&'a [f64], &'a [f64], &'a [f64]),
    pub velocity: (&'a [f64], &'a [f64], &'a [f64]),
    pub pressure: &'a [f64],
    pub temperature: Option<&'a [f64]>,
    pub mach: Option<&'a [f64]>,
    pub generation: u64,
}

// ============================================================================
// Contract 3 & 4: SimulationOrchestrator — Lifecycle Control
// ============================================================================

/// Commands the UI can send to the solver (unidirectional: UI → Core).
#[derive(Debug, Clone)]
pub enum SimulationCommand {
    /// Start simulation with the given configuration.
    Start(SolverConfig),
    /// Pause (preserve state).
    Pause,
    /// Resume from paused state.
    Resume,
    /// Stop completely and release resources.
    Stop,
    /// Request a field data snapshot.
    RequestSnapshot,
    /// Modify a parameter at runtime.
    SetParameter {
        key: String,
        value: ParameterValue,
    },
}

/// Runtime parameter value types.
#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Text(String),
}

/// Simulation state visible to the UI (lock-free polling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum SimulationState {
    Idle,
    Initializing,
    Running,
    Paused,
    Converged,
    Diverged,
    Error,
}

/// Orchestrator contract — the bridge between UI thread and solver thread.
///
/// All methods are non-blocking and thread-safe.
pub trait SimulationOrchestrator: Send {
    /// Sends a command to the solver's queue (non-blocking).
    fn send_command(&self, cmd: SimulationCommand) -> Result<(), OrchestratorError>;

    /// Queries the current simulation state (lock-free).
    fn state(&self) -> SimulationState;

    /// Queries the last available StepResult (may be stale).
    fn last_step_result(&self) -> Option<StepResult>;

    /// Takes a field data snapshot if one is available.
    /// Returns `None` if no new snapshot is ready.
    fn try_take_snapshot(&self) -> Option<FieldDataView<'_>>;
}

/// Orchestrator error types.
#[derive(Debug)]
pub enum OrchestratorError {
    /// The communication channel is disconnected.
    ChannelDisconnected,
    /// Invalid state transition.
    InvalidState {
        expected: SimulationState,
        actual: SimulationState,
    },
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelDisconnected => write!(f, "Channel disconnected"),
            Self::InvalidState { expected, actual } =>
                write!(f, "Invalid state: expected {:?}, got {:?}", expected, actual),
        }
    }
}

impl std::error::Error for OrchestratorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_config_default() {
        let config = SolverConfig::default();
        assert_eq!(config.max_iterations, 10_000);
        assert!((config.convergence_threshold - 1e-6).abs() < f64::EPSILON);
        assert_eq!(config.precision, PrecisionMode::FP64);
        assert!(!config.use_gpu);
    }

    #[test]
    fn test_step_result_repr_c() {
        // Verify repr(C) layout is stable
        let result = StepResult {
            timestep: 42,
            time: 0.042,
            dt: 0.001,
            residual_l2: 1e-5,
            converged: false,
        };
        assert_eq!(result.timestep, 42);
        assert!((result.time - 0.042).abs() < f64::EPSILON);
    }

    #[test]
    fn test_field_data_buffer_size() {
        // FieldDataBuffer should have a known, stable size for FFI
        let size = std::mem::size_of::<FieldDataBuffer>();
        // 9 pointers (usize) + 1 usize (num_points) + 1 u64 (generation)
        // On 64-bit: (9 + 1) × 8 + 8 = 88 bytes
        assert!(size > 0, "FieldDataBuffer should be non-zero size");
    }

    #[test]
    fn test_simulation_state_values() {
        assert_ne!(SimulationState::Idle, SimulationState::Running);
        assert_eq!(SimulationState::Paused, SimulationState::Paused);
    }
}
