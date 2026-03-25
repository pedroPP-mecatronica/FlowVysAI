//! LBM boundary conditions — STUB. Implementation in Sprint S3.
//!
//! # Reference
//! - Bounce-back: standard half-way scheme for no-slip walls
//! - Zou-He: pressure/velocity inlet/outlet

/// LBM boundary condition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LbmBoundaryType {
    /// Half-way bounce-back (no-slip wall). Simple, first-order accurate.
    BounceBack,
    /// Zou-He velocity boundary (inlet).
    ZouHeVelocity,
    /// Zou-He pressure boundary (outlet).
    ZouHePressure,
    /// Periodic boundary (wraps around domain).
    Periodic,
    /// Moving wall (e.g., moving ground plane for F1 simulations).
    MovingWall,
}
