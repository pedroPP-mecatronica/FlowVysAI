# Product Vision

> **English** | [Português abaixo ↓](#visão-do-produto)

---

## Table of Contents
- [What is FlowVysAI?](#what-is-flowvysai)
- [Intended Use Cases](#intended-use-cases)
- [Current Capabilities](#current-capabilities)
- [Planned Capabilities](#planned-capabilities)
- [Honest Limitations](#honest-limitations)
- [Technology Choice](#technology-choice)

---

## What is FlowVysAI?

**FlowVysAI** (also known as the **AeroCore CFD Engine**) is an open-source computational fluid dynamics (CFD) simulation engine. It is designed to help engineers and researchers model and analyze fluid flow around and through physical geometries — from simple canonical cases to complex aerodynamic shapes.

The engine is built on the **Lattice Boltzmann Method (LBM)** — a modern, inherently parallel CFD approach that is well suited for:

- Complex geometries (imported as standard STL files from CAD software)
- Unsteady, vortex-shedding flows
- Thermal and multi-physics coupling
- High performance on both CPUs and GPUs

**The engine is written in Rust** — chosen for guaranteed memory safety, zero-cost performance abstractions, and first-class cross-platform support (Windows + Linux).

---

## Intended Use Cases

| Use Case | Status |
|----------|--------|
| External aerodynamics around a body (car, aircraft, building) | 📋 Planned (S5+) |
| Internal flow in channels and ducts | 🔄 In Progress (S1, 2D) |
| Thermal analysis (heated walls, natural convection) | 📋 Planned (S6) |
| Vortex shedding and wake characterization | 📋 Planned (S4) |
| Ventilation and indoor air quality (simplified) | 📋 Planned (S7) |
| Drag and lift coefficient estimation | 📋 Planned (S5+) |
| Educational / benchmark CFD cases | ✅ Available (Poiseuille, lid-driven cavity) |

---

## Current Capabilities

### ✅ Geometry Processing
- Load STL files (ASCII and binary) — any standard CAD export.
- Compute bounding box and characteristic length automatically.
- Convert triangulated surface meshes into 3D voxel occupancy grids (for use as solid obstacles in the LBM domain).
- Memory-mapped file I/O: capable of handling very large geometry files (> 1 GB) without exhausting RAM.

### 🔄 2D Flow Solver (In Progress)
- Lattice Boltzmann D2Q9 (9 velocity directions, 2D)
- BGK single-relaxation-time collision model
- Boundary conditions: no-slip walls, inlet, outlet
- Validation case: Poiseuille flow (parabolic velocity profile)
- Performance metric: MLUPS (Million Lattice Updates Per Second)

### ✅ Infrastructure
- Cross-platform build (Windows + Linux via `cargo`)
- Criterion benchmarks for I/O and solver performance
- Integration test suite with quantitative validation
- Arena and pool memory allocators for zero-allocation simulation loops

---

## Planned Capabilities

### 📋 3D Solver (Sprint S5)
Full three-dimensional simulation with the D3Q19 velocity set. Enables:
- Flow around 3D geometries imported from STL
- 3D channel flows and duct flows
- VTK output compatible with ParaView for 3D visualization

### 📋 Thermal Coupling (Sprint S6)
Adds energy transport to the simulation:
- Temperature field alongside velocity/pressure
- Heated wall boundary conditions
- Validation: heated channel, differentially heated cavity (De Vahl Davis benchmark)

### 📋 Environmental / Buoyancy Effects (Sprint S7)
- Boussinesq approximation for buoyancy-driven flows
- Gravity body force
- Applications: thermal plumes, room ventilation, outdoor air movement

### 📋 Graphical User Interface (Sprint S8)
- Interactive case setup: load STL, configure domain, set boundary conditions
- Real-time visualization of velocity and pressure fields
- Result export to VTK, CSV

### 📋 GPU Acceleration
- WGPU-based compute backend (Vulkan/DirectX 12/Metal)
- Optional CUDA backend for NVIDIA GPUs

---

## Honest Limitations

The following are **current limitations** that potential users should be aware of:

| Limitation | Details |
|------------|---------|
| **No compressible flow yet** | The LBM implementation targets incompressible low-Mach flows. Compressible Navier-Stokes is planned but not implemented. |
| **No turbulence model yet** | Current implementation is laminar. LES or RANS turbulence models are planned for a future sprint. |
| **2D only (currently)** | The 3D solver (D3Q19) is planned for Sprint S5 and not yet available. |
| **No GUI yet** | The graphical frontend is planned for Sprint S8. Currently, interaction is via code and configuration files. |
| **No CGNS support yet** | Only STL is supported for geometry import. CGNS (industry standard for structured meshes) is planned. |
| **No parallel/distributed computation yet** | Current solver runs on a single CPU core (multi-core with `rayon` is planned). |
| **No experimental validation yet** | All validation is against analytical solutions and published numerical benchmarks. Wind tunnel data comparison is a future goal. |

---

## Technology Choice

| Criterion | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | Rust | Memory safety without GC; zero-cost abstractions; cross-platform native |
| **Simulation method** | Lattice Boltzmann (LBM) | Inherently parallel; handles complex geometries naturally; well-suited for unsteady flows |
| **Memory layout** | Structure-of-Arrays (SoA) | SIMD-friendly; GPU-upload-ready; cache-efficient |
| **File format (geometry)** | STL | Universal CAD export; simple binary format |
| **Build system** | Cargo (Rust) | Single command cross-platform build; reproducible dependency resolution |

---

---

# Visão do Produto

> [English above ↑](#product-vision)

---

## O que é o FlowVysAI?

**FlowVysAI** (também conhecido como **AeroCore CFD Engine**) é um motor de simulação de dinâmica dos fluidos computacional (CFD) de código aberto. Ele foi projetado para ajudar engenheiros e pesquisadores a modelar e analisar o escoamento de fluidos ao redor e através de geometrias físicas — de casos canônicos simples a formas aerodinâmicas complexas.

O motor é construído sobre o **Método de Lattice Boltzmann (LBM)** — uma abordagem CFD moderna e inerentemente paralela que é bem adequada para:

- Geometrias complexas (importadas como arquivos STL padrão de software CAD)
- Escoamentos instacionários com desprendimento de vórtices
- Acoplamento térmico e multi-física
- Alto desempenho em CPUs e GPUs

**O motor é escrito em Rust** — escolhido pela segurança de memória garantida, abstrações de performance de custo zero e suporte multiplataforma de primeira classe (Windows + Linux).

---

## Casos de Uso Pretendidos

| Caso de Uso | Status |
|-------------|--------|
| Aerodinâmica externa ao redor de um corpo (carro, aeronave, edifício) | 📋 Planejado (S5+) |
| Escoamento interno em canais e dutos | 🔄 Em andamento (S1, 2D) |
| Análise térmica (paredes aquecidas, convecção natural) | 📋 Planejado (S6) |
| Desprendimento de vórtices e caracterização de esteira | 📋 Planejado (S4) |
| Ventilação e qualidade do ar interior (simplificado) | 📋 Planejado (S7) |
| Estimativa de coeficientes de arrasto e sustentação | 📋 Planejado (S5+) |
| Casos CFD educacionais / de benchmark | ✅ Disponível (Poiseuille, cavidade com tampa deslizante) |

---

## Capacidades Atuais

### ✅ Processamento de Geometria
- Carregar arquivos STL (ASCII e binário) — qualquer exportação CAD padrão.
- Calcular bounding box e comprimento característico automaticamente.
- Converter malhas de superfície trianguladas em grades de ocupação de voxel 3D.
- I/O de arquivo mapeado em memória: capaz de lidar com arquivos de geometria muito grandes (> 1 GB) sem esgotar a RAM.

### 🔄 Solver de Fluxo 2D (Em andamento)
- Lattice Boltzmann D2Q9 (9 direções de velocidade, 2D)
- Modelo de colisão BGK com tempo de relaxação único
- Condições de contorno: paredes sem-deslizamento, entrada, saída
- Caso de validação: fluxo de Poiseuille (perfil de velocidade parabólico)

---

## Limitações Honestas

| Limitação | Detalhes |
|-----------|----------|
| **Sem escoamento compressível ainda** | A implementação LBM visa escoamentos incompressíveis de baixo número de Mach. Navier-Stokes compressível está planejado. |
| **Sem modelo de turbulência ainda** | A implementação atual é laminar. Modelos de turbulência LES ou RANS estão planejados. |
| **Apenas 2D (atualmente)** | O solver 3D (D3Q19) está planejado para o Sprint S5. |
| **Sem GUI ainda** | O frontend gráfico está planejado para o Sprint S8. |
| **Sem suporte CGNS ainda** | Apenas STL é suportado para importação de geometria. |
| **Sem validação experimental ainda** | Toda validação é feita contra soluções analíticas e benchmarks numéricos publicados. |
