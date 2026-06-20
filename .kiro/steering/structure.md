# Estrutura do Projeto

## Raiz: `FlowVysAI/`

```
FlowVysAI/
├── README.md                   ← Visão geral e quickstart
├── Architecture_Manifest.md    ← Decisões arquiteturais, contratos de interface e boilerplate
├── aerocore/                   ← Workspace Rust (todo o código do motor)
│   ├── Cargo.toml              ← Workspace manifest + dependências compartilhadas
│   ├── Cargo.lock
│   ├── build.bat               ← Wrapper Windows para cargo build
│   └── crates/
│       ├── aerocore_core/      ← Biblioteca principal (lib crate)
│       ├── aerocore_ffi/       ← Camada FFI com ABI C
│       └── aerocore_ui/        ← Frontend egui + wgpu (binary crate)
├── src/                        ← Área legada / rascunho
├── docs/                       ← Documentação do projeto
│   ├── technical/              ← Docs para desenvolvedores e pesquisadores
│   └── engineering/            ← Docs para usuários finais / engenheiros
└── .github/
    ├── workflows/ci.yml        ← CI GitHub Actions (Windows + Linux)
    └── ISSUE_TEMPLATE/         ← Templates de bug, feature, research
```

## Módulo: `aerocore_core`

Biblioteca principal do motor. Não possui `main.rs`.

```
aerocore_core/
├── Cargo.toml
├── src/
│   ├── lib.rs                  ← Re-exporta todos os módulos públicos
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── arena.rs            ← SimArena (bumpalo) — alocação O(1) por timestep
│   │   ├── pool.rs             ← ObjectPool (slab) — entidades com índices estáveis
│   │   └── aligned.rs          ← Alocações cache-line aligned (64 bytes)
│   ├── math_core/
│   │   ├── mod.rs
│   │   ├── precision.rs        ← Trait FloatPrecision: f32 | f64
│   │   ├── vector.rs           ← Vec3<T>, Vec4<T>
│   │   ├── matrix.rs           ← Mat3x3<T>, Mat4x4<T>
│   │   └── tensor.rs           ← Tensor de stress/deformação
│   ├── solvers/
│   │   ├── mod.rs
│   │   ├── traits.rs           ← Trait Solver, StepResult, SolverConfig, FieldDataBuffer
│   │   ├── lbm/
│   │   │   ├── mod.rs
│   │   │   ├── d2q9.rs         ← Modelo de velocidades D2Q9 (2D) — em andamento
│   │   │   ├── d3q19.rs        ← Modelo de velocidades D3Q19 (3D) — planejado
│   │   │   ├── collision.rs    ← Operador BGK / MRT
│   │   │   ├── streaming.rs    ← Propagação do lattice
│   │   │   ├── boundary.rs     ← Bounce-back, Zou-He
│   │   │   └── lbm2d.rs        ← Solver LBM 2D completo
│   │   └── navier_stokes/
│   │       ├── mod.rs
│   │       ├── compressible.rs ← Solver NS compressível
│   │       ├── boundary.rs     ← Condições de contorno NS
│   │       └── turbulence.rs   ← Modelos k-ε, k-ω SST
│   ├── gpu_compute/
│   │   ├── mod.rs
│   │   └── device.rs           ← Trait GpuComputeBackend, GpuDeviceInfo, GpuBufferHandle
│   └── io_mesh/
│       ├── mod.rs
│       ├── stl.rs              ← Parser STL (ASCII + binário) via nom + memmap2
│       ├── voxelize.rs         ← Pipeline STL → voxel mask
│       └── mesh.rs             ← SoaMesh, MeshInfo, Trait MeshProvider
├── benches/
│   ├── memory_bench.rs         ← Benchmarks de alocadores
│   └── stl_bench.rs            ← Benchmarks do parser STL
├── tests/
│   ├── lid_driven_cavity.rs    ← Validação: cavidade com tampa deslizante
│   └── cavity2d_s1_validation.rs
└── examples/
    └── cavity2d.rs
```

## Módulo: `aerocore_ffi`

Camada de interoperabilidade C. Expõe a API do Core via `extern "C"`.

```
aerocore_ffi/
├── Cargo.toml
└── src/
    ├── lib.rs      ← Funções extern "C" exportadas
    └── types.rs    ← Structs #[repr(C)] para tipos de fronteira
```

## Módulo: `aerocore_ui` (planejado)

Frontend egui + wgpu. Binary crate com `main.rs`.

```
aerocore_ui/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── app.rs                  ← Estado da aplicação egui
    ├── viewport_3d.rs          ← Renderizador de malha wgpu
    ├── bridge.rs               ← Consome FFI / canal crossbeam
    └── panels/
        ├── solver_config.rs
        ├── mesh_inspector.rs
        └── simulation_monitor.rs
```

## Contratos de Interface (Core ↔ UI)

Definidos em `aerocore_core/src/solvers/traits.rs`. A UI **nunca** acessa internals do solver.

| Trait / Tipo | Arquivo | Propósito |
|---|---|---|
| `Solver` | `solvers/traits.rs` | Contrato do motor: `init()`, `step()`, `snapshot_field_data()` |
| `FieldDataBuffer` | `solvers/traits.rs` | Transferência Core → UI em layout SoA (raw pointers) |
| `MeshProvider` | `io_mesh/mesh.rs` | Carregamento e acesso à geometria |
| `SimulationOrchestrator` | `aerocore_ffi/lib.rs` | Controle do ciclo de vida pela UI via command queue |
| `GpuComputeBackend` | `gpu_compute/device.rs` | Abstração WGPU / CUDA sob interface unificada |

## Convenções

- **Novos módulos no workspace**: declarar em `aerocore/Cargo.toml` → `members = [...]`
- **Novas dependências**: adicionar em `[workspace.dependencies]` no `aerocore/Cargo.toml`, depois herdar nos crates filhos com `{ workspace = true }`
- **Visibilidade**: usar `pub(crate)` para internals; `pub` apenas para API pública do crate
- **Erros**: cada módulo define seu próprio `enum XxxError` — sem `anyhow` ou `Box<dyn Error>` no core
- **Testes de integração**: em `tests/` na raiz do crate, nomeados pelo caso de validação CFD
- **Benchmarks**: em `benches/` com `criterion`, sempre comparando contra uma baseline documentada
- **Shaders WGSL**: em `aerocore/shaders/` compartilhados entre `aerocore_core` e `aerocore_ui`
