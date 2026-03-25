//! AeroCore CFD Engine — Core Library
//!
//! High-performance computational fluid dynamics engine supporting:
//! - Lattice Boltzmann Method (LBM) for automotive/F1 aerodynamics
//! - Compressible Navier-Stokes for aeronautical applications
//!
//! # Architecture
//! - **Data-Oriented Design (DOD)**: SoA layouts for cache efficiency
//! - **Zero-allocation hot paths**: All memory pre-allocated via arenas/pools
//! - **Dual precision**: Generic `FloatPrecision` trait (f64 default, f32 opt-in)
//! - **GPU-ready**: Abstracted compute backend (WGPU/CUDA/CPU fallback)

pub mod memory;
pub mod math_core;
pub mod solvers;
pub mod io_mesh;
pub mod gpu_compute;
