//! 3×3 matrix — for velocity gradients, rotation tensors, and
//! coordinate transformations in CFD simulations.
//!
//! # Layout
//! Row-major storage in a flat `[T; 9]` array for cache-friendly access.
//! `repr(C)` for FFI compatibility.

use crate::math_core::precision::FloatPrecision;
use crate::math_core::vector::Vec3;
use std::ops::{Mul, Add};

/// 3×3 matrix stored in row-major order.
///
/// Used for: velocity gradient tensor ∇u, rotation matrices,
/// deformation rate tensor, Reynolds stress tensor.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3x3<T: FloatPrecision> {
    /// Row-major: [m00, m01, m02, m10, m11, m12, m20, m21, m22]
    pub data: [T; 9],
}

impl<T: FloatPrecision> Mat3x3<T> {
    /// Creates a matrix from 9 elements in row-major order.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m00: T, m01: T, m02: T,
        m10: T, m11: T, m12: T,
        m20: T, m21: T, m22: T,
    ) -> Self {
        Self {
            data: [m00, m01, m02, m10, m11, m12, m20, m21, m22],
        }
    }

    /// Identity matrix.
    #[inline]
    pub fn identity() -> Self {
        Self::new(
            T::ONE,  T::ZERO, T::ZERO,
            T::ZERO, T::ONE,  T::ZERO,
            T::ZERO, T::ZERO, T::ONE,
        )
    }

    /// Zero matrix.
    #[inline]
    pub fn zero() -> Self {
        Self {
            data: [T::ZERO; 9],
        }
    }

    /// Access element at (row, col), zero-indexed.
    #[inline]
    pub fn at(&self, row: usize, col: usize) -> T {
        self.data[row * 3 + col]
    }

    /// Mutable access to element at (row, col).
    #[inline]
    pub fn at_mut(&mut self, row: usize, col: usize) -> &mut T {
        &mut self.data[row * 3 + col]
    }

    /// Matrix-vector multiplication: M × v
    #[inline]
    pub fn mul_vec(&self, v: &Vec3<T>) -> Vec3<T> {
        Vec3::new(
            self.at(0, 0) * v.x + self.at(0, 1) * v.y + self.at(0, 2) * v.z,
            self.at(1, 0) * v.x + self.at(1, 1) * v.y + self.at(1, 2) * v.z,
            self.at(2, 0) * v.x + self.at(2, 1) * v.y + self.at(2, 2) * v.z,
        )
    }

    /// Transpose: Mᵀ
    #[inline]
    pub fn transpose(&self) -> Self {
        Self::new(
            self.at(0, 0), self.at(1, 0), self.at(2, 0),
            self.at(0, 1), self.at(1, 1), self.at(2, 1),
            self.at(0, 2), self.at(1, 2), self.at(2, 2),
        )
    }

    /// Determinant: det(M)
    #[inline]
    pub fn determinant(&self) -> T {
        let a = self.at(0, 0);
        let b = self.at(0, 1);
        let c = self.at(0, 2);
        let d = self.at(1, 0);
        let e = self.at(1, 1);
        let f = self.at(1, 2);
        let g = self.at(2, 0);
        let h = self.at(2, 1);
        let i = self.at(2, 2);

        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    /// Trace: tr(M) = m00 + m11 + m22
    #[inline]
    pub fn trace(&self) -> T {
        self.at(0, 0) + self.at(1, 1) + self.at(2, 2)
    }

    /// Inverse: M⁻¹. Returns `None` if the matrix is singular (det ≈ 0).
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < T::EPSILON {
            return None;
        }

        let inv_det = T::ONE / det;

        let a = self.at(0, 0);
        let b = self.at(0, 1);
        let c = self.at(0, 2);
        let d = self.at(1, 0);
        let e = self.at(1, 1);
        let f = self.at(1, 2);
        let g = self.at(2, 0);
        let h = self.at(2, 1);
        let i = self.at(2, 2);

        Some(Self::new(
            (e * i - f * h) * inv_det,
            (c * h - b * i) * inv_det,
            (b * f - c * e) * inv_det,
            (f * g - d * i) * inv_det,
            (a * i - c * g) * inv_det,
            (c * d - a * f) * inv_det,
            (d * h - e * g) * inv_det,
            (b * g - a * h) * inv_det,
            (a * e - b * d) * inv_det,
        ))
    }

    /// Frobenius norm: ‖M‖_F = √(Σ mij²)
    #[inline]
    pub fn frobenius_norm(&self) -> T {
        let mut sum = T::ZERO;
        for &v in &self.data {
            sum += v * v;
        }
        sum.sqrt()
    }

    /// Scalar multiplication: s × M
    #[inline]
    pub fn scale(&self, scalar: T) -> Self {
        let mut result = *self;
        for v in &mut result.data {
            *v *= scalar;
        }
        result
    }
}

impl<T: FloatPrecision> Default for Mat3x3<T> {
    fn default() -> Self {
        Self::identity()
    }
}

// Matrix × Matrix
impl<T: FloatPrecision> Mul for Mat3x3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = T::ZERO;
                for k in 0..3 {
                    sum += self.at(i, k) * rhs.at(k, j);
                }
                *result.at_mut(i, j) = sum;
            }
        }
        result
    }
}

// Matrix + Matrix
impl<T: FloatPrecision> Add for Mat3x3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..9 {
            result.data[i] = self.data[i] + rhs.data[i];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type M3 = Mat3x3<f64>;
    type V3 = Vec3<f64>;

    #[test]
    fn test_identity() {
        let i = M3::identity();
        assert!((i.determinant() - 1.0).abs() < f64::EPSILON);
        assert!((i.trace() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mat3x3_inverse() {
        let m = M3::new(
            1.0, 2.0, 3.0,
            0.0, 1.0, 4.0,
            5.0, 6.0, 0.0,
        );

        let inv = m.inverse().expect("Matrix should be invertible");
        let product = m * inv;

        // A × A⁻¹ ≈ I
        let identity = M3::identity();
        for i in 0..9 {
            assert!(
                (product.data[i] - identity.data[i]).abs() < 1e-10,
                "Element {} differs: {} vs {}",
                i, product.data[i], identity.data[i]
            );
        }
    }

    #[test]
    fn test_singular_matrix_no_inverse() {
        let m = M3::new(
            1.0, 2.0, 3.0,
            2.0, 4.0, 6.0, // Row 2 = 2 * Row 1
            0.0, 0.0, 0.0,
        );
        assert!(m.inverse().is_none());
    }

    #[test]
    fn test_mul_vec() {
        let m = M3::identity();
        let v = V3::new(1.0, 2.0, 3.0);
        let result = m.mul_vec(&v);
        assert!((result.x - 1.0).abs() < f64::EPSILON);
        assert!((result.y - 2.0).abs() < f64::EPSILON);
        assert!((result.z - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transpose() {
        let m = M3::new(
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0,
        );
        let t = m.transpose();
        assert!((t.at(0, 1) - 4.0).abs() < f64::EPSILON);
        assert!((t.at(1, 0) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_determinant() {
        let m = M3::new(
            6.0, 1.0, 1.0,
            4.0, -2.0, 5.0,
            2.0, 8.0, 7.0,
        );
        // det = 6*(-2*7 - 5*8) - 1*(4*7 - 5*2) + 1*(4*8 - (-2)*2)
        //     = 6*(-14-40) - 1*(28-10) + 1*(32+4)
        //     = 6*(-54) - 18 + 36 = -324 - 18 + 36 = -306
        assert!((m.determinant() - (-306.0)).abs() < 1e-10);
    }
}
