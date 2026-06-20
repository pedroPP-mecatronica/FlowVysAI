# Produto: FlowVysAI · AeroCore CFD Engine

FlowVysAI é um motor de Dinâmica dos Fluidos Computacional (CFD) de alto desempenho, de código aberto, escrito em **Rust**, projetado para fluxos de trabalho de simulação aeronáutica e automotiva.

## Funcionalidade Principal

- Solver LBM (Lattice Boltzmann Method) 2D (D2Q9) e 3D (D3Q19 — planejado)
- Parser e voxelização de malhas STL (ASCII + binário)
- Alocadores de memória de alta performance (arena, pool, alinhado)
- Backend de compute GPU via WGPU (planejado)
- Frontend egui + wgpu para visualização 3D (planejado)
- Camada FFI compatível com C para integração externa

## Casos de Uso

- Simulação aeronáutica e automotiva
- Validação contra benchmarks clássicos: Poiseuille, lid-driven cavity, canal com obstáculo
- Pesquisa e ensino de CFD / LBM com progressão didática 2D → 3D → térmico

## Plataformas Alvo

- **Windows** (alvo primário) e **Linux** (suporte de primeira classe)
- Sem dependências de SO diretas — tudo via crates cross-platform (`wgpu`, `egui`, `rfd`)

## Status Atual dos Módulos

| Módulo | Status |
|--------|--------|
| `io_mesh` — Parser STL (ASCII + Binário) | ✅ Implementado |
| `io_mesh` — Pipeline STL → voxel mask | ✅ Implementado |
| `io_mesh` — Formato CGNS | 📋 Planejado |
| `solvers/lbm` — D2Q9 skeleton | 🔄 Em andamento |
| `solvers/lbm` — D3Q19 | 📋 Planejado |
| `solvers/lbm` — Acoplamento térmico | 📋 Planejado |
| `solvers/navier_stokes` | 📋 Planejado |
| `gpu_compute` — Backend WGPU | 📋 Planejado |
| `memory` — Arena / Pool allocators | ✅ Implementado |
| CI — GitHub Actions (Windows + Linux) | 📋 Planejado |
| UI — Frontend egui / wgpu | 📋 Planejado |

## Roadmap de Sprints

| Sprint | Foco |
|--------|------|
| S1 | Baseline 2D LBM D2Q9 + validação Poiseuille |
| S2 | I/O STL + pipeline de voxelização ✅ |
| S3 | Pipeline orientado a caso (config TOML + CLI) |
| S4 | Qualidade numérica e robustez |
| S5 | 3D LBM D3Q19 |
| S6 | Acoplamento térmico |
| S7 | Ambiente / efeitos de empuxo |
| S8 | Polimento, performance tuning, UI |
