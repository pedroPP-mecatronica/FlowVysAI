//! Symmetric 3×3 tensor — for stress, strain rate, and Reynolds stress tensors.
//!
//! # Memory Optimization (DOD)
//! A symmetric tensor has only 6 independent components (instead of 9).
//! This saves 33% memory for large fields and improves cache utilization.
//!
//! Layout: [xx, yy, zz, xy, xz, yz] (Voigt notation)

use crate::math_core::matrix::Mat3x3;
use crate::math_core::precision::FloatPrecision;

/// Symmetric 3×3 tensor stored in Voigt notation (6 components).
///
/// ```text
/// | xx  xy  xz |
/// | xy  yy  yz |  →  stored as [xx, yy, zz, xy, xz, yz]
/// | xz  yz  zz |
/// ```
///
/// Used for:
/// - Viscous stress tensor τ
/// - Strain rate tensor S = ½(∇u + ∇uᵀ)
/// - Reynolds stress tensor in RANS turbulence models
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct SymmetricTensor3<T: FloatPrecision> {
    pub xx: T,
    pub yy: T,
    pub zz: T,
    pub xy: T,
    pub xz: T,
    pub yz: T,
}

impl<T: FloatPrecision> SymmetricTensor3<T> {
    /// Creates a new symmetric tensor from 6 independent components.
    #[inline]
    pub fn new(xx: T, yy: T, zz: T, xy: T, xz: T, yz: T) -> Self {
        Self {
            xx,
            yy,
            zz,
            xy,
            xz,
            yz,
        }
    }

    /// Zero tensor.
    #[inline]
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO, T::ZERO, T::ZERO, T::ZERO)
    }

    /// Isotropic tensor: p × I (pressure tensor, for example).
    #[inline]
    pub fn isotropic(p: T) -> Self {
        Self::new(p, p, p, T::ZERO, T::ZERO, T::ZERO)
    }

    /// Trace: tr(σ) = σ_xx + σ_yy + σ_zz
    ///
    /// In fluid mechanics: tr(stress) = -3p for Newtonian fluids.
    #[inline]
    pub fn trace(&self) -> T {
        self.xx + self.yy + self.zz
    }

    /// Deviatoric part: σ' = σ - (1/3)tr(σ)I
    ///
    /// The deviatoric stress drives deformation in incompressible flow.
    #[inline]
    pub fn deviatoric(&self) -> Self {
        let third = T::ONE / (T::ONE + T::ONE + T::ONE);
        let p = self.trace() * third;
        Self::new(
            self.xx - p,
            self.yy - p,
            self.zz - p,
            self.xy,
            self.xz,
            self.yz,
        )
    }

    /// Von Mises equivalent stress: σ_vm = √(3/2 × S:S)
    ///
    /// Where S is the deviatoric tensor. Used in structural/FSI analysis.
    pub fn von_mises(&self) -> T {
        let dev = self.deviatoric();
        let s_s = dev.xx * dev.xx
            + dev.yy * dev.yy
            + dev.zz * dev.zz
            + T::TWO * (dev.xy * dev.xy + dev.xz * dev.xz + dev.yz * dev.yz);
        let three_half = (T::ONE + T::ONE + T::ONE) / T::TWO;
        (three_half * s_s).sqrt()
    }

    /// Double contraction: A:B = Σᵢⱼ Aᵢⱼ Bᵢⱼ
    ///
    /// For symmetric tensors: A:B = xx*xx + yy*yy + zz*zz + 2*(xy*xy + xz*xz + yz*yz)
    #[inline]
    pub fn double_contraction(&self, other: &Self) -> T {
        self.xx * other.xx
            + self.yy * other.yy
            + self.zz * other.zz
            + T::TWO * (self.xy * other.xy + self.xz * other.xz + self.yz * other.yz)
    }

    /// Converts to a full 3×3 matrix (expanding the symmetry).
    #[inline]
    pub fn to_mat3x3(&self) -> Mat3x3<T> {
        Mat3x3::new(
            self.xx, self.xy, self.xz, self.xy, self.yy, self.yz, self.xz, self.yz, self.zz,
        )
    }

    /// Creates from a 3×3 matrix by symmetrizing: S = ½(M + Mᵀ)
    ///
    /// This is the strain rate tensor when M = ∇u.
    #[inline]
    pub fn from_mat3x3_symmetric(m: &Mat3x3<T>) -> Self {
        let half = T::ONE / T::TWO;
        Self::new(
            m.at(0, 0),
            m.at(1, 1),
            m.at(2, 2),
            half * (m.at(0, 1) + m.at(1, 0)),
            half * (m.at(0, 2) + m.at(2, 0)),
            half * (m.at(1, 2) + m.at(2, 1)),
        )
    }

    /// Scalar multiplication.
    #[inline]
    pub fn scale(&self, s: T) -> Self {
        Self::new(
            self.xx * s,
            self.yy * s,
            self.zz * s,
            self.xy * s,
            self.xz * s,
            self.yz * s,
        )
    }
}

impl<T: FloatPrecision> Default for SymmetricTensor3<T> {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type ST = SymmetricTensor3<f64>;

    #[test]
    fn test_trace() {
        let t = ST::new(1.0, 2.0, 3.0, 0.0, 0.0, 0.0);
        assert!((t.trace() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_isotropic() {
        let t = ST::isotropic(5.0);
        assert!((t.trace() - 15.0).abs() < f64::EPSILON);
        assert!((t.xy - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deviatoric_trace_zero() {
        let t = ST::new(10.0, 20.0, 30.0, 1.0, 2.0, 3.0);
        let dev = t.deviatoric();
        // Deviatoric tensor always has trace = 0
        assert!(dev.trace().abs() < 1e-12);
    }

    #[test]
    fn test_von_mises_isotropic_is_zero() {
        let t = ST::isotropic(100.0);
        // Isotropic stress has zero deviatoric → zero von Mises
        assert!(t.von_mises().abs() < 1e-10);
    }

    #[test]
    fn test_roundtrip_mat3x3() {
        let t = ST::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let m = t.to_mat3x3();
        let t2 = ST::from_mat3x3_symmetric(&m);
        assert!((t2.xx - t.xx).abs() < f64::EPSILON);
        assert!((t2.xy - t.xy).abs() < f64::EPSILON);
        assert!((t2.yz - t.yz).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_savings() {
        // Symmetric tensor uses 6 components vs 9 for full matrix = 33% savings
        assert_eq!(std::mem::size_of::<ST>(), 6 * std::mem::size_of::<f64>());
    }
}
