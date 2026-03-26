# Solver Roadmap

> **English** | [Português abaixo ↓](#roteiro-do-solver)

---

## Table of Contents
- [Learning Progression](#learning-progression)
- [Sprint Overview](#sprint-overview)
- [Sprint Details](#sprint-details)
- [Validation Cases](#validation-cases)
- [Definition of Done](#definition-of-done)

---

## Learning Progression

AeroCore follows a deliberate **didactic progression** from simpler to more complex physics:

```
2D Lattice Boltzmann (D2Q9)
        ↓
3D Lattice Boltzmann (D3Q19)
        ↓
Thermal / Energy Coupling
        ↓
Environmental Effects (buoyancy, natural convection)
```

Each phase builds on the previous, allowing numerical validation of increasingly complex physics before adding new degrees of freedom.

---

## Sprint Overview

| Sprint | Focus | Status |
|--------|-------|--------|
| S1 | 2D LBM D2Q9 baseline + Poiseuille validation | 🔄 In Progress |
| S2 | STL I/O + voxelization pipeline | ✅ Done |
| S3 | Case-driven pipeline (TOML config + CLI runner) | 📋 Planned |
| S4 | Numerical quality & robustness | 📋 Planned |
| S5 | 3D LBM D3Q19 | 📋 Planned |
| S6 | Thermal coupling (energy equation) | 📋 Planned |
| S7 | Environmental effects (buoyancy, Boussinesq) | 📋 Planned |
| S8 | Polish, performance tuning, UI | 📋 Planned |

---

## Sprint Details

### Sprint S1 — 2D LBM D2Q9 Baseline 🔄 In Progress

**Goal:** Minimal working 2D solver with quantitative validation.

**Deliverables:**
- D2Q9 velocity set (9 discrete velocities, 2D)
- BGK (Bhatnagar-Gross-Krook) single-relaxation-time collision operator
- Streaming step
- Boundary conditions:
  - No-slip walls (standard bounce-back)
  - Inlet: fixed velocity (Zou/He or simple forced inlet)
  - Outlet: open boundary / zero-gradient
- Poiseuille flow validation: analytical parabolic profile vs. simulated — L2 error < 1e-4
- Performance metric: MLUPS (Million Lattice Updates Per Second)
- Criterion benchmark for the simulation loop

**Definition of Done:** `cargo test` passes with L2 error < 1e-4; MLUPS reported; docs updated.

---

### Sprint S2 — STL I/O + Voxelization ✅ Done

**Goal:** Geometry enters the simulation pipeline with quality and performance.

**Deliverables:**
- STL binary parser with `nom` + `memmap2` ✅
- STL ASCII parser (state machine) ✅
- Format auto-detection ✅
- `SoaMesh` data structure ✅
- STL → voxel mask pipeline ✅
- `stl_bench` Criterion benchmark ✅
- Integration tests (bbox, triangle count, round-trip) ✅

---

### Sprint S3 — Case-Driven Pipeline 📋 Planned

**Goal:** Run simulations from a configuration file without recompiling.

**Deliverables:**
- TOML case format (domain, resolution, fluid properties, BC specs)
- `serde` deserialization of case files
- CLI runner: `aerocore run case.toml`
- Structured output layout:
  ```
  outputs/<case_name>/<timestamp>/
    case.toml          ← copied for reproducibility
    metrics.csv        ← step, time, MLUPS, residual, mass_error
    fields_0000.vtk
    fields_0001.vtk
    ...
  ```
- Stability pre-checks: Mach number, relaxation time τ, Re number

---

### Sprint S4 — Numerical Quality & Robustness 📋 Planned

**Goal:** Improve solver accuracy and stability for practical engineering use.

**Deliverables:**
- Zou/He velocity boundary conditions (more accurate than simple bounce-back for inlet/outlet)
- Outflow stability improvement (convective outflow or sponge layer)
- Regression test suite: compare outputs to reference with configurable tolerance
- Lid-driven cavity validation at Re = 100, 400, 1000 (compare with Ghia et al. 1982)
- Channel with cylindrical obstacle (vortex shedding, Strouhal number)

---

### Sprint S5 — 3D LBM D3Q19 📋 Planned

**Goal:** Port the solver core to 3D while maintaining all tests and performance characteristics.

**Deliverables:**
- D3Q19 velocity set (19 discrete velocities, 3D)
- 3D streaming and collision kernels
- 3D boundary conditions (6 faces: ±x, ±y, ±z)
- 3D Poiseuille flow (analytical solution: parabolic in 2 cross-sectional directions)
- 3D lid-driven cavity
- VTK output for 3D fields (ParaView-compatible)
- Performance benchmark: MLUPS in 3D on Windows and Linux

---

### Sprint S6 — Thermal Coupling 📋 Planned

**Goal:** Add energy transport to the simulation.

**Deliverables:**
- Double-distribution-function (DDF) approach: one distribution for mass/momentum (D2Q9/D3Q19), one for energy/temperature
- BGK collision for temperature distribution
- Advection-diffusion temperature field
- Validation: heated channel (analytical Nusselt number comparison)
- Validation: natural convection in a differentially heated cavity (De Vahl Davis benchmark)

---

### Sprint S7 — Environmental / Buoyancy Effects 📋 Planned

**Goal:** Simulate thermally driven flows and environmental scenarios.

**Deliverables:**
- Boussinesq approximation: buoyancy force `F = ρ·g·β·(T - T_ref)`
- Gravity body force in streaming step
- Benchmark: thermal plume rising in still fluid
- Benchmark: simplified room ventilation (inlet + heated floor/wall)

---

### Sprint S8 — Polish & Product 📋 Planned

**Goal:** Production-quality codebase ready for external users.

**Deliverables:**
- CI fully green on Windows + Linux (GitHub Actions)
- egui + wgpu UI: load STL, configure case, run, visualize fields
- Performance tuning: AVX2/AVX-512 SIMD in hot kernels
- Public API documentation (`cargo doc`)
- Example cases with expected outputs checked into `examples/`
- Release tagging and changelog

---

## Validation Cases

| Case | Dimension | Analytical Solution | Sprint |
|------|-----------|-------------------|--------|
| Poiseuille flow | 2D | Parabolic velocity profile | S1 |
| Lid-driven cavity | 2D | Ghia et al. (1982) | S4 |
| Channel with cylinder | 2D | Strouhal ≈ 0.20 (Re=100) | S4 |
| Poiseuille flow | 3D | Parabolic in 2D cross-section | S5 |
| Heated channel | 2D/3D | Nusselt number (analytical) | S6 |
| De Vahl Davis cavity | 2D | Benchmark data table | S6 |
| Thermal plume | 2D/3D | Qualitative + Nusselt | S7 |

---

## Definition of Done

For any sprint to be considered complete:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes with quantitative error bounds (where applicable)
- [ ] `cargo bench` compiles and runs (no panics)
- [ ] Documentation updated (this file or a new technical doc)
- [ ] CI green on Windows and Linux (once CI is merged)

---

---

# Roteiro do Solver

> [English above ↑](#solver-roadmap)

---

## Progressão de Aprendizado

O AeroCore segue uma **progressão didática** deliberada de física mais simples para mais complexa:

```
Lattice Boltzmann 2D (D2Q9)
        ↓
Lattice Boltzmann 3D (D3Q19)
        ↓
Acoplamento Térmico / Energia
        ↓
Efeitos Ambientais (empuxo, convecção natural)
```

---

## Visão Geral dos Sprints

| Sprint | Foco | Status |
|--------|------|--------|
| S1 | Baseline 2D LBM D2Q9 + validação Poiseuille | 🔄 Em andamento |
| S2 | I/O STL + pipeline de voxelização | ✅ Concluído |
| S3 | Pipeline orientado a caso (configuração TOML + CLI) | 📋 Planejado |
| S4 | Qualidade numérica e robustez | 📋 Planejado |
| S5 | 3D LBM D3Q19 | 📋 Planejado |
| S6 | Acoplamento térmico (equação de energia) | 📋 Planejado |
| S7 | Efeitos ambientais (empuxo, Boussinesq) | 📋 Planejado |
| S8 | Polimento, ajuste de performance, UI | 📋 Planejado |

---

## Detalhes dos Sprints

### Sprint S1 — Baseline 2D LBM D2Q9 🔄 Em andamento

**Objetivo:** Solver 2D mínimo funcionando com validação quantitativa.

**Entregáveis:**
- Conjunto de velocidades D2Q9 (9 velocidades discretas, 2D)
- Operador de colisão BGK (Bhatnagar-Gross-Krook) com tempo de relaxação único
- Etapa de streaming
- Condições de contorno:
  - Paredes sem-deslizamento (bounce-back padrão)
  - Entrada: velocidade fixa
  - Saída: contorno aberto / gradiente zero
- Validação de fluxo de Poiseuille: perfil parabólico analítico vs. simulado — erro L2 < 1e-4
- Métrica de performance: MLUPS (Milhões de Atualizações de Rede por Segundo)

### Sprint S5 — 3D LBM D3Q19 📋 Planejado

**Objetivo:** Portar o núcleo do solver para 3D mantendo todos os testes e características de performance.

**Entregáveis:**
- Conjunto de velocidades D3Q19 (19 velocidades discretas, 3D)
- Kernels de streaming e colisão 3D
- Condições de contorno 3D (6 faces: ±x, ±y, ±z)
- Fluxo de Poiseuille 3D (solução analítica: parabólica em 2 direções da seção transversal)
- Saída VTK para campos 3D (compatível com ParaView)

### Sprint S6 — Acoplamento Térmico 📋 Planejado

**Objetivo:** Adicionar transporte de energia à simulação.

**Entregáveis:**
- Abordagem de função de distribuição dupla (DDF)
- Colisão BGK para distribuição de temperatura
- Campo de temperatura advecção-difusão
- Validação: canal aquecido (comparação analítica de número de Nusselt)

---

## Casos de Validação

| Caso | Dimensão | Solução Analítica | Sprint |
|------|----------|-------------------|--------|
| Fluxo de Poiseuille | 2D | Perfil de velocidade parabólico | S1 |
| Cavidade com tampa deslizante | 2D | Ghia et al. (1982) | S4 |
| Canal com cilindro | 2D | Strouhal ≈ 0,20 (Re=100) | S4 |
| Fluxo de Poiseuille | 3D | Parabólico em seção transversal 2D | S5 |
| Canal aquecido | 2D/3D | Número de Nusselt (analítico) | S6 |
| Cavidade De Vahl Davis | 2D | Tabela de dados de benchmark | S6 |

---

## Definição de Pronto

Para qualquer sprint ser considerado completo:

- [ ] `cargo fmt --check` passa
- [ ] `cargo clippy -- -D warnings` passa
- [ ] `cargo test` passa com limites de erro quantitativos (quando aplicável)
- [ ] `cargo bench` compila e executa (sem panics)
- [ ] Documentação atualizada
- [ ] CI verde no Windows e Linux (assim que o CI for integrado)
