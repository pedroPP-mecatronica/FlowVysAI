//! D2Q9 velocity model — 9 discrete velocities in 2D lattice.
//!
//! # Reference
//! - Krüger et al. (2017), "The Lattice Boltzmann Method", Ch. 3
//! - Succi (2018), "The Lattice Boltzmann Equation", Ch. 7

/// Number of discrete velocities in D2Q9.
pub const Q: usize = 9;

/// Discrete velocity vectors eᵢ = (eᵢₓ, eᵢᵧ) for D2Q9.
///
/// Ordering: rest, 4 cardinal (±x, ±y), 4 diagonal.
pub const E: [[i32; 2]; Q] = [
    [0, 0],   // 0: rest
    [1, 0],   // 1: right
    [0, 1],   // 2: up
    [-1, 0],  // 3: left
    [0, -1],  // 4: down
    [1, 1],   // 5: right-up
    [-1, 1],  // 6: left-up
    [-1, -1], // 7: left-down
    [1, -1],  // 8: right-down
];

/// Lattice weights wᵢ for D2Q9.
///
/// w₀ = 4/9  (rest)
/// w₁₋₄ = 1/9  (cardinal)
/// w₅₋₈ = 1/36 (diagonal)
pub const W: [f64; Q] = [
    4.0 / 9.0,  // rest
    1.0 / 9.0,  // right
    1.0 / 9.0,  // up
    1.0 / 9.0,  // left
    1.0 / 9.0,  // down
    1.0 / 36.0, // right-up
    1.0 / 36.0, // left-up
    1.0 / 36.0, // left-down
    1.0 / 36.0, // right-down
];

/// Opposite direction index for bounce-back boundary conditions.
///
/// `OPPOSITE[i]` is the index j such that eⱼ = −eᵢ.
pub const OPPOSITE: [usize; Q] = [
    0, // rest → rest
    3, // right ↔ left
    4, // up ↔ down
    1, // left ↔ right
    2, // down ↔ up
    7, // right-up ↔ left-down
    8, // left-up ↔ right-down
    5, // left-down ↔ right-up
    6, // right-down ↔ left-up
];

/// Speed of sound squared: cs² = 1/3 (in lattice units).
pub const CS2: f64 = 1.0 / 3.0;

/// Speed of sound: cs = 1/√3.
pub const CS: f64 = 0.577_350_269_189_625_8; // 1.0 / sqrt(3.0)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_sum_to_one() {
        let sum: f64 = W.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-14,
            "D2Q9 weights sum to {sum} (expected 1.0)"
        );
    }

    #[test]
    fn opposite_directions_are_negatives() {
        for i in 0..Q {
            let j = OPPOSITE[i];
            assert_eq!(E[j][0], -E[i][0], "OPPOSITE x mismatch at i={i}");
            assert_eq!(E[j][1], -E[i][1], "OPPOSITE y mismatch at i={i}");
        }
    }

    #[test]
    fn velocity_count_is_nine() {
        assert_eq!(E.len(), 9);
        assert_eq!(W.len(), 9);
        assert_eq!(OPPOSITE.len(), 9);
    }

    #[test]
    fn isotropy_first_moment() {
        // Σ wᵢ eᵢ = 0 (isotropy condition)
        let mut sx = 0.0_f64;
        let mut sy = 0.0_f64;
        for i in 0..Q {
            sx += W[i] * E[i][0] as f64;
            sy += W[i] * E[i][1] as f64;
        }
        assert!(sx.abs() < 1e-14, "first moment x = {sx}");
        assert!(sy.abs() < 1e-14, "first moment y = {sy}");
    }

    #[test]
    fn isotropy_second_moment() {
        // Σ wᵢ eᵢₓ eᵢᵧ = 0  and  Σ wᵢ eᵢₓ² = cs²
        let mut sxx = 0.0_f64;
        let mut syy = 0.0_f64;
        let mut sxy = 0.0_f64;
        for i in 0..Q {
            let ex = E[i][0] as f64;
            let ey = E[i][1] as f64;
            sxx += W[i] * ex * ex;
            syy += W[i] * ey * ey;
            sxy += W[i] * ex * ey;
        }
        assert!((sxx - CS2).abs() < 1e-14, "Σ w eₓ² = {sxx}");
        assert!((syy - CS2).abs() < 1e-14, "Σ w eᵧ² = {syy}");
        assert!(sxy.abs() < 1e-14, "Σ w eₓeᵧ = {sxy}");
    }
}
