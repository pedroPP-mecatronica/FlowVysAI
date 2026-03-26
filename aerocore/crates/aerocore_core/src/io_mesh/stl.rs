//! STL mesh loader — binary and ASCII formats.
//!
//! # Binary STL
//! Uses `nom` combinators for zero-copy parsing of the `&[u8]` body and
//! `memmap2` for memory-mapped file I/O on large meshes.
//!
//! # Binary format layout
//! ```text
//! [0..80)  — 80-byte header (ignored)
//! [80..84) — u32 LE: number of triangles
//! per triangle (50 bytes):
//!   [0..12)  — 3 × f32 LE: face normal (nx, ny, nz)
//!   [12..24) — 3 × f32 LE: vertex 0 (x, y, z)
//!   [24..36) — 3 × f32 LE: vertex 1 (x, y, z)
//!   [36..48) — 3 × f32 LE: vertex 2 (x, y, z)
//!   [48..50) — u16 LE: attribute byte count (ignored)
//! ```
//!
//! # References
//! - ADR 0002: `docs/adr/0002-stl-binary-parser-mmap-nom.md`
//! - nom docs: <https://docs.rs/nom>
//! - memmap2 docs: <https://docs.rs/memmap2>

use memmap2::Mmap;
use nom::{
    multi::count,
    number::complete::{le_f32, le_u16, le_u32},
    sequence::tuple,
    IResult,
};
use std::path::Path;

use super::mesh::{MeshError, MeshFormat, MeshInfo, SoaMesh};

// ── internal data types ───────────────────────────────────────────────────────

/// A single triangle parsed from a binary STL file.
#[derive(Debug, Clone, Copy)]
struct Triangle {
    normal: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
}

// ── nom parsers ───────────────────────────────────────────────────────────────

/// Parses one 3-component f32 vector from a little-endian byte stream.
#[inline]
fn parse_vec3(input: &[u8]) -> IResult<&[u8], [f32; 3]> {
    let (rest, (x, y, z)) = tuple((le_f32, le_f32, le_f32))(input)?;
    Ok((rest, [x, y, z]))
}

/// Parses one STL triangle record (50 bytes).
fn parse_triangle(input: &[u8]) -> IResult<&[u8], Triangle> {
    let (rest, normal) = parse_vec3(input)?;
    let (rest, v0) = parse_vec3(rest)?;
    let (rest, v1) = parse_vec3(rest)?;
    let (rest, v2) = parse_vec3(rest)?;
    let (rest, _attr) = le_u16(rest)?;
    Ok((rest, Triangle { normal, v0, v1, v2 }))
}

/// Parses the triangle body of a binary STL file (`tri_count` triangles).
///
/// `body` must point to the start of the first triangle record (i.e. after the
/// 80-byte header and 4-byte count).
fn parse_stl_binary_body(body: &[u8], tri_count: usize) -> Result<Vec<Triangle>, MeshError> {
    let (_, triangles) =
        count(parse_triangle, tri_count)(body).map_err(|e| MeshError::ParseError {
            line: 0,
            message: format!("failed to parse binary STL triangle data: {e}"),
        })?;
    Ok(triangles)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Parses a complete binary STL blob (including header and triangle count).
///
/// This is a zero-copy parse: `data` is not modified; the returned [`SoaMesh`]
/// owns its vertex data as `Vec<f64>` (upcast from `f32`).
///
/// # Errors
/// Returns [`MeshError::ParseError`] if the data is malformed or truncated.
pub fn parse_stl_binary(data: &[u8]) -> Result<SoaMesh, MeshError> {
    const HEADER_SIZE: usize = 80;
    const COUNT_SIZE: usize = 4;

    if data.len() < HEADER_SIZE + COUNT_SIZE {
        return Err(MeshError::ParseError {
            line: 0,
            message: format!(
                "binary STL too short: {} bytes (minimum {})",
                data.len(),
                HEADER_SIZE + COUNT_SIZE
            ),
        });
    }

    // Skip 80-byte header; read triangle count.
    let (_, tri_count) =
        le_u32::<&[u8], nom::error::Error<&[u8]>>(&data[HEADER_SIZE..]).map_err(|e| {
            MeshError::ParseError {
                line: 0,
                message: format!("cannot read triangle count: {e}"),
            }
        })?;

    let body = &data[HEADER_SIZE + COUNT_SIZE..];
    let triangles = parse_stl_binary_body(body, tri_count as usize)?;

    triangles_to_mesh(triangles, MeshFormat::StlBinary)
}

/// Loads a binary STL file from `path` using memory-mapped I/O.
///
/// Suitable for large meshes (≫ 1 GB): the file is mapped into virtual address
/// space instead of read into a heap buffer.
///
/// # Errors
/// - [`MeshError::FileNotFound`] if the path does not exist.
/// - [`MeshError::IoError`] on OS-level I/O failure.
/// - [`MeshError::ParseError`] if the data is malformed.
pub fn load_stl(path: &Path) -> Result<SoaMesh, MeshError> {
    let file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            MeshError::FileNotFound(path.display().to_string())
        } else {
            MeshError::IoError(e)
        }
    })?;

    // SAFETY: the file is not modified while the map is live.  This is the
    // standard memmap2 usage pattern for read-only parsing.
    let mmap = unsafe { Mmap::map(&file) }.map_err(MeshError::IoError)?;
    parse_stl_binary(&mmap[..])
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Converts a flat list of [`Triangle`]s into an [`SoaMesh`].
fn triangles_to_mesh(triangles: Vec<Triangle>, format: MeshFormat) -> Result<SoaMesh, MeshError> {
    let n_tris = triangles.len();
    let n_verts = n_tris * 3;

    let mut vx = Vec::with_capacity(n_verts);
    let mut vy = Vec::with_capacity(n_verts);
    let mut vz = Vec::with_capacity(n_verts);
    let mut nx = Vec::with_capacity(n_tris);
    let mut ny = Vec::with_capacity(n_tris);
    let mut nz = Vec::with_capacity(n_tris);
    let mut face_indices = Vec::with_capacity(n_verts);

    let mut bb_min = [f64::MAX; 3];
    let mut bb_max = [f64::MIN; 3];

    for (i, tri) in triangles.iter().enumerate() {
        let base = (i * 3) as u32;
        face_indices.extend_from_slice(&[base, base + 1, base + 2]);

        nx.push(tri.normal[0] as f64);
        ny.push(tri.normal[1] as f64);
        nz.push(tri.normal[2] as f64);

        for v in [tri.v0, tri.v1, tri.v2] {
            let (x, y, z) = (v[0] as f64, v[1] as f64, v[2] as f64);
            vx.push(x);
            vy.push(y);
            vz.push(z);
            bb_min[0] = bb_min[0].min(x);
            bb_min[1] = bb_min[1].min(y);
            bb_min[2] = bb_min[2].min(z);
            bb_max[0] = bb_max[0].max(x);
            bb_max[1] = bb_max[1].max(y);
            bb_max[2] = bb_max[2].max(z);
        }
    }

    if n_tris == 0 {
        bb_min = [0.0; 3];
        bb_max = [0.0; 3];
    }

    Ok(SoaMesh {
        vertices_x: vx,
        vertices_y: vy,
        vertices_z: vz,
        face_indices,
        normals_x: nx,
        normals_y: ny,
        normals_z: nz,
        info: MeshInfo {
            format,
            num_vertices: n_verts,
            num_faces: n_tris,
            num_cells: 0,
            bounding_box_min: bb_min,
            bounding_box_max: bb_max,
        },
    })
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal binary STL blob with `n` triangles.
    fn make_binary_stl(n: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(84 + n * 50);
        buf.extend_from_slice(&[0u8; 80]); // header
        buf.extend_from_slice(&(n as u32).to_le_bytes());
        for i in 0..n {
            let x = i as f32;
            // normal (0, 0, 1)
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&1.0_f32.to_le_bytes());
            // v0
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // v1
            buf.extend_from_slice(&(x + 1.0).to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // v2
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&1.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // attribute byte count
            buf.extend_from_slice(&0u16.to_le_bytes());
        }
        buf
    }

    #[test]
    fn parse_empty_stl() {
        let data = make_binary_stl(0);
        let mesh = parse_stl_binary(&data).expect("parse failed");
        assert_eq!(mesh.num_faces(), 0);
        assert_eq!(mesh.num_vertices(), 0);
    }

    #[test]
    fn parse_single_triangle() {
        let data = make_binary_stl(1);
        let mesh = parse_stl_binary(&data).expect("parse failed");
        assert_eq!(mesh.num_faces(), 1);
        assert_eq!(mesh.num_vertices(), 3);
        // normal z = 1.0
        assert!((mesh.normals_z[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn parse_multiple_triangles_bounding_box() {
        let n = 5;
        let data = make_binary_stl(n);
        let mesh = parse_stl_binary(&data).expect("parse failed");
        assert_eq!(mesh.num_faces(), n);
        // x goes from 0 to n (last triangle x = n-1, x+1 = n)
        assert!(mesh.info.bounding_box_min[0] < mesh.info.bounding_box_max[0]);
    }

    #[test]
    fn load_stl_from_file() {
        use std::io::Write;
        let data = make_binary_stl(3);
        let mut tmp = tempfile::NamedTempFile::new().expect("tmp file");
        tmp.write_all(&data).expect("write");
        tmp.flush().expect("flush");
        let mesh = load_stl(tmp.path()).expect("load_stl failed");
        assert_eq!(mesh.num_faces(), 3);
    }

    #[test]
    fn load_stl_file_not_found() {
        let result = load_stl(Path::new("/nonexistent/path/mesh.stl"));
        assert!(
            matches!(result, Err(MeshError::FileNotFound(_))),
            "Expected FileNotFound, got {result:?}"
        );
    }

    #[test]
    fn parse_too_short_returns_error() {
        let result = parse_stl_binary(&[0u8; 10]);
        assert!(
            matches!(result, Err(MeshError::ParseError { .. })),
            "Expected ParseError for truncated data"
        );
    }
}
