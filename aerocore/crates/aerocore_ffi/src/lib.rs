//! AeroCore FFI — C ABI Foreign Function Interface layer.
//!
//! Exposes the core engine functionality through `extern "C"` functions
//! for consumption by the UI (Rust-native or any language with C FFI).
//!
//! # OKR 3 — KR 3.2: Decoupled frontend communication

pub mod types;

use aerocore_core::solvers::traits::*;

/// Returns the AeroCore engine version as a C string.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program.
#[no_mangle]
pub extern "C" fn aerocore_version() -> *const std::ffi::c_char {
    // Static string — valid for the entire program lifetime
    static VERSION: &[u8] = b"AeroCore CFD Engine v0.1.0\0";
    VERSION.as_ptr() as *const std::ffi::c_char
}

/// Returns the default solver configuration.
#[no_mangle]
pub extern "C" fn aerocore_default_config() -> types::FfiSolverConfig {
    let config = SolverConfig::default();
    types::FfiSolverConfig::from_rust(&config)
}

/// Returns the current simulation state.
///
/// Thread-safe, lock-free call.
#[no_mangle]
pub extern "C" fn aerocore_get_state() -> types::FfiSimulationState {
    // TODO: Connect to actual orchestrator
    types::FfiSimulationState::Idle
}
