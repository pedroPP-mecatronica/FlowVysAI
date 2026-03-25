//! LBM collision operators — STUB. Implementation in Sprint S3.
//!
//! # Reference
//! - BGK: Bhatnagar, Gross & Krook (1954), Phys. Rev. 94(3)
//! - MRT: d'Humières (2002), Phil. Trans. R. Soc.

/// Collision operator type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionOperator {
    /// Single relaxation time (Bhatnagar-Gross-Krook).
    /// Simple, fast, but can be unstable at low viscosity.
    BGK,
    /// Multiple relaxation time.
    /// More stable at high Reynolds numbers, slightly more expensive.
    MRT,
}

impl Default for CollisionOperator {
    fn default() -> Self {
        Self::BGK
    }
}
