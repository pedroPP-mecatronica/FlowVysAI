//! Mesh data structures and traits — SoA storage for geometry.
//!
//! # Design
//! - SoA (Structure of Arrays) layout for cache-friendly vertex access
//! - `MeshProvider` trait for format-agnostic loading
//! - Supports STL (ASCII + binary) and CGNS formats
//! - Memory-mapped I/O for large meshes (>1 GB)

/// Supported mesh file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    /// STL ASCII format (human-readable).
    StlAscii,
    /// STL Binary format (compact, fast to parse).
    StlBinary,
    /// CGNS (CFD General Notation System) — industry standard.
    Cgns,
}

/// Summary information about a loaded mesh.
#[derive(Debug, Clone)]
pub struct MeshInfo {
    pub format: MeshFormat,
    pub num_vertices: usize,
    pub num_faces: usize,
    pub num_cells: usize,
    pub bounding_box_min: [f64; 3],
    pub bounding_box_max: [f64; 3],
}

impl MeshInfo {
    /// Extent of the bounding box along each axis.
    pub fn extents(&self) -> [f64; 3] {
        [
            self.bounding_box_max[0] - self.bounding_box_min[0],
            self.bounding_box_max[1] - self.bounding_box_min[1],
            self.bounding_box_max[2] - self.bounding_box_min[2],
        ]
    }

    /// Maximum dimension of the bounding box (characteristic length).
    pub fn characteristic_length(&self) -> f64 {
        let e = self.extents();
        e[0].max(e[1]).max(e[2])
    }
}

/// Mesh error types.
#[derive(Debug)]
pub enum MeshError {
    /// File not found at path.
    FileNotFound(String),
    /// Parse error at a specific location.
    ParseError { line: usize, message: String },
    /// File format is not supported.
    UnsupportedFormat(String),
    /// I/O error during file reading.
    IoError(std::io::Error),
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "Mesh file not found: {}", p),
            Self::ParseError { line, message } =>
                write!(f, "Parse error at line {}: {}", line, message),
            Self::UnsupportedFormat(fmt) => write!(f, "Unsupported mesh format: {}", fmt),
            Self::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for MeshError {}

impl From<std::io::Error> for MeshError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Mesh data in SoA (Structure of Arrays) layout.
///
/// # Data-Oriented Design
/// Vertices stored as separate x[], y[], z[] arrays for:
/// - SIMD-friendly iteration
/// - Direct GPU buffer upload
/// - Cache-efficient sequential access
pub struct SoaMesh {
    /// Vertex X coordinates.
    pub vertices_x: Vec<f64>,
    /// Vertex Y coordinates.
    pub vertices_y: Vec<f64>,
    /// Vertex Z coordinates.
    pub vertices_z: Vec<f64>,

    /// Face indices: each 3 consecutive values define a triangle.
    pub face_indices: Vec<u32>,

    /// Face normal X components.
    pub normals_x: Vec<f64>,
    /// Face normal Y components.
    pub normals_y: Vec<f64>,
    /// Face normal Z components.
    pub normals_z: Vec<f64>,

    /// Mesh metadata.
    pub info: MeshInfo,
}

impl SoaMesh {
    /// Creates an empty mesh.
    pub fn empty() -> Self {
        Self {
            vertices_x: Vec::new(),
            vertices_y: Vec::new(),
            vertices_z: Vec::new(),
            face_indices: Vec::new(),
            normals_x: Vec::new(),
            normals_y: Vec::new(),
            normals_z: Vec::new(),
            info: MeshInfo {
                format: MeshFormat::StlBinary,
                num_vertices: 0,
                num_faces: 0,
                num_cells: 0,
                bounding_box_min: [0.0; 3],
                bounding_box_max: [0.0; 3],
            },
        }
    }

    /// Number of vertices.
    pub fn num_vertices(&self) -> usize {
        self.vertices_x.len()
    }

    /// Number of triangular faces.
    pub fn num_faces(&self) -> usize {
        self.face_indices.len() / 3
    }
}

/// Trait for loading and accessing mesh geometry.
///
/// The UI uses this to load meshes and get rendering data.
/// The Core uses it to feed the solver with computational geometry.
pub trait MeshProvider: Send + Sync {
    /// Loads a mesh from a file path.
    fn load(&mut self, path: &std::path::Path) -> Result<MeshInfo, MeshError>;

    /// Returns vertex arrays in SoA layout: (x[], y[], z[]).
    fn vertices_soa(&self) -> (&[f64], &[f64], &[f64]);

    /// Returns face indices as a contiguous slice of triangles.
    fn face_indices(&self) -> &[u32];

    /// Returns face normals in SoA layout: (nx[], ny[], nz[]).
    fn face_normals_soa(&self) -> (&[f64], &[f64], &[f64]);

    /// Mesh summary information.
    fn info(&self) -> &MeshInfo;

    /// Unloads the mesh from memory.
    fn unload(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_info_characteristic_length() {
        let info = MeshInfo {
            format: MeshFormat::StlBinary,
            num_vertices: 100,
            num_faces: 50,
            num_cells: 0,
            bounding_box_min: [0.0, 0.0, 0.0],
            bounding_box_max: [2.0, 3.0, 1.0],
        };
        assert!((info.characteristic_length() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_soa_mesh_empty() {
        let mesh = SoaMesh::empty();
        assert_eq!(mesh.num_vertices(), 0);
        assert_eq!(mesh.num_faces(), 0);
    }
}
