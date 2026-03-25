//! D3Q19 velocity model — 19 discrete velocities in 3D lattice.
//!
//! # Reference
//! - Krüger et al. (2017), "The Lattice Boltzmann Method", Ch. 3
//! - Succi (2018), "The Lattice Boltzmann Equation", Ch. 7

/// Number of discrete velocities in D3Q19.
pub const Q: usize = 19;

/// Discrete velocity vectors eᵢ = (eᵢₓ, eᵢᵧ, eᵢ_z) for D3Q19.
///
/// Ordering: rest, 6 face neighbors, 12 edge neighbors.
pub const E: [[i32; 3]; Q] = [
    // Rest
    [ 0,  0,  0],
    // Face neighbors (±x, ±y, ±z)
    [ 1,  0,  0], [-1,  0,  0],
    [ 0,  1,  0], [ 0, -1,  0],
    [ 0,  0,  1], [ 0,  0, -1],
    // Edge neighbors
    [ 1,  1,  0], [-1, -1,  0],
    [ 1, -1,  0], [-1,  1,  0],
    [ 1,  0,  1], [-1,  0, -1],
    [ 1,  0, -1], [-1,  0,  1],
    [ 0,  1,  1], [ 0, -1, -1],
    [ 0,  1, -1], [ 0, -1,  1],
];

/// Lattice weights wᵢ for D3Q19.
///
/// w₀ = 1/3 (rest)
/// w₁₋₆ = 1/18 (face)
/// w₇₋₁₈ = 1/36 (edge)
pub const W: [f64; Q] = [
    1.0 / 3.0,
    // Face
    1.0 / 18.0, 1.0 / 18.0,
    1.0 / 18.0, 1.0 / 18.0,
    1.0 / 18.0, 1.0 / 18.0,
    // Edge
    1.0 / 36.0, 1.0 / 36.0,
    1.0 / 36.0, 1.0 / 36.0,
    1.0 / 36.0, 1.0 / 36.0,
    1.0 / 36.0, 1.0 / 36.0,
    1.0 / 36.0, 1.0 / 36.0,
    1.0 / 36.0, 1.0 / 36.0,
];

/// Opposite direction index for bounce-back boundary conditions.
///
/// `OPPOSITE[i]` is the index j such that eⱼ = -eᵢ.
pub const OPPOSITE: [usize; Q] = [
    0,     // rest → rest
    2, 1,  // +x ↔ -x
    4, 3,  // +y ↔ -y
    6, 5,  // +z ↔ -z
    8, 7,  // (+x,+y) ↔ (-x,-y)
    10, 9, // (+x,-y) ↔ (-x,+y)
    12, 11,// (+x,+z) ↔ (-x,-z)
    14, 13,// (+x,-z) ↔ (-x,+z)
    16, 15,// (+y,+z) ↔ (-y,-z)
    18, 17,// (+y,-z) ↔ (-y,+z)
];

/// Speed of sound squared: cs² = 1/3 (in lattice units).
pub const CS2: f64 = 1.0 / 3.0;

/// Speed of sound: cs = 1/√3.
pub const CS: f64 = 0.5773502691896258; // 1.0 / sqrt(3.0)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_sum_to_one() {
        let sum: f64 = W.iter().sum();
        assert!((sum - 1.0).abs() < 1e-14, "Weights sum to {} (should be 1.0)", sum);
    }

    #[test]
    fn test_opposite_directions() {
        for i in 0..Q {
            let j = OPPOSITE[i];
            // e_j should be -e_i
            assert_eq!(E[j][0], -E[i][0], "Opposite x mismatch for direction {}", i);
            assert_eq!(E[j][1], -E[i][1], "Opposite y mismatch for direction {}", i);
            assert_eq!(E[j][2], -E[i][2], "Opposite z mismatch for direction {}", i);
        }
    }

    #[test]
    fn test_d3q19_velocity_count() {
        assert_eq!(E.len(), 19);
        assert_eq!(W.len(), 19);
        assert_eq!(OPPOSITE.len(), 19);
    }

    #[test]
    fn test_isotropy_first_moment() {
        // First moment: Σ wᵢ eᵢ = 0 (isotropy condition)
        let mut sum_x = 0.0_f64;
        let mut sum_y = 0.0_f64;
        let mut sum_z = 0.0_f64;
        for i in 0..Q {
            sum_x += W[i] * E[i][0] as f64;
            sum_y += W[i] * E[i][1] as f64;
            sum_z += W[i] * E[i][2] as f64;
        }
        assert!(sum_x.abs() < 1e-14);
        assert!(sum_y.abs() < 1e-14);
        assert!(sum_z.abs() < 1e-14);
    }
}
