//! Navier-Stokes solver module — compressible flow for aeronautical applications.
//!
//! # Scope
//! - Compressible Navier-Stokes equations (Euler + viscous terms)
//! - Roe / HLLC approximate Riemann solvers for flux computation
//! - Turbulence models: k-ε, k-ω SST
//! - Boundary conditions: no-slip wall, far-field, pressure outlet
//!
//! # Status: STUB (Sprint S9)
//! This module contains placeholder types. Implementation is scheduled
//! for Phase 3 — Advanced Aero (Sprint S9).

pub mod compressible;
pub mod boundary;
pub mod turbulence;
