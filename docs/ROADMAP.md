# FlowVysAI — Sprint Roadmap

> **Reading guide:** Each sprint is one logical unit of work.  
> A single PR per sprint is fine (PRs can be large as long as all items in the
> DoD checklist pass).  
> The learning progression is deliberately **2-D first → 3-D → thermal /
> environment** so students can build intuition before adding dimensions.

---

## Status

| Sprint | Title                             | Status     |
|--------|-----------------------------------|------------|
| S1     | Infrastructure & LBM Foundation   | ✅ Done (PR #3) |
| S2     | Binary STL Parser (mmap + nom)    | 🔄 In progress (PR #4) |
| S3     | LBM 2-D Core — D2Q9               | ⬜ Planned  |
| S4     | Boundary Conditions 2-D           | ⬜ Planned  |
| S5     | LBM 3-D Pipeline — D3Q19          | ⬜ Planned  |
| S6     | Performance & Benchmarks          | ⬜ Planned  |
| S7     | Thermal / Environment             | ⬜ Planned  |
| S8     | Productization & Academic Docs    | ⬜ Planned  |

---

## S1 — Infrastructure & LBM Foundation ✅

**Merged:** PR #3 (2026-03-25)

**Delivered:**
- `is_boundary` mask on `LbmSolver`; bounce-back per-cell in all `step_*` variants
- `io_mesh::voxelize::stl_to_boundary_mask` pipeline
- `--bench` flag printing MLUPS
- Clippy / fmt baseline

---

## S2 — Binary STL Parser (mmap + nom) 🔄

**Branch:** `sprint/S2-io-mesh-stl-mmap`  
**PR scope:** one large PR

**Goals:**
- Zero-copy binary STL parsing via `memmap2` + `nom` combinators
- ASCII STL fallback with `solid` prefix detection
- Criterion benchmark (throughput in MB/s and triangles/s)

**Suggested PR contents:**
- `aerocore_core` deps: `memmap2`, `nom`
- `src/io_mesh/stl.rs`: `load_stl` (ASCII) + `load_stl_mmap` (binary, zero-copy)
- `benches/stl_binary.rs`: Criterion group measuring parse throughput
- Unit tests with in-memory byte buffers (no file system required)

**Testing expectations:**
- Unit: parse a hard-coded 3-triangle binary blob
- Unit: parse a minimal ASCII STL string
- Integration: round-trip a known STL file via `tempfile`
- Windows: `Mmap` must be explicitly dropped before file cleanup in tests

**Outputs:** parsed `Vec<Triangle>` passed to voxeliser

---

## S3 — LBM 2-D Core — D2Q9 ⬜

**Why 2-D first?**  
A 2-D D2Q9 solver is ideal for learning: fields are easy to visualise, runs
finish in seconds on a laptop, and the physics are well-documented in textbooks.

**Goals:**
- New module `solvers::lbm::d2q9` (separate from existing D3Q19)
- Canonical validation cases included in the same PR:
  - **Couette flow** (moving top wall): analytical solution available
  - **Poiseuille 2-D channel**: analytical parabolic profile
  - **Lid-driven cavity 2-D**: qualitative comparison with Ghia et al. (1982)
- CLI entry-point: `cargo run -- --case cases/2d_lid.toml`
- Case format: TOML (see ADR 0001)
- Output: CSV files `ux.csv`, `uy.csv` (one row per lattice row)

**Suggested PR contents:**
- `src/solvers/lbm/d2q9.rs`: weights, velocities, `LbmSolver2D` struct,
  `collide`, `stream`, `apply_boundary`
- `src/solvers/lbm/d2q9/boundary.rs`: bounce-back, Zou/He velocity inlet/outlet
- `cases/2d_lid.toml`, `cases/2d_couette.toml`, `cases/2d_poiseuille.toml`
- `src/io_mesh/export.rs`: CSV exporter (cross-platform `\n`, UTF-8)
- `tests/d2q9_couette.rs`, `tests/d2q9_poiseuille.rs`, `tests/d2q9_lid.rs`

**Testing expectations:**
- Couette: relative error < 1 % vs analytical at mid-channel
- Poiseuille: relative error < 5 % (half-way bounce-back geometry shift)
- Lid cavity: vortex centre within ±2 lattice units of published result
- All tests pass on Windows CI

**Outputs:** `results/ux.csv`, `results/uy.csv` viewable in Excel / Python

---

## S4 — Boundary Conditions 2-D ⬜

**Goals:**
- Solidify and generalise boundary condition API introduced in S3
- Support complex 2-D geometries (rasterised obstacles from bitmap or STL slice)

**Suggested PR contents:**
- Zou/He pressure outlet
- Periodic boundaries (channel)
- `stl_slice_to_mask_2d`: slice a 3-D STL at a given Z to produce a 2-D mask
- Case `cases/2d_cylinder.toml`: flow around a circular cylinder; export `Cd`
- `tests/d2q9_cylinder.rs`: Strouhal number and drag coefficient sanity check

**Testing expectations:**
- Drag coefficient for cylinder at Re=100 within 10 % of published value
- Periodic channel: mass conservation to machine precision over 10 000 steps

**Outputs:** `results/cd.csv`, `results/field.csv`

---

## S5 — LBM 3-D Pipeline — D3Q19 ⬜

**Goals:**
- Make the existing D3Q19 solver production-ready by closing the full pipeline:
  STL → voxel mask → 3-D simulation → VTK export
- Case TOML drives the whole flow

**Suggested PR contents:**
- `src/io_mesh/vtk.rs`: legacy VTK ASCII exporter for velocity + pressure fields
  (viewable in ParaView and VisIt — both free and cross-platform)
- `cases/3d_sphere.toml`, `cases/3d_naca.toml` (sample STLs in `cases/stl/`)
- Integration test: load sphere STL → voxelise → 50 LBM steps → mass conserved
- Improve `stl_to_boundary_mask`: deterministic bounding-box padding, configurable
  voxel resolution

**Testing expectations:**
- Round-trip test: voxelise a known STL, run solver, check density sum is stable
- VTK output parseable by a simple Python script (included in `tools/`)
- Windows: VTK file uses `\n` line endings (not `\r\n`)

**Outputs:** `results/velocity.vtk`, `results/pressure.vtk` (open in ParaView)

---

## S6 — Performance & Benchmarks ⬜

**Goals:**
- Establish a reproducible performance baseline for 2-D and 3-D solvers
- Ensure the solver hot path is allocation-free

**Suggested PR contents:**
- `benches/lbm_2d.rs`: Criterion benchmark for D2Q9 at multiple domain sizes
- `benches/lbm_3d.rs`: Criterion benchmark for D3Q19 at multiple domain sizes
- `--bench` flag extended to print domain size, MLUPS, and memory footprint
- `#[allow(clippy::...)]` annotations replaced by genuine fixes where possible
- Guard: a unit test that fails if `LbmSolver::step` allocates (using a custom
  allocator or `assert_no_alloc` crate)

**Testing expectations:**
- Criterion HTML reports committed to `benches/results/` (gitignored in CI,
  optionally stored as artefact)
- MLUPS regression: new code must not be more than 5 % slower than previous
  PR on the same domain size

**Outputs:** `benches/results/` (local only), CI prints MLUPS to log

---

## S7 — Thermal / Environment ⬜

**Goals:**
- Add a coupled advection–diffusion solver for the temperature field
- Make environment parameters (ambient temperature, thermal conductivity,
  heat source) configurable via the case TOML

**Physics approach (no user choice required):**  
The temperature field `T(x,t)` is evolved with a passive-scalar LBM
advection–diffusion equation on the same lattice, coupled to the velocity field
from the main LBM step.  This is the standard approach in CFD textbooks and
avoids the complexity of a fully coupled double-distribution-function method
until it is needed.

**Suggested PR contents:**
- `src/solvers/lbm/thermal.rs`: `ThermalSolver` struct, `step_thermal`
- Case TOML extension:

  ```toml
  [thermal]
  enabled        = true
  T_ambient      = 293.15  # K
  conductivity   = 0.026   # W/(m·K) — air at 20 °C
  heat_source    = 0.0

  [material]
  name           = "aluminium"
  density        = 2700.0  # kg/m³
  specific_heat  = 900.0   # J/(kg·K)
  ```

- Export: `results/temperature.vtk` / `results/temperature.csv`
- Cases: `cases/2d_heated_plate.toml`, `cases/3d_convection.toml`
- Unit tests for `step_thermal` (conservation, monotone diffusion)

**Testing expectations:**
- 2-D heated plate: temperature profile converges to analytical steady state
  within 2 %
- All tests pass on Windows CI

**Outputs:** `results/temperature.vtk` (ParaView), `results/T_profile.csv`

---

## S8 — Productization & Academic Docs ⬜

**Goals:**
- Make FlowVysAI usable as a teaching and research platform out of the box
- Windows-first UX polish

**Suggested PR contents:**
- CLI subcommands: `run`, `bench`, `export`, `validate-case`
- `README.md` rewrite with:
  - "Getting Started on Windows" (step-by-step)
  - "Running your first 2-D case"
  - "Running your first 3-D case with a custom STL"
- `docs/GETTING_STARTED.md`: detailed walkthrough
- `docs/PHYSICS.md`: LBM theory primer (link to textbooks)
- Dataset of example cases in `cases/`:
  - `2d/lid_driven_cavity.toml`
  - `2d/flow_past_cylinder.toml`
  - `3d/flow_past_sphere.toml`
- `tools/plot_csv.py`: minimal Python script to plot CSV results (no
  installation beyond `matplotlib`)
- Release tag `v0.1.0-academic`

**Testing expectations:**
- `cargo run -- validate-case cases/2d/lid_driven_cavity.toml` exits 0 on
  Windows and Linux
- End-to-end smoke test: run 100 steps, check output files exist

---

## Long-term Epics (post-S8)

These are research-grade features planned after the academic baseline is solid:

| Epic | Description |
|------|-------------|
| **Turbulence** | LES / Smagorinsky subgrid model for high-Re flows |
| **Multi-body** | Wake interaction, multi-STL domain |
| **Ground effect** | Moving ground plane, wheel rotation |
| **Material database** | Thermal/mechanical properties for common aero materials |
| **FEM coupling** | Structural deformation under aerodynamic loads (FSI) |
| **GPU acceleration** | WGPU/CUDA backend for production-scale domains |
| **F1 / Motorsport cases** | Imola track environment, tyre heat, car wash |

---

*Last updated: 2026-03-25 — Sprints S1 (done) and S2 (in progress) based on PR #3 and PR #4.*
