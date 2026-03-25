//! STL mesh parser — binary (nom + memmap2) and ASCII.
//!
//! # Binary format
//! 80-byte header | u32 tri_count | N × 50-byte triangle records
//! Triangle record: normal (3×f32) | v0 (3×f32) | v1 (3×f32) | v2 (3×f32) | attr (u16)

use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use memmap2::Mmap;
use nom::{bytes::complete::take, multi::count, number::complete::le_f32, sequence::tuple, IResult};

use crate::io_mesh::mesh::{MeshError, MeshFormat, MeshInfo, SoaMesh};

const HEADER_BYTES: usize = 80;
const BINARY_HEADER: usize = HEADER_BYTES + 4; // 80-byte header + 4-byte tri count
const BINARY_TRIANGLE_BYTES: usize = 50;

// ── helpers ───────────────────────────────────────────────────────────────────

fn compute_bounding_box(xs: &[f64], ys: &[f64], zs: &[f64]) -> ([f64; 3], [f64; 3]) {
    if xs.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for ((&x, &y), &z) in xs.iter().zip(ys.iter()).zip(zs.iter()) {
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        min[2] = min[2].min(z);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
        max[2] = max[2].max(z);
    }
    (min, max)
}

/// Reads the triangle count from bytes [80..84] of a binary STL.
///
/// # Panics
/// Panics if `data.len() < BINARY_HEADER` — callers must check this first.
fn read_tri_count(data: &[u8]) -> usize {
    u32::from_le_bytes(
        data[HEADER_BYTES..BINARY_HEADER]
            .try_into()
            .expect("slice is exactly 4 bytes; caller guarantees data.len() >= BINARY_HEADER"),
    ) as usize
}

// ── nom parsers ───────────────────────────────────────────────────────────────

type TriangleRecord = ([f32; 3], [[f32; 3]; 3]);

fn parse_vec3(input: &[u8]) -> IResult<&[u8], [f32; 3]> {
    let (input, (x, y, z)) = tuple((le_f32, le_f32, le_f32))(input)?;
    Ok((input, [x, y, z]))
}

fn parse_triangle(input: &[u8]) -> IResult<&[u8], TriangleRecord> {
    let (input, normal) = parse_vec3(input)?;
    let (input, v0) = parse_vec3(input)?;
    let (input, v1) = parse_vec3(input)?;
    let (input, v2) = parse_vec3(input)?;
    let (input, _attr) = take(2usize)(input)?;
    Ok((input, (normal, [v0, v1, v2])))
}

// ── public API ────────────────────────────────────────────────────────────────

/// Parses a binary STL blob.
///
/// Validates the minimum header size and that enough bytes are present for the
/// declared triangle count before invoking the nom combinators.
pub fn parse_stl_binary(data: &[u8]) -> Result<SoaMesh, MeshError> {
    if data.len() < BINARY_HEADER {
        return Err(MeshError::ParseError {
            line: 0,
            message: format!(
                "binary STL too small: need at least {} bytes, got {}",
                BINARY_HEADER,
                data.len()
            ),
        });
    }

    let tri_count = read_tri_count(data);
    let expected = BINARY_HEADER + tri_count * BINARY_TRIANGLE_BYTES;
    if data.len() < expected {
        return Err(MeshError::ParseError {
            line: 0,
            message: format!(
                "binary STL truncated: expected {} bytes for {} triangles, got {}",
                expected,
                tri_count,
                data.len()
            ),
        });
    }

    let num_verts = tri_count * 3;
    let mut vertices_x = Vec::with_capacity(num_verts);
    let mut vertices_y = Vec::with_capacity(num_verts);
    let mut vertices_z = Vec::with_capacity(num_verts);
    let mut normals_x = Vec::with_capacity(tri_count);
    let mut normals_y = Vec::with_capacity(tri_count);
    let mut normals_z = Vec::with_capacity(tri_count);
    let mut face_indices: Vec<u32> = Vec::with_capacity(tri_count * 3);

    let body = &data[BINARY_HEADER..expected];
    let (_, triangles) = count(parse_triangle, tri_count)(body).map_err(|_| {
        MeshError::ParseError {
            line: 0,
            message: "failed to parse binary STL triangle data".to_string(),
        }
    })?;

    for (i, (normal, verts)) in triangles.iter().enumerate() {
        let base = (i * 3) as u32;
        normals_x.push(f64::from(normal[0]));
        normals_y.push(f64::from(normal[1]));
        normals_z.push(f64::from(normal[2]));
        for &v in verts {
            vertices_x.push(f64::from(v[0]));
            vertices_y.push(f64::from(v[1]));
            vertices_z.push(f64::from(v[2]));
        }
        face_indices.push(base);
        face_indices.push(base + 1);
        face_indices.push(base + 2);
    }

    let (bbox_min, bbox_max) = compute_bounding_box(&vertices_x, &vertices_y, &vertices_z);

    Ok(SoaMesh {
        info: MeshInfo {
            format: MeshFormat::StlBinary,
            num_vertices: num_verts,
            num_faces: tri_count,
            num_cells: 0,
            bounding_box_min: bbox_min,
            bounding_box_max: bbox_max,
        },
        vertices_x,
        vertices_y,
        vertices_z,
        face_indices,
        normals_x,
        normals_y,
        normals_z,
    })
}

/// Parses an ASCII STL blob (state-machine over lines).
///
/// No vertex deduplication: each facet contributes 3 unique vertex slots.
pub fn parse_stl_ascii(data: &[u8]) -> Result<SoaMesh, MeshError> {
    let text = std::str::from_utf8(data).map_err(|_| MeshError::ParseError {
        line: 0,
        message: "ASCII STL is not valid UTF-8".to_string(),
    })?;

    let mut vertices_x: Vec<f64> = Vec::new();
    let mut vertices_y: Vec<f64> = Vec::new();
    let mut vertices_z: Vec<f64> = Vec::new();
    let mut normals_x: Vec<f64> = Vec::new();
    let mut normals_y: Vec<f64> = Vec::new();
    let mut normals_z: Vec<f64> = Vec::new();
    let mut face_indices: Vec<u32> = Vec::new();

    let mut current_normal: Option<[f64; 3]> = None;
    let mut verts_in_face = 0usize;
    let mut face_base: u32 = 0;

    for (lineno, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("facet normal") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 5 {
                return Err(MeshError::ParseError {
                    line: lineno + 1,
                    message: "malformed facet normal".to_string(),
                });
            }
            let nx = parts[2].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid normal x: {}", parts[2]),
            })?;
            let ny = parts[3].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid normal y: {}", parts[3]),
            })?;
            let nz = parts[4].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid normal z: {}", parts[4]),
            })?;
            current_normal = Some([nx, ny, nz]);
            verts_in_face = 0;
            face_base = vertices_x.len() as u32;
        } else if trimmed.split_whitespace().next() == Some("vertex") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 4 {
                return Err(MeshError::ParseError {
                    line: lineno + 1,
                    message: "malformed vertex line".to_string(),
                });
            }
            let vx = parts[1].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid vertex x: {}", parts[1]),
            })?;
            let vy = parts[2].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid vertex y: {}", parts[2]),
            })?;
            let vz = parts[3].parse::<f64>().map_err(|_| MeshError::ParseError {
                line: lineno + 1,
                message: format!("invalid vertex z: {}", parts[3]),
            })?;
            vertices_x.push(vx);
            vertices_y.push(vy);
            vertices_z.push(vz);
            verts_in_face += 1;
        } else if trimmed == "endfacet" {
            if verts_in_face != 3 {
                return Err(MeshError::ParseError {
                    line: lineno + 1,
                    message: format!("facet has {} vertices, expected 3", verts_in_face),
                });
            }
            if let Some(n) = current_normal.take() {
                normals_x.push(n[0]);
                normals_y.push(n[1]);
                normals_z.push(n[2]);
            }
            face_indices.push(face_base);
            face_indices.push(face_base + 1);
            face_indices.push(face_base + 2);
        }
    }

    let num_faces = normals_x.len();
    let num_verts = vertices_x.len();
    let (bbox_min, bbox_max) = compute_bounding_box(&vertices_x, &vertices_y, &vertices_z);

    Ok(SoaMesh {
        info: MeshInfo {
            format: MeshFormat::StlAscii,
            num_vertices: num_verts,
            num_faces,
            num_cells: 0,
            bounding_box_min: bbox_min,
            bounding_box_max: bbox_max,
        },
        vertices_x,
        vertices_y,
        vertices_z,
        face_indices,
        normals_x,
        normals_y,
        normals_z,
    })
}

/// Auto-detects binary vs ASCII and delegates to the appropriate parser.
///
/// Detection heuristic: if the data is at least 84 bytes and the declared
/// triangle count fits the file size exactly, treat as binary.  Otherwise, if
/// the file does not start with the ASCII `solid` keyword, also treat as binary.
pub fn parse_stl(data: &[u8]) -> Result<SoaMesh, MeshError> {
    if is_binary(data) {
        parse_stl_binary(data)
    } else {
        parse_stl_ascii(data)
    }
}

fn is_binary(data: &[u8]) -> bool {
    if data.len() < BINARY_HEADER {
        return false;
    }
    let tri_count = read_tri_count(data);
    // Exact size match is a reliable binary indicator.
    if data.len() == BINARY_HEADER + tri_count * BINARY_TRIANGLE_BYTES {
        return true;
    }
    // Files not starting with the ASCII "solid" keyword are binary.
    !data.starts_with(b"solid")
}

/// Loads an STL file from disk using `memmap2` and delegates to `parse_stl`.
///
/// `NotFound` I/O errors are mapped to `MeshError::FileNotFound`.
pub fn load_stl(path: &Path) -> Result<SoaMesh, MeshError> {
    let file = File::open(path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            MeshError::FileNotFound(path.display().to_string())
        } else {
            MeshError::IoError(e)
        }
    })?;
    // SAFETY: The file is not modified while the mapping is live within this function.
    let mmap = unsafe { Mmap::map(&file) }.map_err(MeshError::IoError)?;
    parse_stl(&mmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── test helpers ──────────────────────────────────────────────────────────

    /// Builds an in-memory binary STL blob from a slice of (normal, [v0,v1,v2]) records.
    fn make_binary_stl(triangles: &[([f32; 3], [[f32; 3]; 3])]) -> Vec<u8> {
        let n = triangles.len();
        let mut buf = Vec::with_capacity(BINARY_HEADER + n * BINARY_TRIANGLE_BYTES);
        buf.extend_from_slice(&[0u8; HEADER_BYTES]); // header
        buf.extend_from_slice(&(n as u32).to_le_bytes());
        for (normal, verts) in triangles {
            for &c in normal {
                buf.extend_from_slice(&c.to_le_bytes());
            }
            for v in verts {
                for &c in v {
                    buf.extend_from_slice(&c.to_le_bytes());
                }
            }
            buf.extend_from_slice(&0u16.to_le_bytes()); // attribute
        }
        buf
    }

    fn simple_triangle() -> ([f32; 3], [[f32; 3]; 3]) {
        ([0.0f32, 0.0, 1.0], [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }

    // ── binary parser tests ───────────────────────────────────────────────────

    #[test]
    fn test_binary_single_triangle() {
        let blob = make_binary_stl(&[simple_triangle()]);
        let mesh = parse_stl_binary(&blob).expect("parse failed");
        assert_eq!(mesh.info.num_faces, 1);
        assert_eq!(mesh.info.num_vertices, 3);
        assert_eq!(mesh.face_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_binary_multi_triangle_vertex_count() {
        const N: usize = 5;
        let tris: Vec<_> = (0..N).map(|_| simple_triangle()).collect();
        let blob = make_binary_stl(&tris);
        let mesh = parse_stl_binary(&blob).expect("parse failed");
        assert_eq!(mesh.info.num_faces, N);
        assert_eq!(mesh.info.num_vertices, N * 3);
        assert_eq!(mesh.vertices_x.len(), N * 3);
        assert_eq!(mesh.face_indices.len(), N * 3);
    }

    #[test]
    fn test_binary_bounding_box_multi() {
        let tri1 = ([0.0f32, 0.0, 1.0], [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let tri2 =
            ([0.0f32, 0.0, 1.0], [[2.0f32, 3.0, 4.0], [3.0, 3.0, 4.0], [2.0, 4.0, 4.0]]);
        let blob = make_binary_stl(&[tri1, tri2]);
        let mesh = parse_stl_binary(&blob).expect("parse failed");
        let [minx, miny, minz] = mesh.info.bounding_box_min;
        let [maxx, maxy, maxz] = mesh.info.bounding_box_max;
        assert!((minx - 0.0).abs() < 1e-6, "min x");
        assert!((miny - 0.0).abs() < 1e-6, "min y");
        assert!((minz - 0.0).abs() < 1e-6, "min z");
        assert!((maxx - 3.0).abs() < 1e-6, "max x");
        assert!((maxy - 4.0).abs() < 1e-6, "max y");
        assert!((maxz - 4.0).abs() < 1e-6, "max z");
    }

    #[test]
    fn test_binary_too_small() {
        let err = parse_stl_binary(&[0u8; 10]).unwrap_err();
        assert!(matches!(err, MeshError::ParseError { .. }));
    }

    #[test]
    fn test_binary_truncated() {
        let mut blob = make_binary_stl(&[simple_triangle()]);
        blob.truncate(blob.len() - 1);
        let err = parse_stl_binary(&blob).unwrap_err();
        assert!(matches!(err, MeshError::ParseError { .. }));
    }

    // ── ASCII parser tests ────────────────────────────────────────────────────

    #[test]
    fn test_ascii_single_facet() {
        let stl = b"solid test\n\
            facet normal 0 0 1\n\
              outer loop\n\
                vertex 0 0 0\n\
                vertex 1 0 0\n\
                vertex 0 1 0\n\
              endloop\n\
            endfacet\n\
            endsolid test\n";
        let mesh = parse_stl_ascii(stl).expect("ascii parse failed");
        assert_eq!(mesh.info.num_faces, 1);
        assert_eq!(mesh.info.num_vertices, 3);
    }

    // ── mmap / load_stl tests ─────────────────────────────────────────────────

    #[test]
    fn test_load_stl_mmap_matches_parse_binary() {
        use std::io::Write;
        let tris: Vec<_> = (0..10).map(|_| simple_triangle()).collect();
        let blob = make_binary_stl(&tris);

        let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
        tmp.write_all(&blob).expect("write");
        tmp.flush().expect("flush");

        let mesh_mmap = load_stl(tmp.path()).expect("load_stl failed");
        let mesh_mem = parse_stl_binary(&blob).expect("parse_stl_binary failed");

        assert_eq!(mesh_mmap.info.num_faces, mesh_mem.info.num_faces);
        assert_eq!(mesh_mmap.info.num_vertices, mesh_mem.info.num_vertices);
        assert_eq!(mesh_mmap.info.bounding_box_min, mesh_mem.info.bounding_box_min);
        assert_eq!(mesh_mmap.info.bounding_box_max, mesh_mem.info.bounding_box_max);
    }

    #[test]
    fn test_load_stl_not_found() {
        let err = load_stl(Path::new("/nonexistent/path/no_such.stl")).unwrap_err();
        assert!(matches!(err, MeshError::FileNotFound(_)));
    }
}