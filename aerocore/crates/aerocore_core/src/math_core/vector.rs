//! 3D vector type — the fundamental building block for velocity, position,
//! force, and gradient fields in CFD simulations.
//!
//! # Design
//! - `repr(C)` for FFI compatibility (direct mapping to C struct)
//! - Generic over `FloatPrecision` (f32/f64)
//! - Implements `Copy`, `Clone`, basic arithmetic ops

use crate::math_core::precision::FloatPrecision;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 3D vector: velocity, position, force, gradient, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec3<T: FloatPrecision> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: FloatPrecision> Vec3<T> {
    /// Creates a new vector from components.
    #[inline]
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// Zero vector (0, 0, 0).
    #[inline]
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// Unit vector along X axis (1, 0, 0).
    #[inline]
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }

    /// Unit vector along Y axis (0, 1, 0).
    #[inline]
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }

    /// Unit vector along Z axis (0, 0, 1).
    #[inline]
    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// Dot product: a · b = ax*bx + ay*by + az*bz
    #[inline]
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product: a × b
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Squared magnitude: |v|² (avoids sqrt for comparisons).
    #[inline]
    pub fn magnitude_squared(&self) -> T {
        self.dot(self)
    }

    /// Magnitude: |v| = √(x² + y² + z²)
    #[inline]
    pub fn magnitude(&self) -> T {
        self.magnitude_squared().sqrt()
    }

    /// Returns a normalized (unit length) version of this vector.
    /// Returns zero vector if magnitude is near zero.
    #[inline]
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag < T::EPSILON {
            Self::zero()
        } else {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        }
    }

    /// Scalar multiplication: v * s
    #[inline]
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// Component-wise absolute value.
    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    /// Component-wise maximum.
    #[inline]
    pub fn max_component(&self) -> T {
        self.x.max(self.y).max(self.z)
    }

    /// Component-wise minimum.
    #[inline]
    pub fn min_component(&self) -> T {
        self.x.min(self.y).min(self.z)
    }

    /// Linear interpolation: lerp(a, b, t) = a + t*(b - a)
    #[inline]
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
            self.z + t * (other.z - self.z),
        )
    }
}

impl<T: FloatPrecision> Default for Vec3<T> {
    fn default() -> Self {
        Self::zero()
    }
}

// Arithmetic operator overloads

impl<T: FloatPrecision> Add for Vec3<T> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: FloatPrecision> Sub for Vec3<T> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: FloatPrecision> Mul<T> for Vec3<T> {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: T) -> Self {
        self.scale(scalar)
    }
}

impl<T: FloatPrecision> Neg for Vec3<T> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl<T: FloatPrecision> AddAssign for Vec3<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T: FloatPrecision> SubAssign for Vec3<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T: FloatPrecision> MulAssign<T> for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, scalar: T) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type V3 = Vec3<f64>;

    #[test]
    fn test_vec3_dot_product() {
        let a = V3::new(1.0, 2.0, 3.0);
        let b = V3::new(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 32
        assert!((a.dot(&b) - 32.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_vec3_cross_product() {
        let a = V3::unit_x();
        let b = V3::unit_y();
        let c = a.cross(&b);
        // x × y = z
        assert!((c.x - 0.0).abs() < f64::EPSILON);
        assert!((c.y - 0.0).abs() < f64::EPSILON);
        assert!((c.z - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = V3::new(3.0, 4.0, 0.0);
        let n = v.normalized();
        assert!((n.magnitude() - 1.0).abs() < 1e-14);
        assert!((n.x - 0.6).abs() < 1e-14);
        assert!((n.y - 0.8).abs() < 1e-14);
    }

    #[test]
    fn test_vec3_magnitude() {
        let v = V3::new(1.0, 2.0, 2.0);
        assert!((v.magnitude() - 3.0).abs() < 1e-14);
    }

    #[test]
    fn test_vec3_arithmetic() {
        let a = V3::new(1.0, 2.0, 3.0);
        let b = V3::new(4.0, 5.0, 6.0);

        let sum = a + b;
        assert!((sum.x - 5.0).abs() < f64::EPSILON);

        let diff = b - a;
        assert!((diff.x - 3.0).abs() < f64::EPSILON);

        let scaled = a * 2.0;
        assert!((scaled.x - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_vec3_lerp() {
        let a = V3::new(0.0, 0.0, 0.0);
        let b = V3::new(10.0, 20.0, 30.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < f64::EPSILON);
        assert!((mid.y - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_vec3_zero_normalize() {
        let v = V3::zero();
        let n = v.normalized();
        assert!((n.magnitude() - 0.0).abs() < f64::EPSILON);
    }
}
