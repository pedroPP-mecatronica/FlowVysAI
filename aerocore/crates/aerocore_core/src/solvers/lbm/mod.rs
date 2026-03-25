//! Lattice Boltzmann Method (LBM) solver module — incompressible/transient
//! flows for automotive aerodynamics (F1, WEC).
//!
//! # Scope
//! - D3Q19 velocity model (19 discrete velocities in 3D)
//! - BGK single-relaxation-time collision operator
//! - MRT multi-relaxation-time for stability at high Re
//! - Boundary conditions: bounce-back, Zou-He
//!
//! # Status: STUB (Sprint S3)
//! Module structure ready. Implementation in Phase 1 Sprint S3.

pub mod d3q19;
pub mod collision;
pub mod streaming;
pub mod boundary;
