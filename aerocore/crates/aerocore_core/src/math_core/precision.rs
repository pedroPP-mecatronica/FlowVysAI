//! Float precision trait — generic over f32/f64 for the entire solver stack.
//!
//! # Steering #3: Precision Numérica
//! Default is `f64` (strict double precision). `f32` is opt-in only
//! when `PrecisionMode::FP32` is explicitly configured for fast GPU execution.
//!
//! # Usage
//! All solver internals are generic over `T: FloatPrecision`:
//! ```ignore
//! fn compute_residual<T: FloatPrecision>(field: &[T]) -> T { ... }
//! ```

use num_traits::{Float, FromPrimitive, NumAssign};
use std::fmt::{Debug, Display};

/// Trait abstracting numeric precision across the entire engine.
///
/// Provides associated constants for common values and constrains
/// the type to be FFI-safe, thread-safe, and SIMD-friendly.
pub trait FloatPrecision:
    Float
    + NumAssign
    + FromPrimitive
    + Debug
    + Display
    + Send
    + Sync
    + Copy
    + Default
    + 'static
{
    /// Additive identity: 0.0
    const ZERO: Self;
    /// Multiplicative identity: 1.0
    const ONE: Self;
    /// Two: 2.0 (frequently used in CFD: ½ρv², 2Δx, etc.)
    const TWO: Self;
    /// Machine epsilon for this precision
    const EPSILON: Self;
    /// π — used in cylindrical/spherical coordinates, vortex models
    const PI: Self;
    /// Human-readable name for logging/UI
    const NAME: &'static str;
    /// Size in bytes (8 for f64, 4 for f32)
    const BYTES: usize;
}

impl FloatPrecision for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const EPSILON: Self = f64::EPSILON;
    const PI: Self = std::f64::consts::PI;
    const NAME: &'static str = "f64";
    const BYTES: usize = 8;
}

impl FloatPrecision for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const EPSILON: Self = f32::EPSILON;
    const PI: Self = std::f32::consts::PI;
    const NAME: &'static str = "f32";
    const BYTES: usize = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_precision_f64() {
        assert!((f64::ZERO - 0.0).abs() < f64::EPSILON);
        assert!((f64::ONE - 1.0).abs() < f64::EPSILON);
        assert!((f64::PI - std::f64::consts::PI).abs() < f64::EPSILON);
        assert_eq!(f64::NAME, "f64");
        assert_eq!(f64::BYTES, 8);
    }

    #[test]
    fn test_float_precision_f32() {
        assert!((f32::ZERO - 0.0).abs() < f32::EPSILON);
        assert!((f32::ONE - 1.0).abs() < f32::EPSILON);
        assert!((f32::PI - std::f32::consts::PI).abs() < f32::EPSILON);
        assert_eq!(f32::NAME, "f32");
        assert_eq!(f32::BYTES, 4);
    }

    fn generic_add<T: FloatPrecision>(a: T, b: T) -> T {
        a + b
    }

    #[test]
    fn test_generic_usage() {
        let result_f64 = generic_add(1.0_f64, 2.0_f64);
        assert!((result_f64 - 3.0).abs() < f64::EPSILON);

        let result_f32 = generic_add(1.0_f32, 2.0_f32);
        assert!((result_f32 - 3.0).abs() < f32::EPSILON);
    }
}
