# Validation and Trust

> **English** | [Português abaixo ↓](#validação-e-confiança)

---

## Table of Contents
- [What "Validation" Means in CFD](#what-validation-means-in-cfd)
- [Validation Strategy](#validation-strategy)
- [Current Validation Cases](#current-validation-cases)
- [Planned Validation Cases](#planned-validation-cases)
- [How to Interpret Results](#how-to-interpret-results)
- [Known Limitations and Assumptions](#known-limitations-and-assumptions)
- [Trust Criteria](#trust-criteria)

---

## What "Validation" Means in CFD

In computational fluid dynamics, **validation** is the process of comparing simulation results to known reference solutions. It is distinct from **verification** (does the code correctly solve the equations?) and **experimental comparison** (does the code match physical reality?).

AeroCore follows a three-level trust hierarchy:

```
Level 1 — Analytical Validation (highest confidence)
  Compare to exact mathematical solutions (Poiseuille, Couette, etc.)
  Error: < 1% (typically L2 norm < 1e-4)

Level 2 — Numerical Benchmark Validation (high confidence)
  Compare to widely cited reference numerical results
  e.g., Ghia et al. (1982) for lid-driven cavity
  Error: < 2% for key flow features

Level 3 — Qualitative Validation (lower confidence)
  Flow topology, vortex structures, physics "looks right"
  Used for complex flows without exact solutions
```

Currently, AeroCore targets **Level 1** (analytical) validation for the 2D solver.

---

## Validation Strategy

### Guiding Principles

1. **Every solver sprint ships with a validation case** — a simulation with a known, quantitative reference solution.
2. **Error metrics are reported explicitly** — not just "it looks OK" but "L2 error = 3.2e-5".
3. **Regression tests prevent regressions** — once a case passes, it must keep passing after refactoring.
4. **Refinement studies** — results should improve systematically as grid resolution increases (convergence study).

### Error Metrics Used

| Metric | Formula | Meaning |
|--------|---------|---------|
| L2 norm of velocity | `√(Σ(u_sim - u_exact)²) / N` | Average velocity error |
| L∞ norm | `max|u_sim - u_exact|` | Maximum point-wise error |
| Mass conservation error | `|Σρ_final - Σρ_initial| / Σρ_initial` | Fractional mass change |
| Strouhal number | `St = f·D/U` | Vortex shedding frequency |

---

## Current Validation Cases

### ✅ Poiseuille Flow (2D) — In Progress

**Setup:** Pressure-driven (or body-force-driven) flow between two infinite parallel plates.

**Analytical solution:**
```
u(y) = U_max · (1 - (2y/H)²)

where:
  H    = channel height
  U_max = maximum velocity at centerline
  y    = distance from centerline
```

**What is checked:**
- Velocity profile matches parabola at steady state.
- L2 norm error < 1e-4 at sufficient resolution (grid convergence study).
- Mass conservation error < 1e-10 per timestep.

**Relevance:** Validates the basic collision-streaming cycle, BGK relaxation, and no-slip boundary conditions.

---

### ✅ Lid-Driven Cavity (2D) — Planned (S4)

**Setup:** Square cavity with a moving top wall (velocity U_lid). All other walls no-slip. No net flow.

**Reference:** Ghia, Ghia, and Shin (1982) — tabulated velocity profiles along centerlines at Re = 100, 400, 1000.

**What is checked:**
- u-velocity along vertical centerline vs. Ghia table.
- v-velocity along horizontal centerline vs. Ghia table.
- Primary vortex center location.

**Relevance:** Validates boundary condition interactions, vortex structure, and solver accuracy at moderate Re numbers.

---

### 📋 Channel with Cylindrical Obstacle (2D) — Planned (S4)

**Setup:** Channel flow past a circular cylinder. At Re ≈ 100, produces periodic vortex shedding (Kármán vortex street).

**What is checked:**
- Strouhal number `St = f·D/U ≈ 0.165` (Re=100, classic result).
- Drag and lift coefficient oscillation.

**Relevance:** Validates unsteady solver behavior and force extraction.

---

### 📋 3D Poiseuille Flow — Planned (S5)

Extension of the 2D case to a rectangular duct. Analytical solution is a 2D parabolic profile in the yz cross-section.

---

### 📋 De Vahl Davis Cavity (Thermal) — Planned (S6)

**Setup:** Square cavity with differentially heated vertical walls (hot left wall, cold right wall). Buoyancy-driven natural convection.

**Reference:** De Vahl Davis (1983) — benchmark Nusselt numbers at Ra = 10³, 10⁴, 10⁵, 10⁶.

**Relevance:** Validates thermal coupling and buoyancy forces.

---

## How to Interpret Results

### VTK Field Files

When visualization is available, look for:

1. **Velocity field (arrows or color map):** Check that the flow direction and magnitude make physical sense.
2. **Pressure field (color contours):** Should decrease in flow direction for channel flows; should be higher on the leading edge of obstacles.
3. **Streamlines:** Should close into vortices for recirculation zones; should be smooth for laminar flows.

### Metrics CSV

| Column | Healthy Range | Warning Signs |
|--------|--------------|--------------|
| `residual` | Decreasing to < 1e-6 | Non-monotonic or diverging |
| `mass_error` | < 1e-10 per step | Growing monotonically → BC issue |
| `mlups` | Consistent | Drops → memory or CPU throttle |

### Common Simulation Artifacts

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| Velocity blows up | Mach > 0.3 or τ < 0.5 | Reduce inlet velocity or increase viscosity |
| Checkerboard pressure | Insufficient convergence | Run more timesteps |
| No-flow result | BC not applied | Check domain/BC configuration |
| Slow convergence | Low Re (high viscosity) | Expected; increase timesteps |

---

## Known Limitations and Assumptions

| Assumption | Implication |
|------------|-------------|
| **Incompressible flow** | Valid for Ma < 0.3. Do not use for supersonic or transonic flows. |
| **Laminar flow** | No turbulence model yet. Re must be below transition for quantitative accuracy. |
| **Newtonian fluid** | Constant viscosity. Non-Newtonian fluids (blood, polymer solutions) not supported. |
| **Single fluid phase** | No multiphase (liquid-gas, droplets) yet. |
| **No chemical reactions** | No combustion or reacting flow. |
| **Uniform grid** | No adaptive mesh refinement yet. |

---

## Trust Criteria

AeroCore results can be **trusted for quantitative engineering use** when:

- [ ] The simulation has converged (residual < target tolerance).
- [ ] Mass is conserved (mass error < 1e-8 per step).
- [ ] The flow conditions are in the validated regime (Re < transition, Ma < 0.3).
- [ ] A refinement study has been performed (results don't change significantly when resolution is doubled).
- [ ] The test suite passes for the solver version being used.

Results should be treated as **qualitative reference only** when:
- The flow regime is outside the validated range.
- Turbulence effects are significant (Re > ~2300 for pipe flow).
- 3D effects are important but a 2D simulation is used.

---

---

# Validação e Confiança

> [English above ↑](#validation-and-trust)

---

## O que "Validação" Significa em CFD

Em dinâmica dos fluidos computacional, **validação** é o processo de comparar resultados de simulação com soluções de referência conhecidas. É distinto de **verificação** (o código resolve corretamente as equações?) e **comparação experimental** (o código corresponde à realidade física?).

O AeroCore segue uma hierarquia de confiança em três níveis:

```
Nível 1 — Validação Analítica (maior confiança)
  Comparar com soluções matemáticas exatas (Poiseuille, Couette, etc.)
  Erro: < 1% (tipicamente norma L2 < 1e-4)

Nível 2 — Validação de Benchmark Numérico (alta confiança)
  Comparar com resultados numéricos de referência amplamente citados
  ex.: Ghia et al. (1982) para cavidade com tampa deslizante
  Erro: < 2% para características-chave do escoamento

Nível 3 — Validação Qualitativa (menor confiança)
  Topologia do escoamento, estruturas de vórtice, física "parece correta"
  Usado para escoamentos complexos sem soluções exatas
```

Atualmente, o AeroCore tem como alvo a validação de **Nível 1** (analítica) para o solver 2D.

---

## Casos de Validação Atuais

### ✅ Fluxo de Poiseuille (2D) — Em andamento

**Configuração:** Escoamento conduzido por pressão (ou força de corpo) entre duas placas paralelas infinitas.

**Solução analítica:**
```
u(y) = U_max · (1 - (2y/H)²)

onde:
  H     = altura do canal
  U_max = velocidade máxima na linha central
  y     = distância da linha central
```

**O que é verificado:**
- O perfil de velocidade corresponde à parábola no estado estacionário.
- Erro de norma L2 < 1e-4 em resolução suficiente.
- Erro de conservação de massa < 1e-10 por passo de tempo.

---

### 📋 Cavidade com Tampa Deslizante (2D) — Planejado (S4)

**Configuração:** Cavidade quadrada com parede superior em movimento (velocidade U_lid). Todas as outras paredes sem deslizamento.

**Referência:** Ghia, Ghia e Shin (1982) — perfis de velocidade tabelados ao longo das linhas centrais em Re = 100, 400, 1000.

---

### 📋 Canal com Obstáculo Cilíndrico (2D) — Planejado (S4)

**O que é verificado:**
- Número de Strouhal `St = f·D/U ≈ 0,165` (Re=100)
- Oscilação dos coeficientes de arrasto e sustentação

---

## Critérios de Confiança

Os resultados do AeroCore podem ser **confiáveis para uso quantitativo de engenharia** quando:

- [ ] A simulação convergiu (resíduo < tolerância alvo).
- [ ] A massa é conservada (erro de massa < 1e-8 por passo).
- [ ] As condições de escoamento estão no regime validado (Re < transição, Ma < 0,3).
- [ ] Um estudo de refinamento foi realizado.
- [ ] O conjunto de testes passa para a versão do solver sendo usada.

Os resultados devem ser tratados como **referência apenas qualitativa** quando:
- O regime de escoamento está fora do intervalo validado.
- Efeitos de turbulência são significativos (Re > ~2300 para escoamento em tubo).
- Efeitos 3D são importantes mas uma simulação 2D é usada.
