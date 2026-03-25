//! Boundary conditions for compressible Navier-Stokes — STUB.

/// Boundary condition type for the N-S solver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NsBoundaryType {
    /// No-slip wall (v = 0 at surface). Adiabatic or isothermal.
    NoSlipWall,
    /// Moving wall (v = V_wall). For rotating components.
    MovingWall,
    /// Far-field / freestream (characteristic-based).
    FarField,
    /// Pressure outlet (fixed static pressure).
    PressureOutlet,
    /// Symmetry plane (zero normal gradient).
    Symmetry,
}
