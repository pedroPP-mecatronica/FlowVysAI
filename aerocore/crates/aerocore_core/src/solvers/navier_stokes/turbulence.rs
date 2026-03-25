//! Turbulence models for RANS — STUB.

/// Available turbulence models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurbulenceModel {
    /// No turbulence model (laminar flow).
    Laminar,
    /// Standard k-ε model (Launder & Spalding, 1974).
    KEpsilon,
    /// k-ω SST model (Menter, 1994) — recommended for aerodynamic flows.
    #[default]
    KOmegaSST,
    /// Spalart-Allmaras one-equation model.
    SpalartAllmaras,
}
