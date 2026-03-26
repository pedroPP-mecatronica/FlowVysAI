# Simulation Workflows

> **English** | [Português abaixo ↓](#fluxos-de-trabalho-de-simulação)

---

## Table of Contents
- [Overview](#overview)
- [Step 1 — Prepare Geometry](#step-1--prepare-geometry)
- [Step 2 — Configure Simulation](#step-2--configure-simulation)
- [Step 3 — Run Simulation](#step-3--run-simulation)
- [Step 4 — Post-process Results](#step-4--post-process-results)
- [Output File Structure](#output-file-structure)
- [Current Workflow Limitations](#current-workflow-limitations)

---

## Overview

The intended end-to-end simulation workflow is:

```
CAD Software (SolidWorks, FreeCAD, Blender, etc.)
     ↓  Export as .stl
STL File (surface mesh of your geometry)
     ↓  load_stl() — mmap-based reader
SoaMesh (triangles in world space)
     ↓  voxelize(mesh, resolution, domain_bounds)
VoxelMask (solid/fluid occupancy grid)
     ↓  configure domain + boundary conditions
LBM Solver Domain
     ↓  run N timesteps
Simulation Fields (velocity, pressure, temperature*)
     ↓  export
VTK / CSV output files
     ↓  open in ParaView / Python / MATLAB
Visualization & Post-processing
```

> \* Temperature field requires thermal coupling (Sprint S6, Planned)

---

## Step 1 — Prepare Geometry

### Supported Input
- **STL Binary** (`.stl`) — preferred for speed; any CAD package can export this.
- **STL ASCII** (`.stl`) — human-readable alternative; larger file size.
- CGNS: 📋 Planned

### Geometry Requirements
- The mesh must be a **closed, watertight surface** for correct voxelization.  
  Open meshes may lead to incorrect solid/fluid classification.
- Faces should be consistently oriented (outward normals).
- Units: the engine works in whatever unit system your STL is exported in — you must match the domain bounds and physical parameters to the same unit system.

### How to Export from Common CAD Tools
| Tool | Steps |
|------|-------|
| SolidWorks | File → Save As → STL (Binary) |
| FreeCAD | File → Export → STL Mesh |
| Blender | File → Export → STL |
| Meshmixer | File → Export → .STL |

### What the Engine Reads
```rust
// Current API (library mode)
use aerocore_core::io_mesh::stl::load_stl;

let mesh = load_stl(Path::new("geometry.stl"))?;
println!("Faces: {}", mesh.num_faces());
println!("Bounds: {:?}", mesh.info.bounding_box_min);
```

---

## Step 2 — Configure Simulation

> **Note:** This step currently requires writing Rust code or a configuration file.  
> A TOML-based configuration format and CLI runner are **planned** for Sprint S3.

### Parameters to Set
| Parameter | Description | Typical Values |
|-----------|-------------|---------------|
| Domain size (Nx, Ny, Nz) | Grid resolution in lattice units | 64–512 per axis |
| Fluid density ρ₀ | Reference density | 1.0 (LB units) |
| Kinematic viscosity ν | Fluid viscosity | From Re = U·L/ν |
| Inlet velocity | Inflow boundary condition | 0.01–0.1 (LB units, keep Ma < 0.3) |
| Number of timesteps | Simulation duration | 10,000–1,000,000 |
| Output interval | How often to write field snapshots | 100–1000 steps |

### Stability Constraints (Important!)
Lattice Boltzmann is conditionally stable. The following must be satisfied:

1. **Mach number:** `Ma = U / c_s < 0.3` (where `c_s = 1/√3` in LB units)  
   → Keep inlet velocity `U < 0.17` in lattice units.

2. **Relaxation time:** `τ > 0.5`  
   → `τ = 3ν + 0.5`; ensure `ν > 0`.

3. **Reynolds number:** `Re = U · L / ν`  
   → Higher Re needs finer resolution to resolve boundary layers.

---

## Step 3 — Run Simulation

### Current Method (library API)
```bash
cd aerocore
cargo test    # runs integration test cases
cargo bench   # runs performance benchmarks
```

### Planned Method (CLI, Sprint S3)
```bash
aerocore run case.toml
```

### What Happens During a Run
1. Mesh loaded and voxelized → solid/fluid mask created.
2. LBM distributions initialized to equilibrium at rest.
3. Each timestep:
   a. **Collision**: apply BGK operator to each fluid cell.
   b. **Streaming**: propagate distributions to neighbors.
   c. **Boundary conditions**: apply at walls, inlet, outlet.
   d. **Macroscopic variables**: compute ρ, u from distributions.
4. Every N steps: write field snapshot (VTK), log metrics (CSV).
5. End: write summary.

---

## Step 4 — Post-process Results

### Output Files (Planned format)
```
outputs/<case_name>/<timestamp>/
├── case.toml          ← Case configuration (for reproducibility)
├── metrics.csv        ← Per-step: step, time, MLUPS, residual, mass_error
├── fields_0000.vtk    ← Velocity and pressure at step 0
├── fields_0001.vtk
└── ...
```

### Visualization
- Open `.vtk` files in **[ParaView](https://www.paraview.org/)** (free, open-source).
- Use the "Stream Tracer" filter for streamlines.
- Use the "Contour" filter for pressure iso-surfaces.
- Export images or animations directly from ParaView.

### Metrics CSV Columns (Planned)
| Column | Description |
|--------|-------------|
| `step` | Timestep number |
| `time` | Simulated physical time (if units set) |
| `mlups` | Performance in Million Lattice Updates Per Second |
| `residual` | L2 norm of velocity change between steps |
| `mass_error` | Fractional change in total mass (conservation check) |

---

## Output File Structure

```
outputs/
└── poiseuille_re100/
    └── 2026-03-24T120000/
        ├── case.toml
        ├── metrics.csv
        ├── fields_0000.vtk
        ├── fields_0100.vtk
        └── fields_final.vtk
```

The timestamp folder ensures that re-running a case never overwrites previous results.

---

## Current Workflow Limitations

| Limitation | Planned Resolution |
|------------|-------------------|
| No CLI runner | Sprint S3 |
| No TOML case file | Sprint S3 |
| No VTK output yet | Sprint S3 / S5 |
| No GUI | Sprint S8 |
| 2D only | Sprint S5 (3D) |
| No thermal output | Sprint S6 |
| Windows: no installer | Sprint S8 |

---

---

# Fluxos de Trabalho de Simulação

> [English above ↑](#simulation-workflows)

---

## Visão Geral

O fluxo de trabalho de simulação de ponta a ponta pretendido é:

```
Software CAD (SolidWorks, FreeCAD, Blender, etc.)
     ↓  Exportar como .stl
Arquivo STL (malha de superfície da sua geometria)
     ↓  load_stl() — leitor baseado em mmap
SoaMesh (triângulos no espaço do mundo)
     ↓  voxelize(mesh, resolução, limites_do_domínio)
VoxelMask (grade de ocupação sólido/fluido)
     ↓  configurar domínio + condições de contorno
Domínio do Solver LBM
     ↓  executar N passos de tempo
Campos de Simulação (velocidade, pressão, temperatura*)
     ↓  exportar
Arquivos de saída VTK / CSV
     ↓  abrir no ParaView / Python / MATLAB
Visualização e Pós-processamento
```

> \* Campo de temperatura requer acoplamento térmico (Sprint S6, Planejado)

---

## Etapa 1 — Preparar Geometria

### Entrada Suportada
- **STL Binário** (`.stl`) — preferido pela velocidade; qualquer pacote CAD pode exportar isso.
- **STL ASCII** (`.stl`) — alternativa legível por humanos; arquivo maior.
- CGNS: 📋 Planejado

### Requisitos de Geometria
- A malha deve ser uma **superfície fechada e estanque** para voxelização correta.
- As faces devem ser orientadas consistentemente (normais para fora).
- Unidades: o motor trabalha em qualquer sistema de unidades que seu STL usa — você deve corresponder os limites do domínio e parâmetros físicos ao mesmo sistema de unidades.

### Como Exportar de Ferramentas CAD Comuns
| Ferramenta | Passos |
|------------|--------|
| SolidWorks | Arquivo → Salvar Como → STL (Binário) |
| FreeCAD | Arquivo → Exportar → Malha STL |
| Blender | Arquivo → Exportar → STL |

---

## Etapa 2 — Configurar Simulação

> **Nota:** Esta etapa atualmente requer escrever código Rust ou um arquivo de configuração.  
> Um formato de configuração baseado em TOML e executor CLI são **planejados** para o Sprint S3.

### Restrições de Estabilidade (Importante!)
O Lattice Boltzmann é condicionalmente estável. O seguinte deve ser satisfeito:

1. **Número de Mach:** `Ma = U / c_s < 0,3` (onde `c_s = 1/√3` em unidades LB)  
   → Mantenha a velocidade de entrada `U < 0,17` em unidades de rede.

2. **Tempo de relaxação:** `τ > 0,5`  
   → `τ = 3ν + 0,5`; garanta `ν > 0`.

3. **Número de Reynolds:** `Re = U · L / ν`  
   → Re mais alto precisa de resolução mais fina para resolver camadas limite.

---

## Etapa 4 — Pós-processar Resultados

### Visualização
- Abra arquivos `.vtk` no **[ParaView](https://www.paraview.org/)** (gratuito, código aberto).
- Use o filtro "Stream Tracer" para linhas de corrente.
- Use o filtro "Contour" para iso-superfícies de pressão.
- Exporte imagens ou animações diretamente do ParaView.

---

## Limitações Atuais do Fluxo de Trabalho

| Limitação | Resolução Planejada |
|-----------|---------------------|
| Sem executor CLI | Sprint S3 |
| Sem arquivo de caso TOML | Sprint S3 |
| Sem saída VTK ainda | Sprint S3 / S5 |
| Sem GUI | Sprint S8 |
| Apenas 2D | Sprint S5 (3D) |
| Sem saída térmica | Sprint S6 |
