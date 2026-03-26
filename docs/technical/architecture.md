# Architecture

> **English** | [Português abaixo ↓](#arquitetura)

---

## Table of Contents
- [Workspace Layout](#workspace-layout)
- [Module Responsibilities](#module-responsibilities)
- [Design Principles](#design-principles)
- [Memory Strategy](#memory-strategy)
- [Cross-Platform Notes](#cross-platform-notes)

---

## Workspace Layout

The Rust workspace lives entirely under `aerocore/`. It uses Cargo's workspace resolver v2 with shared dependency versions declared in `[workspace.dependencies]`.

```
aerocore/
├── Cargo.toml                  ← Workspace manifest (resolver = "2")
└── crates/
    ├── aerocore_core/          ← Core library — solvers, I/O, memory, math
    │   └── src/
    │       ├── lib.rs
    │       ├── io_mesh/        ← Mesh file I/O and geometry processing
    │       │   ├── mod.rs
    │       │   ├── mesh.rs     ← MeshInfo, SoaMesh, MeshProvider, MeshError
    │       │   ├── stl.rs      ← STL ASCII + binary parser (nom + memmap2)
    │       │   └── voxelize.rs ← STL → voxel mask pipeline
    │       ├── solvers/
    │       │   ├── mod.rs
    │       │   ├── traits.rs   ← Solver trait definitions
    │       │   ├── lbm/        ← Lattice Boltzmann solvers
    │       │   │   ├── mod.rs
    │       │   │   ├── collision.rs
    │       │   │   ├── streaming.rs
    │       │   │   ├── boundary.rs
    │       │   │   └── d3q19.rs
    │       │   └── navier_stokes/ ← Compressible NS stubs (planned)
    │       ├── memory/         ← Arena, pool, aligned allocators
    │       │   ├── arena.rs
    │       │   ├── pool.rs
    │       │   └── aligned.rs
    │       ├── math_core/      ← Precision traits, numeric helpers
    │       │   └── precision.rs
    │       └── gpu_compute/    ← WGPU/CUDA backend stubs (planned)
    │           ├── mod.rs
    │           └── device.rs
    ├── aerocore_ffi/           ← C-ABI FFI layer (extern "C" functions)
    │   └── src/
    │       ├── lib.rs
    │       └── types.rs
    └── aerocore_ui/            ← egui + wgpu frontend (planned)
```

---

## Module Responsibilities

### `io_mesh` ✅ Implemented

Handles all geometry file I/O and preprocessing:

- **`mesh.rs`**: Core types — `MeshFormat`, `MeshInfo`, `MeshError`, `SoaMesh`, `MeshProvider` trait.
- **`stl.rs`**: STL parser supporting both ASCII and binary formats. Binary parsing uses `nom` combinators for zero-copy `&[u8]` parsing and `memmap2::Mmap` for large files.
- **`voxelize.rs`**: Converts triangle meshes to voxel occupancy masks for use as LBM domain boundary conditions.

### `solvers/lbm` 🔄 In Progress

Lattice Boltzmann Method implementations:

- **`collision.rs`**: BGK (Bhatnagar-Gross-Krook) single-relaxation-time collision operator.
- **`streaming.rs`**: Streaming step — particle distribution propagation along velocity directions.
- **`boundary.rs`**: Boundary condition implementations (bounce-back, inflow/outflow).
- **`d3q19.rs`**: D3Q19 velocity set definition (19 discrete velocities in 3D). 📋 Planned
- D2Q9 implementation: 🔄 In Progress

### `solvers/navier_stokes` 📋 Planned

Compressible Navier-Stokes stubs. Intended for higher Mach number aeronautical cases.

### `memory` ✅ Implemented

Three-level memory strategy:

- **`arena.rs`** (`bumpalo`): O(1) bump allocation for short-lived per-timestep data. Reset in bulk at end of each step.
- **`pool.rs`** (`slab`): Stable index handles for long-lived entities (mesh cells, faces).
- **`aligned.rs`**: Cache-line aligned allocations for SIMD buffers.

### `math_core` ✅ Implemented

- **`precision.rs`**: `FloatPrecision` trait abstracting `f32`/`f64` — allows the same solver code to run at both precisions.

### `gpu_compute` 📋 Planned

- **`device.rs`**: Abstracted compute device — CPU fallback, WGPU (Vulkan/DX12/Metal), CUDA.

### `aerocore_ffi` ✅ Structural Scaffolding

C-ABI bindings generated via `cbindgen`. Exposes the core library to non-Rust consumers (Python, C/C++ integrations).

### `aerocore_ui` 📋 Planned

egui + wgpu frontend for interactive simulation setup, real-time visualization, and result exploration.

---

## Design Principles

### 1. Data-Oriented Design (DOD)

All simulation fields use Structure-of-Arrays (SoA) layout:

```
// AoS (avoided in hot paths)
struct Cell { f: [f64; 19], rho: f64, ux: f64, uy: f64, uz: f64 }
cells: Vec<Cell>

// SoA (used in AeroCore)
f: Vec<[f64; 19]>    // or flattened: f: Vec<f64> with stride
rho: Vec<f64>
ux:  Vec<f64>
uy:  Vec<f64>
uz:  Vec<f64>
```

Benefits: sequential SIMD-friendly access, direct GPU buffer upload, better cache line utilization.

### 2. Zero-allocation Hot Paths

The streaming and collision steps operate entirely on pre-allocated buffers. No `Vec::push` or heap allocation occurs during the simulation loop. All temporary allocations use arena reset instead of individual `drop`.

### 3. Dual Precision

The `FloatPrecision` trait allows running the same solver kernel at `f32` (speed) or `f64` (accuracy) with zero code duplication via monomorphization.

### 4. Separation of Concerns

- `aerocore_core` has **no UI dependency** — it is a pure library.
- The UI (`aerocore_ui`) calls into the core through well-defined traits.
- The FFI layer (`aerocore_ffi`) provides a stable C ABI for interoperability.

---

## Memory Strategy

| Layer | Crate | Use Case | Lifetime |
|-------|-------|----------|----------|
| Arena (bump) | `bumpalo` | Per-timestep temporaries | Reset each step |
| Pool (slab) | `slab` | Mesh entity handles | Persistent |
| Aligned heap | `aligned.rs` | SIMD buffers | Persistent |
| Memory-mapped | `memmap2` | Large STL files (> 1 GB) | File duration |
| Stack | — | Small fixed arrays | Frame |

---

## Cross-Platform Notes

- All file path handling uses `std::path::Path` (handles `/` vs `\` transparently).
- `memmap2` is cross-platform (Windows + Linux + macOS).
- `wgpu` supports DirectX 12 (Windows), Vulkan (Linux/Windows), Metal (macOS).
- `aerocore/build.bat` is a Windows convenience script wrapping `cargo build`.
- GitHub Actions CI (**planned**) will run on `windows-latest` and `ubuntu-latest` runners.

---

---

# Arquitetura

> [English above ↑](#architecture)

---

## Índice
- [Layout do Workspace](#layout-do-workspace)
- [Responsabilidades dos Módulos](#responsabilidades-dos-módulos)
- [Princípios de Design](#princípios-de-design)
- [Estratégia de Memória](#estratégia-de-memória)
- [Notas Multiplataforma](#notas-multiplataforma)

---

## Layout do Workspace

O workspace Rust fica inteiramente sob `aerocore/`. Ele usa o resolver v2 do Cargo com versões de dependências compartilhadas declaradas em `[workspace.dependencies]`.

```
aerocore/
├── Cargo.toml                  ← Manifesto do workspace (resolver = "2")
└── crates/
    ├── aerocore_core/          ← Biblioteca core — solvers, I/O, memória, matemática
    │   └── src/
    │       ├── lib.rs
    │       ├── io_mesh/        ← I/O de arquivo de malha e processamento de geometria
    │       │   ├── mod.rs
    │       │   ├── mesh.rs     ← MeshInfo, SoaMesh, MeshProvider, MeshError
    │       │   ├── stl.rs      ← Parser STL ASCII + binário (nom + memmap2)
    │       │   └── voxelize.rs ← Pipeline STL → máscara de voxel
    │       ├── solvers/
    │       │   ├── mod.rs
    │       │   ├── traits.rs   ← Definições de trait de solver
    │       │   ├── lbm/        ← Solvers Lattice Boltzmann
    │       │   └── navier_stokes/ ← Stubs NS compressível (planejado)
    │       ├── memory/         ← Alocadores arena, pool, alinhado
    │       ├── math_core/      ← Traits de precisão, helpers numéricos
    │       └── gpu_compute/    ← Stubs de backend WGPU/CUDA (planejado)
    ├── aerocore_ffi/           ← Camada FFI C-ABI
    └── aerocore_ui/            ← Frontend egui + wgpu (planejado)
```

---

## Responsabilidades dos Módulos

### `io_mesh` ✅ Implementado

Gerencia todo o I/O de arquivos de geometria e pré-processamento:

- **`mesh.rs`**: Tipos centrais — `MeshFormat`, `MeshInfo`, `MeshError`, `SoaMesh`, trait `MeshProvider`.
- **`stl.rs`**: Parser STL suportando formatos ASCII e binário. O parsing binário usa combinadores `nom` para parsing zero-copy de `&[u8]` e `memmap2::Mmap` para arquivos grandes.
- **`voxelize.rs`**: Converte malhas de triângulos em máscaras de ocupação de voxel para uso como condições de contorno do domínio LBM.

### `solvers/lbm` 🔄 Em andamento

Implementações do Método de Lattice Boltzmann:

- **`collision.rs`**: Operador de colisão BGK (Bhatnagar-Gross-Krook) com tempo de relaxação único.
- **`streaming.rs`**: Etapa de streaming — propagação da distribuição de partículas ao longo das direções de velocidade.
- **`boundary.rs`**: Implementações de condições de contorno (bounce-back, entrada/saída).
- **`d3q19.rs`**: Definição do conjunto de velocidades D3Q19 (19 velocidades discretas em 3D). 📋 Planejado

### `memory` ✅ Implementado

Estratégia de memória em três níveis:

- **`arena.rs`** (`bumpalo`): Alocação bump O(1) para dados temporários de curta duração por passo de tempo.
- **`pool.rs`** (`slab`): Handles de índice estáveis para entidades de longa duração (células de malha, faces).
- **`aligned.rs`**: Alocações alinhadas à linha de cache para buffers SIMD.

---

## Princípios de Design

### 1. Design Orientado a Dados (DOD)

Todos os campos de simulação usam layout Structure-of-Arrays (SoA):

```
// SoA (usado no AeroCore)
rho: Vec<f64>   // densidade
ux:  Vec<f64>   // velocidade x
uy:  Vec<f64>   // velocidade y
uz:  Vec<f64>   // velocidade z
f:   Vec<f64>   // distribuições (stride = Q)
```

Benefícios: acesso sequencial amigável para SIMD, upload direto para buffer GPU, melhor utilização de linha de cache.

### 2. Caminhos Quentes Sem Alocação

As etapas de streaming e colisão operam inteiramente em buffers pré-alocados. Nenhum `Vec::push` ou alocação de heap ocorre durante o loop de simulação.

### 3. Precisão Dupla

O trait `FloatPrecision` permite executar o mesmo kernel de solver em `f32` (velocidade) ou `f64` (precisão) sem duplicação de código via monomorfização.

---

## Estratégia de Memória

| Camada | Crate | Caso de Uso | Tempo de Vida |
|--------|-------|-------------|---------------|
| Arena (bump) | `bumpalo` | Temporários por passo de tempo | Reset a cada passo |
| Pool (slab) | `slab` | Handles de entidades de malha | Persistente |
| Heap alinhado | `aligned.rs` | Buffers SIMD | Persistente |
| Mapeado em memória | `memmap2` | Arquivos STL grandes (> 1 GB) | Duração do arquivo |
| Pilha | — | Arrays fixos pequenos | Frame |

---

## Notas Multiplataforma

- Todo o tratamento de caminho de arquivo usa `std::path::Path` (trata `/` vs `\` de forma transparente).
- `memmap2` é multiplataforma (Windows + Linux + macOS).
- `wgpu` suporta DirectX 12 (Windows), Vulkan (Linux/Windows), Metal (macOS).
- `aerocore/build.bat` é um script de conveniência Windows que envolve `cargo build`.
- CI GitHub Actions (**planejado**) rodará em runners `windows-latest` e `ubuntu-latest`.
