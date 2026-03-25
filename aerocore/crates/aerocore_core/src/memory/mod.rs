//! Memory management subsystem — arena allocators, object pools, aligned storage.
//!
//! # Steering Compliance
//! All hot-path data MUST be allocated during `Solver::init()` using these primitives.
//! Zero dynamic allocation is permitted inside `Solver::step()`.

pub mod arena;
pub mod pool;
pub mod aligned;
