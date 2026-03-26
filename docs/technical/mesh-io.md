# Mesh I/O

> **English** | [Português abaixo ↓](#io-de-malha)

---

## Table of Contents
- [Overview](#overview)
- [STL File Format](#stl-file-format)
- [STL Parser](#stl-parser)
- [SoaMesh Data Structure](#soamesh-data-structure)
- [MeshProvider Trait](#meshprovider-trait)
- [Voxelization Pipeline](#voxelization-pipeline)
- [Performance & Benchmarks](#performance--benchmarks)
- [Error Handling](#error-handling)
- [Future Formats](#future-formats)

---

## Overview

The `io_mesh` module (in `aerocore/crates/aerocore_core/src/io_mesh/`) is responsible for:

1. **Reading geometry files** (currently STL ASCII and binary; CGNS planned).
2. **Storing geometry** in a cache-efficient SoA format (`SoaMesh`).
3. **Converting geometry to simulation domains** via voxelization (STL → voxel mask → LBM boundary conditions).

---

## STL File Format

STL (STereoLithography) is a simple triangulated surface mesh format, available in two variants:

### Binary STL (preferred)
```
UINT8[80]   — header (ignored by parser)
UINT32      — number of triangles
For each triangle:
  REAL32[3] — normal vector (nx, ny, nz)
  REAL32[3] — vertex 1 (x, y, z)
  REAL32[3] — vertex 2 (x, y, z)
  REAL32[3] — vertex 3 (x, y, z)
  UINT16    — attribute byte count (usually 0)
```
Total size: `84 + 50 * num_triangles` bytes.

### ASCII STL
```
solid <name>
  facet normal <nx> <ny> <nz>
    outer loop
      vertex <x> <y> <z>
      vertex <x> <y> <z>
      vertex <x> <y> <z>
    endloop
  endfacet
  ...
endsolid <name>
```

The parser auto-detects format by checking whether the first 5 bytes spell `solid` (ASCII hint) and validating the expected binary size.

---

## STL Parser

**Source:** `aerocore/crates/aerocore_core/src/io_mesh/stl.rs`

### Binary Parser — `parse_stl_binary`

Uses **`nom` parser combinators** for zero-copy, safe parsing of raw `&[u8]`:

- `nom::number::complete::le_f32` — reads 4-byte little-endian floats.
- `nom::multi::count` — repeats a sub-parser N times without intermediate allocation.
- Triangle count validation: ensures reported count matches actual byte length before parsing.
- Bounding box computed during iteration (no second pass).

### File Loader — `load_stl`

Uses **`memmap2::Mmap`** for memory-mapped file access:

```rust
pub fn load_stl(path: &std::path::Path) -> Result<SoaMesh, MeshError>
```

- Opens the file and maps it into virtual address space.
- The OS pages data lazily — only accessed bytes are loaded from disk.
- Suitable for meshes larger than available RAM (paged I/O).
- Works on both Windows and Linux (cross-platform via `memmap2`).

### ASCII Parser — `parse_stl_ascii`

State-machine parser over `&str` lines:
- States: `Idle → InFacet → InLoop → InVertex(n)`
- Trims whitespace, splits by space, parses f64 coordinates.
- Returns `ParseError` with line number on malformed input.

### Auto-detection — `parse_stl`

```rust
pub fn parse_stl(data: &[u8]) -> Result<SoaMesh, MeshError>
```

Selects binary or ASCII based on header bytes and size validation.

---

## SoaMesh Data Structure

**Source:** `aerocore/crates/aerocore_core/src/io_mesh/mesh.rs`

```rust
pub struct SoaMesh {
    pub vertices_x: Vec<f64>,  // X coordinates of all vertices
    pub vertices_y: Vec<f64>,  // Y coordinates
    pub vertices_z: Vec<f64>,  // Z coordinates

    pub face_indices: Vec<u32>, // 3 indices per triangle face

    pub normals_x: Vec<f64>,   // Face normal X components
    pub normals_y: Vec<f64>,   // Face normal Y components
    pub normals_z: Vec<f64>,   // Face normal Z components

    pub info: MeshInfo,         // Metadata: counts, bounding box, format
}
```

### Why SoA?

Structure-of-Arrays enables:

| Operation | Benefit |
|-----------|---------|
| Iterate all X coords | Sequential memory → prefetcher-friendly |
| SIMD over vertices | Aligned f64 arrays → AVX2/AVX-512 vectorization |
| GPU buffer upload | Direct `memcpy` of `vertices_x` → GPU vertex buffer |
| Bounding box pass | Single read over 3 arrays → cache-efficient |

Contrast with Array-of-Structs (`Vec<[f64; 3]>`): accessing only X coords would stride over Y and Z, wasting cache lines.

### MeshInfo

```rust
pub struct MeshInfo {
    pub format: MeshFormat,
    pub num_vertices: usize,
    pub num_faces: usize,
    pub num_cells: usize,
    pub bounding_box_min: [f64; 3],
    pub bounding_box_max: [f64; 3],
}
```

Helper methods:
- `extents() -> [f64; 3]` — axis-aligned extents.
- `characteristic_length() -> f64` — max extent, useful for Re/Mach number pre-checks.

---

## MeshProvider Trait

```rust
pub trait MeshProvider: Send + Sync {
    fn load(&mut self, path: &std::path::Path) -> Result<MeshInfo, MeshError>;
    fn vertices_soa(&self) -> (&[f64], &[f64], &[f64]);
    fn face_indices(&self) -> &[u32];
    fn face_normals_soa(&self) -> (&[f64], &[f64], &[f64]);
    fn info(&self) -> &MeshInfo;
    fn unload(&mut self);
}
```

The solver and UI code interact with mesh data through this trait — the concrete loader (`StlLoader`, future `CgnsLoader`) is swappable without changing the consumer.

---

## Voxelization Pipeline

**Source:** `aerocore/crates/aerocore_core/src/io_mesh/voxelize.rs`  
**Status:** ✅ Implemented

Converts a triangulated surface mesh into a 3D occupancy grid:

```
STL file
  ↓  load_stl / parse_stl
SoaMesh (triangles in world space)
  ↓  voxelize(mesh, resolution, domain_bounds)
VoxelMask (3D bool grid: solid=true, fluid=false)
  ↓  apply_to_lbm_domain(solver, mask)
LBM domain with boundary conditions
```

The voxel mask is consumed by the LBM solver to mark cells as solid obstacles (bounce-back boundary condition).

---

## Performance & Benchmarks

Benchmarks live in `aerocore/crates/aerocore_core/benches/stl_bench.rs`.

Run with:
```bash
cd aerocore
cargo bench --bench stl_bench
```

Metrics reported:
- **Throughput** (MB/s) — raw I/O + parsing speed.
- **Triangles/s** — parsing rate normalized to geometry complexity.

> Note: Benchmark data is not yet published. Baseline numbers to be established in Sprint S2.

---

## Error Handling

```rust
pub enum MeshError {
    FileNotFound(String),
    ParseError { line: usize, message: String },
    UnsupportedFormat(String),
    IoError(std::io::Error),
}
```

All errors implement `std::error::Error` and `Display`. The `From<std::io::Error>` impl allows transparent `?` propagation from `std::fs` calls.

---

## Future Formats

| Format | Status | Notes |
|--------|--------|-------|
| STL Binary | ✅ Implemented | Primary format, mmap-accelerated |
| STL ASCII | ✅ Implemented | Fallback, human-readable |
| CGNS | 📋 Planned | Industry standard for structured/unstructured grids |
| VTK legacy | 📋 Planned | Output format for visualization |
| OpenFOAM mesh | 📋 Planned | Long-term interoperability goal |

---

---

# I/O de Malha

> [English above ↑](#mesh-io)

---

## Índice
- [Visão Geral](#visão-geral)
- [Formato de Arquivo STL](#formato-de-arquivo-stl)
- [Parser STL](#parser-stl)
- [Estrutura de Dados SoaMesh](#estrutura-de-dados-soamesh)
- [Trait MeshProvider](#trait-meshprovider)
- [Pipeline de Voxelização](#pipeline-de-voxelização)
- [Performance e Benchmarks](#performance-e-benchmarks)
- [Tratamento de Erros](#tratamento-de-erros)
- [Formatos Futuros](#formatos-futuros)

---

## Visão Geral

O módulo `io_mesh` (em `aerocore/crates/aerocore_core/src/io_mesh/`) é responsável por:

1. **Ler arquivos de geometria** (atualmente STL ASCII e binário; CGNS planejado).
2. **Armazenar geometria** em formato SoA eficiente em cache (`SoaMesh`).
3. **Converter geometria em domínios de simulação** via voxelização (STL → máscara de voxel → condições de contorno LBM).

---

## Formato de Arquivo STL

STL (STereoLithography) é um formato simples de malha de superfície triangulada, disponível em duas variantes:

### STL Binário (preferido)
```
UINT8[80]   — cabeçalho (ignorado pelo parser)
UINT32      — número de triângulos
Para cada triângulo:
  REAL32[3] — vetor normal (nx, ny, nz)
  REAL32[3] — vértice 1 (x, y, z)
  REAL32[3] — vértice 2 (x, y, z)
  REAL32[3] — vértice 3 (x, y, z)
  UINT16    — contagem de bytes de atributo (geralmente 0)
```
Tamanho total: `84 + 50 * num_triângulos` bytes.

---

## Parser STL

**Fonte:** `aerocore/crates/aerocore_core/src/io_mesh/stl.rs`

### Parser Binário — `parse_stl_binary`

Usa **combinadores de parser `nom`** para parsing zero-copy seguro de `&[u8]` bruto.

### Carregador de Arquivo — `load_stl`

Usa **`memmap2::Mmap`** para acesso a arquivo mapeado em memória:

- Abre o arquivo e mapeia-o no espaço de endereço virtual.
- O SO carrega as páginas de dados preguiçosamente — apenas os bytes acessados são carregados do disco.
- Adequado para malhas maiores que a RAM disponível (I/O paginado).
- Funciona tanto no Windows quanto no Linux (multiplataforma via `memmap2`).

---

## Estrutura de Dados SoaMesh

**Fonte:** `aerocore/crates/aerocore_core/src/io_mesh/mesh.rs`

```rust
pub struct SoaMesh {
    pub vertices_x: Vec<f64>,   // Coordenadas X de todos os vértices
    pub vertices_y: Vec<f64>,   // Coordenadas Y
    pub vertices_z: Vec<f64>,   // Coordenadas Z
    pub face_indices: Vec<u32>, // 3 índices por face triangular
    pub normals_x: Vec<f64>,    // Componentes X da normal da face
    pub normals_y: Vec<f64>,    // Componentes Y
    pub normals_z: Vec<f64>,    // Componentes Z
    pub info: MeshInfo,         // Metadados: contagens, bounding box, formato
}
```

### Por que SoA?

Structure-of-Arrays permite:
- Iteração sequencial sobre coordenadas individuais → amigável ao prefetcher.
- Vetorização SIMD (AVX2/AVX-512) sobre arrays alinhados de f64.
- Upload direto de buffer GPU via `memcpy` dos arrays.
- Passagem eficiente de bounding box com leitura de apenas 3 arrays.

---

## Pipeline de Voxelização

**Fonte:** `aerocore/crates/aerocore_core/src/io_mesh/voxelize.rs`  
**Status:** ✅ Implementado

Converte uma malha de superfície triangulada em uma grade de ocupação 3D:

```
Arquivo STL
  ↓  load_stl / parse_stl
SoaMesh (triângulos no espaço do mundo)
  ↓  voxelize(mesh, resolução, limites_do_domínio)
VoxelMask (grade booleana 3D: sólido=true, fluido=false)
  ↓  apply_to_lbm_domain(solver, máscara)
Domínio LBM com condições de contorno
```

---

## Formatos Futuros

| Formato | Status | Notas |
|---------|--------|-------|
| STL Binário | ✅ Implementado | Formato primário, acelerado por mmap |
| STL ASCII | ✅ Implementado | Fallback, legível por humanos |
| CGNS | 📋 Planejado | Padrão industrial para grades estruturadas/não-estruturadas |
| VTK legado | 📋 Planejado | Formato de saída para visualização |
| Malha OpenFOAM | 📋 Planejado | Objetivo de interoperabilidade de longo prazo |
