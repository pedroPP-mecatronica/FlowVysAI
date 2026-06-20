# Tech Stack

## Linguagem & Plataforma
- **Rust** (stable ≥ 1.75)
- **Edição**: 2021
- **Plataformas**: Windows (primário) + Linux
- **Licença**: MIT OR Apache-2.0

## Build System
- **Cargo** com workspace resolver "2"
- Workspace root: `aerocore/Cargo.toml`
- Versão unificada do workspace: `0.1.0` (definida em `[workspace.package]`)
- **Todas as versões de dependências** são declaradas em `[workspace.dependencies]` no `Cargo.toml` raiz — crates filhos herdam via `{ workspace = true }`

## Crates do Workspace

| Crate | Tipo | Propósito |
|---|---|---|
| `aerocore_core` | lib | Motor CFD: solvers, I/O de malha, memória, matemática, GPU |
| `aerocore_ffi` | cdylib/staticlib | Camada FFI com ABI C (`extern "C"`, `cbindgen`) |
| `aerocore_ui` | binary | Frontend egui + wgpu (planejado) |

## Dependências Principais

### Memória
| Crate | Versão | Uso |
|---|---|---|
| `bumpalo` | 3.16 | Arena allocator — alocação O(1) por bump pointer |
| `slab` | 0.4 | Pool allocator com índices estáveis |

### Matemática / Numérica
| Crate | Versão | Uso |
|---|---|---|
| `nalgebra` | 0.33 | Álgebra linear, vetores, matrizes |
| `num-traits` | 0.2 | Traits genéricas de ponto flutuante (`Float`, `FromPrimitive`) |
| `approx` | 0.5 | Comparações FP com tolerância (`relative_eq!`, `abs_diff_eq!`) |

### Paralelismo
| Crate | Versão | Uso |
|---|---|---|
| `rayon` | 1.10 | Paralelismo dados-paralelo (`par_iter()`) nos loops de solver |
| `crossbeam` | 0.8 | Channels e threads com escopo para comunicação Core → UI |

### I/O de Malha
| Crate | Versão | Uso |
|---|---|---|
| `nom` | 7.1 | Parser combinators zero-copy (STL, CGNS) |
| `memmap2` | 0.9 | Memory-mapped I/O para malhas grandes (>1 GB) |

### Serialização
| Crate | Versão | Uso |
|---|---|---|
| `serde` | 1.0 | Serialização com derive — configs de simulação |
| `serde_json` | 1.0 | Formato JSON para configs |

### Infraestrutura
| Crate | Versão | Uso |
|---|---|---|
| `tracing` | 0.1 | Logging estruturado e instrumentação |
| `criterion` | 0.5 | Benchmarking estatístico dos hot paths (com `html_reports`) |

## Princípios de Performance (obrigatórios)

- **Zero alocação em hot path**: `step()` do solver NUNCA aloca. Toda alocação ocorre em `init()` via `SimArena`.
- **SoA (Structure of Arrays)**: layouts de memória SoA para máxima cache-efficiency e compatibilidade com GPU.
- **FP64 por padrão**: precisão dupla (`f64`) é o default. `f32` é opt-in explícito via `PrecisionMode::FP32`.
- **Trait `FloatPrecision`**: todo código numérico é genérico sobre `T: FloatPrecision`, suportando `f32` | `f64` com o mesmo código.
- **Cache-line alignment**: buffers críticos alocados com alinhamento de 64 bytes (`alloc_aligned_slice`) para evitar false sharing.

## Comandos Comuns

```bash
# Build (dentro de aerocore/)
cargo build
cargo build --release

# Testes
cargo test
cargo test --workspace

# Benchmarks
cargo bench

# Qualidade (obrigatório antes de PR)
cargo fmt --check
cargo clippy -- -D warnings

# Gerar headers C via FFI
cbindgen --crate aerocore_ffi --output include/aerocore.h
```

> **Windows**: use PowerShell ou Git Bash. `aerocore/build.bat` disponível como wrapper.

## Convenções de Código

- Todos os erros são tipos `enum` explícitos — sem `unwrap()` em código de produção
- `#[repr(C)]` em structs expostas via FFI
- Testes de integração em `crates/aerocore_core/tests/`; testes unitários inline nos módulos
- Benchmarks em `crates/aerocore_core/benches/` com `criterion`
- Validações numéricas usam `approx::assert_relative_eq!` com tolerâncias explícitas
