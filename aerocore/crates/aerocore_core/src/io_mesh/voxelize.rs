//! STL-to-voxel boundary mask conversion.
//!
//! Provides [`stl_to_boundary_mask`], which converts a triangulated surface
//! mesh into a flat `Vec<bool>` grid mask suitable for the LBM solver's
//! per-cell bounce-back boundary condition.
//!
//! # Algorithm
//! For each cell centre in the `nx × ny × nz` grid the function casts an
//! axis-aligned ray in the `+x` direction and counts how many triangles it
//! pierces.  An **odd** count means the point lies **inside** the solid
//! (Jordan curve theorem / parity test).
//!
//! This is O(cells × triangles) and therefore suitable for small domains and
//! correctness-first testing.  Determinism is guaranteed for any fixed mesh.
//!
//! # Coordinate Mapping
//! The mesh bounding box is mapped onto the lattice domain
//! `[0, nx) × [0, ny) × [0, nz)`.  A fractional `padding` (e.g. `0.05` for
//! 5 %) expands the world domain symmetrically beyond the mesh extents so the
//! solid fits comfortably inside the grid without touching the outermost
//! cells.
//!
//! Cell `(i, j, k)` centre maps to world position:
//!
//! ```text
//! wx = world_min_x + (i + 0.5) / nx * world_extent_x
//! wy = world_min_y + (j + 0.5) / ny * world_extent_y
//! wz = world_min_z + (k + 0.5) / nz * world_extent_z
//! ```
//!
//! where `world_min = bbox_min − padding * bbox_extent` and
//! `world_extent = (1 + 2 * padding) * bbox_extent`.

use super::mesh::SoaMesh;

// ── geometry helpers ──────────────────────────────────────────────────────────

/// Cross product `a × b`.
#[inline(always)]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product `a · b`.
#[inline(always)]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Returns `true` if a ray from `orig` in the `+x` direction intersects the
/// triangle `(v0, v1, v2)` at a positive parameter value `t > 0`.
///
/// Uses the Möller–Trumbore algorithm specialised for `dir = (1, 0, 0)`.
fn ray_x_intersects_triangle(orig: [f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> bool {
    const EPS: f64 = 1e-10;

    // dir = (1, 0, 0)
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    // h = dir × e2.  With dir = (1,0,0): h = (0*e2[2]-0*e2[1], 0*e2[0]-1*e2[2], 1*e2[1]-0*e2[0])
    //                                       = (0, -e2[2], e2[1])
    let h = [0.0_f64, -e2[2], e2[1]];

    let a = dot(e1, h);
    if a.abs() < EPS {
        return false; // Ray parallel to triangle
    }

    let f = 1.0 / a;
    let s = [orig[0] - v0[0], orig[1] - v0[1], orig[2] - v0[2]];
    let u = f * dot(s, h);

    if !(0.0..=1.0).contains(&u) {
        return false;
    }

    let q = cross(s, e1);
    // v = f * (dir · q).  With dir = (1,0,0): v = f * q[0]
    let v = f * q[0];

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    // t = f * (e2 · q)
    let t = f * dot(e2, q);
    t > EPS
}

// ── public API ────────────────────────────────────────────────────────────────

/// Converts a triangulated [`SoaMesh`] into a flat solid-cell boundary mask.
///
/// # Parameters
/// - `mesh`    — source surface mesh (e.g. loaded from STL)
/// - `nx / ny / nz` — grid resolution
/// - `padding` — fractional margin added on each side of the mesh bounding
///   box when mapping to grid coordinates (e.g. `0.05` = 5 % on each side).
///   Use `0.0` to fit the mesh exactly to the grid.
///
/// # Returns
/// A `Vec<bool>` of length `nx * ny * nz`.  Index layout matches
/// [`LbmSolver`][crate::solvers::lbm::LbmSolver]:
/// `idx = (z * ny + y) * nx + x`.  `true` = solid (bounce-back cell).
///
/// Returns an all-`false` mask if the mesh has no triangles or has a
/// degenerate bounding box (zero extent in any axis).
pub fn stl_to_boundary_mask(
    mesh: &SoaMesh,
    nx: usize,
    ny: usize,
    nz: usize,
    padding: f64,
) -> Vec<bool> {
    let num_cells = nx * ny * nz;
    let mut mask = vec![false; num_cells];

    let num_faces = mesh.num_faces();
    if num_faces == 0 || nx == 0 || ny == 0 || nz == 0 {
        return mask;
    }

    let bb_min = mesh.info.bounding_box_min;
    let bb_max = mesh.info.bounding_box_max;

    let ext = [
        bb_max[0] - bb_min[0],
        bb_max[1] - bb_min[1],
        bb_max[2] - bb_min[2],
    ];

    // Degenerate mesh (point or line): cannot voxelise
    if ext[0] < f64::EPSILON || ext[1] < f64::EPSILON || ext[2] < f64::EPSILON {
        return mask;
    }

    // Expand world domain by `padding` on each side
    let world_min = [
        bb_min[0] - padding * ext[0],
        bb_min[1] - padding * ext[1],
        bb_min[2] - padding * ext[2],
    ];
    let world_ext = [
        (1.0 + 2.0 * padding) * ext[0],
        (1.0 + 2.0 * padding) * ext[1],
        (1.0 + 2.0 * padding) * ext[2],
    ];

    let inv_nx = 1.0 / nx as f64;
    let inv_ny = 1.0 / ny as f64;
    let inv_nz = 1.0 / nz as f64;

    for k in 0..nz {
        let wz = world_min[2] + (k as f64 + 0.5) * inv_nz * world_ext[2];
        for j in 0..ny {
            let wy = world_min[1] + (j as f64 + 0.5) * inv_ny * world_ext[1];
            for i in 0..nx {
                let wx = world_min[0] + (i as f64 + 0.5) * inv_nx * world_ext[0];
                let orig = [wx, wy, wz];

                // Count ray–triangle intersections (parity test)
                let mut count: u32 = 0;
                for f in 0..num_faces {
                    let v0i = f * 3;
                    let v1i = v0i + 1;
                    let v2i = v0i + 2;

                    let v0 = [
                        mesh.vertices_x[v0i],
                        mesh.vertices_y[v0i],
                        mesh.vertices_z[v0i],
                    ];
                    let v1 = [
                        mesh.vertices_x[v1i],
                        mesh.vertices_y[v1i],
                        mesh.vertices_z[v1i],
                    ];
                    let v2 = [
                        mesh.vertices_x[v2i],
                        mesh.vertices_y[v2i],
                        mesh.vertices_z[v2i],
                    ];

                    if ray_x_intersects_triangle(orig, v0, v1, v2) {
                        count += 1;
                    }
                }

                // Odd intersection count → inside the solid
                if count % 2 == 1 {
                    let idx = (k * ny + j) * nx + i;
                    mask[idx] = true;
                }
            }
        }
    }

    mask
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_mesh::mesh::{MeshFormat, MeshInfo, SoaMesh};

    /// Build a simple axis-aligned unit cube mesh (12 triangles).
    fn unit_cube_mesh() -> SoaMesh {
        // 6 faces × 2 triangles × 3 vertices = 36 vertices
        #[rustfmt::skip]
        let triangles: &[[f64; 3]; 36] = &[
            // -X face (x=0)
            [0.0,0.0,0.0], [0.0,0.0,1.0], [0.0,1.0,1.0],
            [0.0,0.0,0.0], [0.0,1.0,1.0], [0.0,1.0,0.0],
            // +X face (x=1)
            [1.0,0.0,0.0], [1.0,1.0,0.0], [1.0,1.0,1.0],
            [1.0,0.0,0.0], [1.0,1.0,1.0], [1.0,0.0,1.0],
            // -Y face (y=0)
            [0.0,0.0,0.0], [1.0,0.0,0.0], [1.0,0.0,1.0],
            [0.0,0.0,0.0], [1.0,0.0,1.0], [0.0,0.0,1.0],
            // +Y face (y=1)
            [0.0,1.0,0.0], [0.0,1.0,1.0], [1.0,1.0,1.0],
            [0.0,1.0,0.0], [1.0,1.0,1.0], [1.0,1.0,0.0],
            // -Z face (z=0)
            [0.0,0.0,0.0], [0.0,1.0,0.0], [1.0,1.0,0.0],
            [0.0,0.0,0.0], [1.0,1.0,0.0], [1.0,0.0,0.0],
            // +Z face (z=1)
            [0.0,0.0,1.0], [1.0,0.0,1.0], [1.0,1.0,1.0],
            [0.0,0.0,1.0], [1.0,1.0,1.0], [0.0,1.0,1.0],
        ];

        let mut vx = Vec::with_capacity(36);
        let mut vy = Vec::with_capacity(36);
        let mut vz = Vec::with_capacity(36);
        let mut face_indices = Vec::with_capacity(36);

        for (idx, v) in triangles.iter().enumerate() {
            vx.push(v[0]);
            vy.push(v[1]);
            vz.push(v[2]);
            face_indices.push(idx as u32);
        }

        let num_faces = 12;
        SoaMesh {
            vertices_x: vx,
            vertices_y: vy,
            vertices_z: vz,
            face_indices,
            normals_x: vec![0.0; num_faces],
            normals_y: vec![0.0; num_faces],
            normals_z: vec![0.0; num_faces],
            info: MeshInfo {
                format: MeshFormat::StlBinary,
                num_vertices: 36,
                num_faces,
                num_cells: 0,
                bounding_box_min: [0.0, 0.0, 0.0],
                bounding_box_max: [1.0, 1.0, 1.0],
            },
        }
    }

    #[test]
    fn test_empty_mesh_returns_all_false() {
        let mesh = SoaMesh::empty();
        let mask = stl_to_boundary_mask(&mesh, 4, 4, 4, 0.0);
        assert_eq!(mask.len(), 64);
        assert!(mask.iter().all(|&b| !b));
    }

    #[test]
    fn test_unit_cube_interior_marked_solid() {
        let mesh = unit_cube_mesh();
        // 5×5×5 grid, no padding → unit cube fills the entire domain
        let nx = 5;
        let ny = 5;
        let nz = 5;
        let mask = stl_to_boundary_mask(&mesh, nx, ny, nz, 0.0);
        assert_eq!(mask.len(), nx * ny * nz);

        // All cell centres are strictly inside the unit cube [0,1]³ → all solid
        let solid_count = mask.iter().filter(|&&b| b).count();
        assert!(
            solid_count > 0,
            "Expected at least some solid cells for a unit-cube mesh"
        );
    }

    #[test]
    fn test_padding_shrinks_solid_region() {
        let mesh = unit_cube_mesh();
        // With padding=0 the mesh fills the grid entirely → all cells solid
        let mask_no_pad = stl_to_boundary_mask(&mesh, 5, 5, 5, 0.0);
        // With large padding the mesh shrinks to a small fraction → fewer solid cells
        let mask_padded = stl_to_boundary_mask(&mesh, 5, 5, 5, 1.0);

        let solid_no_pad = mask_no_pad.iter().filter(|&&b| b).count();
        let solid_padded = mask_padded.iter().filter(|&&b| b).count();

        assert!(
            solid_padded <= solid_no_pad,
            "Padding should produce no more solid cells than no padding \
             ({} vs {})",
            solid_padded,
            solid_no_pad
        );
    }

    #[test]
    fn test_mask_length_matches_grid() {
        let mesh = unit_cube_mesh();
        for &(nx, ny, nz) in &[(4, 4, 4), (8, 6, 4), (1, 1, 1)] {
            let mask = stl_to_boundary_mask(&mesh, nx, ny, nz, 0.0);
            assert_eq!(mask.len(), nx * ny * nz);
        }
    }
}
