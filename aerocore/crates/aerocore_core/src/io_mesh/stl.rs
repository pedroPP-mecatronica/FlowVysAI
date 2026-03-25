//! STL mesh parser — supports both ASCII and binary formats.
//!
//! # Binary Format Layout
//! ```text
//! [80 bytes]  header (ignored)
//! [4  bytes]  uint32 — number of triangles
//! For each triangle (50 bytes):
//!   [12 bytes] float32 × 3 — face normal (nx, ny, nz)
//!   [12 bytes] float32 × 3 — vertex 0   (x, y, z)
//!   [12 bytes] float32 × 3 — vertex 1   (x, y, z)
//!   [12 bytes] float32 × 3 — vertex 2   (x, y, z)
//!   [2  bytes] uint16       — attribute byte count (ignored)
//! ```
//!
//! # ASCII Format
//! ```text
//! solid <name>
//!   facet normal nx ny nz
//!     outer loop
//!       vertex x y z
//!       vertex x y z
//!       vertex x y z
//!     endloop
//!   endfacet
//!   ...
//! endsolid <name>
//! ```
//!
//! # Binary parsing strategy
//! Binary STL files are parsed from a `&[u8]` using [`nom`] combinators
//! (zero-copy, streaming-friendly).  When loading from disk, [`memmap2`] is
//! used so the OS page cache backs the slice — **no whole-file `Vec<u8>`
//! allocation** for the binary path.
//!
//! # Note on Vertex Deduplication
//! For simplicity and performance, vertices are stored without deduplication.
//! Each triangle contributes 3 unique vertex entries.  Deduplication can be
//! applied as a post-processing step when index buffer size matters.

use std::io;

use memmap2::Mmap;
use nom::{
    bytes::complete::take,
    combinator::map,
    multi::count,
    number::complete::le_f32,
    sequence::tuple,
    IResult,
};

use super::mesh::{MeshError, MeshFormat, MeshInfo, SoaMesh};

// ── binary constants ──────────────────────────────────────────────────────────
const HEADER_BYTES: usize = 80;
const BINARY_TRIANGLE_BYTES: usize = 50; // 12 normal + 36 verts + 2 attr

// ── nom parsers ───────────────────────────────────────────────────────────────

/// `(normal, [v0, v1, v2])` returned by [`parse_triangle`].
type TriangleRecord = ([f32; 3], [[f32; 3]; 3]);

/// Parses three consecutive little-endian `f32` values from the input.
#[inline]
fn parse_vec3(input: &[u8]) -> IResult<&[u8], [f32; 3]> {
    map(tuple((le_f32, le_f32, le_f32)), |(x, y, z)| [x, y, z])(input)
}

/// Parses one binary STL triangle record (50 bytes) and returns
/// `(normal [f32;3], [v0, v1, v2] [[f32;3];3])`.
#[inline]
fn parse_triangle(input: &[u8]) -> IResult<&[u8], TriangleRecord> {
    let (input, normal) = parse_vec3(input)?;
    let (input, v0) = parse_vec3(input)?;
    let (input, v1) = parse_vec3(input)?;
    let (input, v2) = parse_vec3(input)?;
    // attribute byte count (2 bytes) — ignored
    let (input, _) = take(2usize)(input)?;
    Ok((input, (normal, [v0, v1, v2])))
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Reads a little-endian `u32` from a 4-byte slice.
#[inline(always)]
fn read_u32_le(buf: &[u8]) -> u32 {
    u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])
}

// ── format detection ──────────────────────────────────────────────────────────

/// Returns `true` if the first non-whitespace bytes look like the ASCII `solid`
/// keyword.  The heuristic also rejects files whose header happens to start
/// with "solid " but whose body is binary (common encoder quirk).
fn is_ascii_stl(data: &[u8]) -> bool {
    // Skip leading whitespace
    let trimmed = data.iter().position(|&b| !b.is_ascii_whitespace());
    let start = match trimmed {
        Some(i) => i,
        None => return false,
    };

    // Must start with "solid"
    if data.len() < start + 5 {
        return false;
    }
    let prefix = &data[start..start + 5];
    if !prefix.eq_ignore_ascii_case(b"solid") {
        return false;
    }

    // Distinguish binary files whose 80-byte header starts with "solid":
    // in a valid binary STL the triangle count follows at bytes [80..84].
    // If the file is large enough, verify the byte count is consistent.
    if data.len() >= HEADER_BYTES + 4 {
        let count = read_u32_le(&data[HEADER_BYTES..HEADER_BYTES + 4]) as usize;
        let expected_size = HEADER_BYTES + 4 + count * BINARY_TRIANGLE_BYTES;
        if data.len() == expected_size {
            return false; // it's binary
        }
    }

    true
}

// ── bounding-box helper ───────────────────────────────────────────────────────

fn compute_bounding_box(
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
) -> ([f64; 3], [f64; 3]) {
    if xs.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for (&x, (&y, &z)) in xs.iter().zip(ys.iter().zip(zs.iter())) {
        if x < min[0] { min[0] = x; }
        if y < min[1] { min[1] = y; }
        if z < min[2] { min[2] = z; }
        if x > max[0] { max[0] = x; }
        if y > max[1] { max[1] = y; }
        if z > max[2] { max[2] = z; }
    }
    (min, max)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Parses STL data (binary **or** ASCII) from a byte slice and returns a
/// [`SoaMesh`].
///
/// # Errors
/// Returns [`MeshError::ParseError`] if the data is malformed, or
/// [`MeshError::UnsupportedFormat`] if the slice is too short to be a valid
/// STL file.
pub fn parse_stl(data: &[u8]) -> Result<SoaMesh, MeshError> {
    if is_ascii_stl(data) {
        parse_stl_ascii(data)
    } else {
        parse_stl_binary(data)
    }
}

/// Loads an STL file from disk and parses it.
///
/// For binary STL files the file is **memory-mapped** via [`memmap2`], so the
/// OS page cache backs the parse slice — no whole-file allocation.  ASCII STL
/// files are read into a `Vec<u8>` as before.
pub fn load_stl(path: &std::path::Path) -> Result<SoaMesh, MeshError> {
    // Open the file first so we can mmap it.
    let file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            MeshError::FileNotFound(path.display().to_string())
        } else {
            MeshError::IoError(e)
        }
    })?;

    // Safety: the file is not modified while we hold the mapping.
    // This is the standard memmap2 usage pattern for read-only parsing.
    let mmap = unsafe { Mmap::map(&file) }.map_err(MeshError::IoError)?;

    parse_stl(&mmap)
}

// ── binary parser ─────────────────────────────────────────────────────────────

/// Parses a binary STL from a raw byte slice using [`nom`] combinators.
///
/// The slice may be backed by a memory-mapped file (`memmap2::Mmap`) for
/// zero-copy parsing of large meshes.
///
/// # Errors
/// - [`MeshError::ParseError`] — header/count field missing, or file truncated.
pub fn parse_stl_binary(data: &[u8]) -> Result<SoaMesh, MeshError> {
    // Minimum valid binary STL: 80-byte header + 4-byte count
    if data.len() < HEADER_BYTES + 4 {
        return Err(MeshError::ParseError {
            line: 0,
            message: format!(
                "Binary STL too short: {} bytes (minimum {})",
                data.len(),
                HEADER_BYTES + 4
            ),
        });
    }

    let num_triangles = read_u32_le(&data[HEADER_BYTES..HEADER_BYTES + 4]) as usize;
    let expected = HEADER_BYTES + 4 + num_triangles * BINARY_TRIANGLE_BYTES;

    if data.len() < expected {
        return Err(MeshError::ParseError {
            line: 0,
            message: format!(
                "Binary STL truncated: expected {} bytes for {} triangles, got {}",
                expected, num_triangles, data.len()
            ),
        });
    }

    // Pre-allocate SoA buffers — single allocation per array.
    let mut vertices_x = Vec::with_capacity(num_triangles * 3);
    let mut vertices_y = Vec::with_capacity(num_triangles * 3);
    let mut vertices_z = Vec::with_capacity(num_triangles * 3);
    let mut normals_x  = Vec::with_capacity(num_triangles);
    let mut normals_y  = Vec::with_capacity(num_triangles);
    let mut normals_z  = Vec::with_capacity(num_triangles);
    let mut face_indices: Vec<u32> = Vec::with_capacity(num_triangles * 3);

    // nom parse — operates on the triangle payload slice only.
    let payload = &data[HEADER_BYTES + 4..];
    let (_, triangles) = count(parse_triangle, num_triangles)(payload)
        .map_err(|e| MeshError::ParseError {
            line: 0,
            message: format!("nom parse error in binary STL triangles: {e}"),
        })?;

    for (tri_idx, (normal, verts)) in triangles.into_iter().enumerate() {
        let base_idx = (tri_idx * 3) as u32;

        normals_x.push(normal[0] as f64);
        normals_y.push(normal[1] as f64);
        normals_z.push(normal[2] as f64);

        for v in &verts {
            vertices_x.push(v[0] as f64);
            vertices_y.push(v[1] as f64);
            vertices_z.push(v[2] as f64);
        }

        face_indices.push(base_idx);
        face_indices.push(base_idx + 1);
        face_indices.push(base_idx + 2);
    }

    let (bb_min, bb_max) = compute_bounding_box(&vertices_x, &vertices_y, &vertices_z);
    let num_vertices = vertices_x.len();

    Ok(SoaMesh {
        vertices_x,
        vertices_y,
        vertices_z,
        face_indices,
        normals_x,
        normals_y,
        normals_z,
        info: MeshInfo {
            format: MeshFormat::StlBinary,
            num_vertices,
            num_faces: num_triangles,
            num_cells: 0,
            bounding_box_min: bb_min,
            bounding_box_max: bb_max,
        },
    })
}

// ── ASCII parser ──────────────────────────────────────────────────────────────

/// Parses an ASCII STL from a raw byte slice.
pub fn parse_stl_ascii(data: &[u8]) -> Result<SoaMesh, MeshError> {
    let text = std::str::from_utf8(data).map_err(|_| MeshError::ParseError {
        line: 0,
        message: "STL ASCII file contains invalid UTF-8".into(),
    })?;

    let mut vertices_x = Vec::new();
    let mut vertices_y = Vec::new();
    let mut vertices_z = Vec::new();
    let mut normals_x  = Vec::new();
    let mut normals_y  = Vec::new();
    let mut normals_z  = Vec::new();
    let mut face_indices: Vec<u32> = Vec::new();

    // State machine
    #[derive(PartialEq)]
    enum State {
        Root,
        InFacet,
        InLoop,
    }
    let mut state = State::Root;
    let mut cur_normal = [0.0_f64; 3];
    let mut verts_in_face: usize = 0;

    for (line_no, line) in text.lines().enumerate() {
        let line_num = line_no + 1;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0].to_ascii_lowercase().as_str() {
            "solid" | "endsolid" => {
                state = State::Root;
            }
            "facet" => {
                // "facet normal nx ny nz"
                if tokens.len() < 5 || !tokens[1].eq_ignore_ascii_case("normal") {
                    return Err(MeshError::ParseError {
                        line: line_num,
                        message: "Expected 'facet normal nx ny nz'".into(),
                    });
                }
                cur_normal[0] = parse_f64(tokens[2], line_num)?;
                cur_normal[1] = parse_f64(tokens[3], line_num)?;
                cur_normal[2] = parse_f64(tokens[4], line_num)?;
                state = State::InFacet;
                verts_in_face = 0;
            }
            "outer" => {
                if state != State::InFacet {
                    return Err(MeshError::ParseError {
                        line: line_num,
                        message: "'outer loop' outside 'facet'".into(),
                    });
                }
                state = State::InLoop;
            }
            "vertex" => {
                if state != State::InLoop {
                    return Err(MeshError::ParseError {
                        line: line_num,
                        message: "'vertex' keyword outside 'outer loop'".into(),
                    });
                }
                if tokens.len() < 4 {
                    return Err(MeshError::ParseError {
                        line: line_num,
                        message: "Expected 'vertex x y z'".into(),
                    });
                }
                vertices_x.push(parse_f64(tokens[1], line_num)?);
                vertices_y.push(parse_f64(tokens[2], line_num)?);
                vertices_z.push(parse_f64(tokens[3], line_num)?);
                verts_in_face += 1;
            }
            "endloop" => {
                if verts_in_face != 3 {
                    return Err(MeshError::ParseError {
                        line: line_num,
                        message: format!(
                            "Expected exactly 3 vertices per face, got {}",
                            verts_in_face
                        ),
                    });
                }
                state = State::InFacet;
            }
            "endfacet" => {
                let base = (vertices_x.len() as u32) - 3;
                face_indices.push(base);
                face_indices.push(base + 1);
                face_indices.push(base + 2);
                normals_x.push(cur_normal[0]);
                normals_y.push(cur_normal[1]);
                normals_z.push(cur_normal[2]);
                state = State::Root;
            }
            _ => {} // comments or unknown lines — ignored
        }
    }

    let num_faces = normals_x.len();
    let num_vertices = vertices_x.len();
    let (bb_min, bb_max) = compute_bounding_box(&vertices_x, &vertices_y, &vertices_z);

    Ok(SoaMesh {
        vertices_x,
        vertices_y,
        vertices_z,
        face_indices,
        normals_x,
        normals_y,
        normals_z,
        info: MeshInfo {
            format: MeshFormat::StlAscii,
            num_vertices,
            num_faces,
            num_cells: 0,
            bounding_box_min: bb_min,
            bounding_box_max: bb_max,
        },
    })
}

// ── private helpers ───────────────────────────────────────────────────────────

fn parse_f64(s: &str, line: usize) -> Result<f64, MeshError> {
    s.parse::<f64>().map_err(|_| MeshError::ParseError {
        line,
        message: format!("Cannot parse '{}' as f64", s),
    })
}

// ── SoaMesh as MeshProvider ───────────────────────────────────────────────────

impl super::mesh::MeshProvider for SoaMesh {
    fn load(&mut self, path: &std::path::Path) -> Result<MeshInfo, MeshError> {
        let loaded = load_stl(path)?;
        *self = loaded;
        Ok(self.info.clone())
    }

    fn vertices_soa(&self) -> (&[f64], &[f64], &[f64]) {
        (&self.vertices_x, &self.vertices_y, &self.vertices_z)
    }

    fn face_indices(&self) -> &[u32] {
        &self.face_indices
    }

    fn face_normals_soa(&self) -> (&[f64], &[f64], &[f64]) {
        (&self.normals_x, &self.normals_y, &self.normals_z)
    }

    fn info(&self) -> &MeshInfo {
        &self.info
    }

    fn unload(&mut self) {
        *self = SoaMesh::empty();
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Builds a minimal binary STL with a single triangle.
    fn single_triangle_binary() -> Vec<u8> {
        let mut buf = vec![0u8; 80]; // header
        // num_triangles = 1
        buf.extend_from_slice(&1u32.to_le_bytes());

        // normal (0, 0, 1)
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());

        // vertex 0 (0, 0, 0)
        for _ in 0..3 {
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
        }
        // vertex 1 (1, 0, 0)
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        // vertex 2 (0, 1, 0)
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());

        // attribute
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf
    }

    const SINGLE_TRIANGLE_ASCII: &str = "\
solid test
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid test
";

    // ── binary tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_binary_parse_single_triangle() {
        let data = single_triangle_binary();
        let mesh = parse_stl_binary(&data).expect("parse_stl_binary failed");

        assert_eq!(mesh.num_faces(), 1, "expected 1 face");
        assert_eq!(mesh.num_vertices(), 3, "expected 3 vertices");

        // Check face index triple
        assert_eq!(&mesh.face_indices, &[0, 1, 2]);

        // Check normal
        assert!((mesh.normals_z[0] - 1.0).abs() < 1e-6);

        // Check vertex 1 x-coord
        assert!((mesh.vertices_x[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_binary_bounding_box() {
        let data = single_triangle_binary();
        let mesh = parse_stl_binary(&data).unwrap();
        let info = &mesh.info;

        assert!((info.bounding_box_min[0] - 0.0).abs() < 1e-9);
        assert!((info.bounding_box_min[1] - 0.0).abs() < 1e-9);
        assert!((info.bounding_box_max[0] - 1.0).abs() < 1e-6);
        assert!((info.bounding_box_max[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_binary_too_short_returns_error() {
        let result = parse_stl_binary(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_truncated_triangles_returns_error() {
        let mut data = single_triangle_binary();
        data.truncate(data.len() - 10); // truncate last triangle
        let result = parse_stl_binary(&data);
        assert!(result.is_err());
    }

    // ── ASCII tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_ascii_parse_single_triangle() {
        let data = SINGLE_TRIANGLE_ASCII.as_bytes();
        let mesh = parse_stl_ascii(data).expect("parse_stl_ascii failed");

        assert_eq!(mesh.num_faces(), 1);
        assert_eq!(mesh.num_vertices(), 3);
        assert_eq!(&mesh.face_indices, &[0, 1, 2]);

        assert!((mesh.normals_z[0] - 1.0).abs() < 1e-9);
        assert!((mesh.vertices_x[1] - 1.0).abs() < 1e-9);
        assert!((mesh.vertices_y[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_ascii_bounding_box() {
        let mesh = parse_stl_ascii(SINGLE_TRIANGLE_ASCII.as_bytes()).unwrap();
        let info = &mesh.info;
        assert_eq!(info.format, MeshFormat::StlAscii);
        assert!((info.bounding_box_max[0] - 1.0).abs() < 1e-9);
        assert!((info.bounding_box_max[1] - 1.0).abs() < 1e-9);
        assert!((info.bounding_box_max[2] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_ascii_multiple_triangles() {
        let ascii = "\
solid cube_face
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 1 0 0
      vertex 1 1 0
      vertex 0 1 0
    endloop
  endfacet
endsolid cube_face
";
        let mesh = parse_stl_ascii(ascii.as_bytes()).unwrap();
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.num_vertices(), 6);
        // Second face indices
        assert_eq!(&mesh.face_indices[3..], &[3, 4, 5]);
    }

    // ── format detection ──────────────────────────────────────────────────────

    #[test]
    fn test_auto_detect_ascii() {
        let mesh = parse_stl(SINGLE_TRIANGLE_ASCII.as_bytes()).unwrap();
        assert_eq!(mesh.info.format, MeshFormat::StlAscii);
    }

    #[test]
    fn test_auto_detect_binary() {
        let data = single_triangle_binary();
        let mesh = parse_stl(&data).unwrap();
        assert_eq!(mesh.info.format, MeshFormat::StlBinary);
    }

    // ── nom binary parser tests ───────────────────────────────────────────────

    /// Builds a binary STL blob with `n` identical triangles for testing.
    fn multi_triangle_binary(n: usize) -> Vec<u8> {
        let mut buf = vec![0u8; 80]; // header
        buf.extend_from_slice(&(n as u32).to_le_bytes());
        for i in 0..n {
            let fi = i as f32;
            // normal (0, 0, 1)
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&1.0_f32.to_le_bytes());
            // v0
            buf.extend_from_slice(&fi.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // v1
            buf.extend_from_slice(&(fi + 1.0_f32).to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // v2
            buf.extend_from_slice(&fi.to_le_bytes());
            buf.extend_from_slice(&1.0_f32.to_le_bytes());
            buf.extend_from_slice(&0.0_f32.to_le_bytes());
            // attribute
            buf.extend_from_slice(&0u16.to_le_bytes());
        }
        buf
    }

    #[test]
    fn test_nom_binary_multi_triangle_vertex_count() {
        let n = 7;
        let data = multi_triangle_binary(n);
        let mesh = parse_stl_binary(&data).expect("parse_stl_binary failed");
        assert_eq!(mesh.num_faces(), n, "expected {n} faces");
        assert_eq!(mesh.num_vertices(), n * 3, "expected {} vertices", n * 3);
        assert_eq!(mesh.face_indices.len(), n * 3);
    }

    #[test]
    fn test_nom_binary_bounding_box_multi() {
        // 5 triangles with v0.x = 0..4, v1.x = 1..5 → x range [0, 5)
        let n = 5;
        let data = multi_triangle_binary(n);
        let mesh = parse_stl_binary(&data).unwrap();
        let info = &mesh.info;
        assert!((info.bounding_box_min[0] - 0.0).abs() < 1e-6,
            "x_min should be 0, got {}", info.bounding_box_min[0]);
        assert!((info.bounding_box_max[0] - (n as f64)).abs() < 1e-5,
            "x_max should be {n}, got {}", info.bounding_box_max[0]);
        assert!((info.bounding_box_max[1] - 1.0).abs() < 1e-6,
            "y_max should be 1.0, got {}", info.bounding_box_max[1]);
    }

    // ── mmap-based path tests ─────────────────────────────────────────────────

    /// Tests that `load_stl` (which uses memmap2 internally) produces the same
    /// mesh as parsing the same bytes directly.
    #[test]
    fn test_load_stl_mmap_binary_matches_parse() {
        use std::io::Write;
        let data = multi_triangle_binary(4);

        // Write to a temp file.
        let mut tmp = tempfile::NamedTempFile::new()
            .expect("could not create temp file");
        tmp.write_all(&data).expect("write failed");
        tmp.flush().expect("flush failed");

        let mesh_from_file = load_stl(tmp.path()).expect("load_stl failed");
        let mesh_from_bytes = parse_stl_binary(&data).expect("parse_stl_binary failed");

        assert_eq!(mesh_from_file.num_faces(), mesh_from_bytes.num_faces());
        assert_eq!(mesh_from_file.num_vertices(), mesh_from_bytes.num_vertices());
        assert_eq!(mesh_from_file.face_indices, mesh_from_bytes.face_indices);
        for i in 0..mesh_from_file.num_vertices() {
            assert!((mesh_from_file.vertices_x[i] - mesh_from_bytes.vertices_x[i]).abs() < 1e-10);
            assert!((mesh_from_file.vertices_y[i] - mesh_from_bytes.vertices_y[i]).abs() < 1e-10);
            assert!((mesh_from_file.vertices_z[i] - mesh_from_bytes.vertices_z[i]).abs() < 1e-10);
        }
    }

    // ── MeshProvider trait ────────────────────────────────────────────────────

    #[test]
    fn test_mesh_provider_vertices_soa() {
        use super::super::mesh::MeshProvider;
        let mesh = parse_stl_ascii(SINGLE_TRIANGLE_ASCII.as_bytes()).unwrap();
        let (xs, ys, zs) = mesh.vertices_soa();
        assert_eq!(xs.len(), 3);
        assert_eq!(ys.len(), 3);
        assert_eq!(zs.len(), 3);
    }

    #[test]
    fn test_characteristic_length() {
        let data = single_triangle_binary();
        let mesh = parse_stl(&data).unwrap();
        // bbox is [0..1] × [0..1] × [0..0] → max extent = 1.0
        assert!((mesh.info.characteristic_length() - 1.0).abs() < 1e-6);
    }
}
