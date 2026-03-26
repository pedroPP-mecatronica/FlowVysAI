# FlowVysAI · AeroCore CFD Engine

<!-- CI badge placeholder — will be active once workflow is merged -->
![CI](https://img.shields.io/badge/CI-planned-lightgrey?logo=github-actions)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-informational)
![Language](https://img.shields.io/badge/language-Rust-orange?logo=rust)

---

> **English** | [Português abaixo ↓](#flowvysai--aerocore-cfd-engine-pt)

---

## Table of Contents / Índice

- [Overview (EN)](#overview)
- [Goals](#goals)
- [Current Status](#current-status)
- [Repository Layout](#repository-layout)
- [Quickstart](#quickstart)
- [How to Run](#how-to-run)
- [Roadmap](#roadmap)
- [CI & Testing](#ci--testing)
- [How to Contribute](#how-to-contribute)
- [Deeper Docs](#deeper-docs)
- [Visão Geral (PT)](#visão-geral)

---

## Overview

**FlowVysAI** is an open-source, high-performance Computational Fluid Dynamics (CFD) engine written in **Rust**, designed for aeronautical and automotive simulation workflows.  
The Rust workspace lives under **`aerocore/`** and follows a data-oriented design with Structure-of-Arrays (SoA) memory layouts, zero-allocation hot paths, and cross-platform support (**Windows + Linux**).

The engine is being built in a sprint-based progression:

| Phase | Method | Status |
|-------|--------|--------|
| 2D solver | Lattice Boltzmann D2Q9 | 🔄 In Progress |
| 3D solver | Lattice Boltzmann D3Q19 | 📋 Planned |
| Thermal / Environment | LBM thermal coupling | 📋 Planned |

---

## Goals

- Provide an accurate, reproducible, and performant LBM-based CFD solver for engineering use cases.
- Maintain a **didactic learning progression**: 2D → 3D → thermal/environment, so the codebase serves as a reference for researchers and engineers alike.
- Enforce quality from day one: `cargo fmt`, `cargo clippy -D warnings`, benchmarks (MLUPS), and validation against classical cases (Poiseuille, lid-driven cavity, channel with obstacle).
- Deliver a **cross-platform** experience — Windows is a first-class target alongside Linux.

---

## Current Status

| Module | Status |
|--------|--------|
| `io_mesh` — STL parser (ASCII + Binary) | ✅ Implemented |
| `io_mesh` — STL → voxel mask pipeline | ✅ Implemented |
| `io_mesh` — CGNS format | 📋 Planned |
| `solvers/lbm` — D2Q9 skeleton | 🔄 In Progress |
| `solvers/lbm` — D3Q19 | 📋 Planned |
| `solvers/lbm` — Thermal coupling | 📋 Planned |
| `solvers/navier_stokes` | 📋 Planned |
| `gpu_compute` — WGPU backend | 📋 Planned |
| `memory` — Arena / Pool allocators | ✅ Implemented |
| CI — GitHub Actions (Windows + Linux) | 📋 Planned |
| UI — egui / wgpu frontend | 📋 Planned |

---

## Repository Layout

```
FlowVysAI/
├── README.md                   ← This file
├── Architecture_Manifest.md    ← Architectural decisions and library choices
├── aerocore/                   ← Rust workspace (all engine code lives here)
│   ├── Cargo.toml              ← Workspace manifest
│   └── crates/
│       ├── aerocore_core/      ← Core library (solvers, mesh I/O, memory, math)
│       │   ├── src/
│       │   │   ├── io_mesh/    ← STL parser, SoaMesh, voxelizer
│       │   │   ├── solvers/    ← LBM (D2Q9, D3Q19), Navier-Stokes stubs
│       │   │   ├── memory/     ← Arena, pool, aligned allocators
│       │   │   ├── math_core/  ← Precision traits, numeric helpers
│       │   │   └── gpu_compute/← WGPU/CUDA backend stubs
│       │   ├── benches/        ← Criterion benchmarks
│       │   └── tests/          ← Integration tests
│       ├── aerocore_ffi/       ← C-compatible FFI layer
│       └── aerocore_ui/        ← egui + wgpu frontend (planned)
├── src/                        ← Legacy / scratch area
└── docs/                       ← Project documentation
    ├── README.md               ← Doc index
    ├── technical/              ← Developer / researcher docs
    └── engineering/            ← Client-facing / end-user docs
```

---

## Quickstart

**Requirements:** [Rust stable toolchain](https://rustup.rs/) (≥ 1.75)

```bash
# Clone
git clone https://github.com/pedroPP-mecatronica/FlowVysAI.git
cd FlowVysAI

# Build (Windows or Linux — same command)
cd aerocore
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

> **Windows users:** use PowerShell or Git Bash. `aerocore/build.bat` is available as a convenience wrapper.

---

## How to Run

Currently the engine exposes a **library API** — there is no standalone binary yet.  
To exercise the mesh I/O module:

```rust
use aerocore_core::io_mesh::stl::load_stl;

let mesh = load_stl(std::path::Path::new("my_model.stl")).unwrap();
println!("Loaded {} faces", mesh.num_faces());
```

Integration test cases (Poiseuille flow, lid-driven cavity) are in `aerocore/crates/aerocore_core/tests/`.  
A CLI runner and configuration file support (TOML) are **planned** in a future sprint.

---

## Roadmap

See [`docs/technical/solver-roadmap.md`](docs/technical/solver-roadmap.md) for the full sprint plan.

| Sprint | Focus |
|--------|-------|
| S1 | 2D LBM D2Q9 baseline + Poiseuille validation |
| S2 | STL I/O + voxelization pipeline ✅ |
| S3 | Case-driven pipeline (TOML config + CLI) |
| S4 | Numerical quality & robustness |
| S5 | 3D LBM D3Q19 |
| S6 | Thermal coupling |
| S7 | Environment / buoyancy effects |
| S8 | Polish, performance tuning, UI |

---

## CI & Testing

> **Planned** — GitHub Actions CI (Windows + Linux) is not yet merged.  
> Expected gates once CI is active:

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench (compile-check)
```

All validation tests target classical CFD benchmarks with quantitative error bounds (L2 norm of velocity profile, etc.).

---

## How to Contribute

1. Fork → branch (`feat/my-feature` or `fix/my-bug`).
2. Ensure `cargo fmt` and `cargo clippy -D warnings` pass locally.
3. Add or update tests. Integration tests live in `tests/`, unit tests inline.
4. Open a Pull Request with a short description of what changes and why.
5. If you are making a significant architectural decision, include an ADR in `docs/adr/` (see `Architecture_Manifest.md`).

Contributions that improve validation coverage, add benchmark cases, or extend platform support are especially welcome.

---

## Deeper Docs

| Audience | Link |
|----------|------|
| Developers / Researchers | [`docs/technical/`](docs/technical/README.md) |
| Engineers / End Users | [`docs/engineering/`](docs/engineering/README.md) |
| CFD / LBM Glossary | [`docs/glossary.md`](docs/glossary.md) |

---

---

# FlowVysAI · AeroCore CFD Engine (PT)

> [English above ↑](#flowvysai--aerocore-cfd-engine)

---

## Visão Geral

**FlowVysAI** é um motor de Dinâmica dos Fluidos Computacional (CFD) de alto desempenho, de código aberto, escrito em **Rust**, projetado para fluxos de trabalho de simulação aeronáutica e automotiva.  
O workspace Rust fica em **`aerocore/`** e segue um design orientado a dados com layouts de memória Structure-of-Arrays (SoA), caminhos quentes sem alocação e suporte multiplataforma (**Windows + Linux**).

O motor é desenvolvido em uma progressão baseada em sprints:

| Fase | Método | Status |
|------|--------|--------|
| Solver 2D | Lattice Boltzmann D2Q9 | 🔄 Em andamento |
| Solver 3D | Lattice Boltzmann D3Q19 | 📋 Planejado |
| Térmico / Ambiente | Acoplamento térmico LBM | 📋 Planejado |

---

## Objetivos

- Fornecer um solver CFD baseado em LBM preciso, reproduzível e de alto desempenho para casos de uso de engenharia.
- Manter uma **progressão de aprendizado didática**: 2D → 3D → térmico/ambiente, para que a base de código sirva de referência para pesquisadores e engenheiros.
- Garantir qualidade desde o início: `cargo fmt`, `cargo clippy -D warnings`, benchmarks (MLUPS) e validação com casos clássicos (Poiseuille, cavidade com tampa deslizante, canal com obstáculo).
- Entregar uma experiência **multiplataforma** — Windows é um alvo de primeira classe ao lado do Linux.

---

## Status Atual

| Módulo | Status |
|--------|--------|
| `io_mesh` — Parser STL (ASCII + Binário) | ✅ Implementado |
| `io_mesh` — Pipeline STL → máscara de voxel | ✅ Implementado |
| `io_mesh` — Formato CGNS | 📋 Planejado |
| `solvers/lbm` — Esqueleto D2Q9 | 🔄 Em andamento |
| `solvers/lbm` — D3Q19 | 📋 Planejado |
| `solvers/lbm` — Acoplamento térmico | 📋 Planejado |
| `solvers/navier_stokes` | 📋 Planejado |
| `gpu_compute` — Backend WGPU | 📋 Planejado |
| `memory` — Alocadores Arena / Pool | ✅ Implementado |
| CI — GitHub Actions (Windows + Linux) | 📋 Planejado |
| UI — Frontend egui / wgpu | 📋 Planejado |

---

## Estrutura do Repositório

```
FlowVysAI/
├── README.md                   ← Este arquivo
├── Architecture_Manifest.md    ← Decisões arquiteturais e escolhas de biblioteca
├── aerocore/                   ← Workspace Rust (todo o código do motor aqui)
│   ├── Cargo.toml              ← Manifesto do workspace
│   └── crates/
│       ├── aerocore_core/      ← Biblioteca core (solvers, I/O de malha, memória, matemática)
│       │   ├── src/
│       │   │   ├── io_mesh/    ← Parser STL, SoaMesh, voxelizador
│       │   │   ├── solvers/    ← LBM (D2Q9, D3Q19), stubs Navier-Stokes
│       │   │   ├── memory/     ← Alocadores arena, pool, alinhado
│       │   │   ├── math_core/  ← Traits de precisão, helpers numéricos
│       │   │   └── gpu_compute/← Stubs de backend WGPU/CUDA
│       │   ├── benches/        ← Benchmarks Criterion
│       │   └── tests/          ← Testes de integração
│       ├── aerocore_ffi/       ← Camada FFI compatível com C
│       └── aerocore_ui/        ← Frontend egui + wgpu (planejado)
├── src/                        ← Área legada / rascunho
└── docs/                       ← Documentação do projeto
    ├── README.md               ← Índice de documentação
    ├── technical/              ← Docs para desenvolvedores / pesquisadores
    └── engineering/            ← Docs para clientes / usuários finais
```

---

## Início Rápido

**Requisitos:** [Toolchain Rust stable](https://rustup.rs/) (≥ 1.75)

```bash
# Clonar
git clone https://github.com/pedroPP-mecatronica/FlowVysAI.git
cd FlowVysAI

# Compilar (Windows ou Linux — mesmo comando)
cd aerocore
cargo build

# Rodar testes
cargo test

# Rodar benchmarks
cargo bench
```

> **Usuários Windows:** use PowerShell ou Git Bash. `aerocore/build.bat` está disponível como wrapper de conveniência.

---

## Como Executar

Atualmente o motor expõe uma **API de biblioteca** — ainda não há binário standalone.  
Para exercitar o módulo de I/O de malha:

```rust
use aerocore_core::io_mesh::stl::load_stl;

let mesh = load_stl(std::path::Path::new("meu_modelo.stl")).unwrap();
println!("Carregadas {} faces", mesh.num_faces());
```

Os casos de teste de integração (fluxo de Poiseuille, cavidade com tampa deslizante) estão em `aerocore/crates/aerocore_core/tests/`.  
Um executor de CLI e suporte a arquivo de configuração (TOML) são **planejados** para um sprint futuro.

---

## Roteiro

Veja [`docs/technical/solver-roadmap.md`](docs/technical/solver-roadmap.md) para o plano completo de sprints.

| Sprint | Foco |
|--------|------|
| S1 | Baseline 2D LBM D2Q9 + validação Poiseuille |
| S2 | I/O STL + pipeline de voxelização ✅ |
| S3 | Pipeline orientado a caso (configuração TOML + CLI) |
| S4 | Qualidade numérica e robustez |
| S5 | 3D LBM D3Q19 |
| S6 | Acoplamento térmico |
| S7 | Ambiente / efeitos de empuxo |
| S8 | Polimento, ajuste de performance, UI |

---

## CI & Testes

> **Planejado** — CI GitHub Actions (Windows + Linux) ainda não foi integrado.  
> Verificações esperadas assim que o CI estiver ativo:

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench (verificação de compilação)
```

Todos os testes de validação têm como alvo benchmarks clássicos de CFD com limites de erro quantitativos (norma L2 do perfil de velocidade, etc.).

---

## Como Contribuir

1. Fork → branch (`feat/minha-feature` ou `fix/meu-bug`).
2. Garanta que `cargo fmt` e `cargo clippy -D warnings` passem localmente.
3. Adicione ou atualize testes. Testes de integração ficam em `tests/`, testes unitários inline.
4. Abra um Pull Request com uma breve descrição do que muda e por quê.
5. Se estiver tomando uma decisão arquitetural significativa, inclua um ADR em `docs/adr/` (veja `Architecture_Manifest.md`).

Contribuições que melhoram a cobertura de validação, adicionam casos de benchmark ou ampliam o suporte a plataformas são especialmente bem-vindas.

---

## Documentação Aprofundada

| Público | Link |
|---------|------|
| Desenvolvedores / Pesquisadores | [`docs/technical/`](docs/technical/README.md) |
| Engenheiros / Usuários Finais | [`docs/engineering/`](docs/engineering/README.md) |
| Glossário CFD / LBM | [`docs/glossary.md`](docs/glossary.md) |
