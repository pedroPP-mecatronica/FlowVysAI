# Glossary — CFD / LBM Terminology

> **English** | [Português abaixo ↓](#glossário--terminologia-cfd--lbm)

---

## CFD & LBM Glossary (English)

### A

**Advection**  
Transport of a quantity (e.g., temperature, tracer) by the bulk flow velocity. One of the two terms in the advection-diffusion equation.

**AoS (Array of Structs)**  
Memory layout where each element stores all its properties together: `[{x,y,z}, {x,y,z}, ...]`. Contrast with SoA.

### B

**BGK (Bhatnagar-Gross-Krook)**  
The simplest LBM collision model. Relaxes the distribution function toward local equilibrium at a single rate `1/τ`. Named after its 1954 inventors.

**Bounding Box (AABB)**  
The smallest axis-aligned rectangular box that contains the entire geometry. Used to define the simulation domain extent.

**Bounce-Back Boundary Condition**  
A boundary condition in LBM that reflects incoming distribution functions back in the direction they came from. Enforces no-slip (zero-velocity) at solid walls. Simple and robust.

**Boussinesq Approximation**  
An approximation for buoyancy-driven flows: treat the fluid as incompressible except in the buoyancy term, where density variation due to temperature is retained. Simplifies the thermal solver significantly.

### C

**CFD (Computational Fluid Dynamics)**  
Numerical simulation of fluid flows governed by the Navier-Stokes (or equivalent) equations. Uses discretized domains and time-stepping to approximate the flow solution.

**CGNS (CFD General Notation System)**  
An industry-standard file format for storing structured and unstructured computational grids and associated flow-field data.

**Characteristic Length**  
A representative length scale of the geometry or flow, used to compute dimensionless numbers like the Reynolds number. For a channel, typically the hydraulic diameter; for a cylinder, its diameter.

**Collision Step**  
In LBM: the local redistribution of probability distribution functions to relax toward thermodynamic equilibrium. Does not involve data exchange between cells — fully local.

**Convergence**  
The property that the simulation residual (difference between successive timesteps) decreases toward zero as the simulation reaches a steady state.

### D

**D2Q9**  
A 2D Lattice Boltzmann velocity set with 9 discrete velocities: one at rest, 4 face-adjacent, and 4 corner-diagonal. Commonly used for incompressible 2D flows.

**D3Q19**  
A 3D Lattice Boltzmann velocity set with 19 discrete velocities: one at rest, 6 face-adjacent, and 12 edge-adjacent. The standard choice for 3D incompressible flows.

**DDF (Double Distribution Function)**  
LBM approach for thermal flows: two separate distribution functions — one for mass/momentum (D2Q9 or D3Q19), one for energy/temperature.

**Diffusion**  
Spreading of a quantity (momentum, heat, species) due to molecular effects, independent of bulk flow. Characterized by the diffusion coefficient.

**Drag Coefficient (Cd)**  
Dimensionless measure of the drag force on a body: `Cd = 2·F_drag / (ρ·U²·A)`. Used to compare aerodynamic performance across different scales.

### E

**Equilibrium Distribution**  
In LBM, the Maxwell-Boltzmann distribution expanded to second order in velocity. The collision step relaxes the actual distribution toward this. Depends only on local density ρ and velocity **u**.

### F

**f (distribution function)**  
The fundamental variable in LBM. `f_i(x, t)` represents the probability density of finding particles at position **x** and time **t** moving in direction **i**. From these, all macroscopic quantities (ρ, **u**, T) are computed as moments.

### G

**Grid Refinement Study (Convergence Study)**  
Running the same simulation at increasing resolution (Nx×2, Nx×4, etc.) to demonstrate that results converge to a limiting solution. Provides confidence in the mesh independence of results.

### L

**Lattice Boltzmann Method (LBM)**  
A mesoscopic simulation technique for fluid dynamics. Evolves probability distribution functions on a regular lattice according to a simplified Boltzmann equation. Naturally parallel, handles complex geometries via voxelization.

**Lift Coefficient (Cl)**  
Dimensionless measure of lift force: `Cl = 2·F_lift / (ρ·U²·A)`.

**L2 Norm**  
`‖e‖₂ = √(Σ eᵢ²/N)`. The root-mean-square error, commonly used to measure how well a simulated field matches a reference.

**L∞ Norm (Max norm)**  
`‖e‖∞ = max|eᵢ|`. The maximum pointwise error across the domain.

### M

**Mach Number (Ma)**  
`Ma = U / c_s` where `c_s` is the speed of sound. LBM is valid for `Ma < 0.3` (weakly compressible regime). Exceeding this leads to compressibility errors.

**MLUPS (Million Lattice Updates Per Second)**  
Performance metric for LBM solvers: the number of lattice cell updates (collision + streaming) performed per second. Higher = faster simulation.

**Mmap (Memory-Mapped I/O)**  
Operating system feature that maps a file directly into virtual address space, allowing the OS to load pages lazily from disk. Efficient for large files.

### N

**No-Slip Boundary Condition**  
Fluid velocity equals the wall velocity at a solid boundary. For stationary walls: zero velocity. Enforced in LBM via bounce-back.

**Nusselt Number (Nu)**  
Dimensionless heat transfer coefficient: `Nu = h·L/k`. Relates convective to conductive heat transfer at a surface. Used to validate thermal simulations.

**Navier-Stokes Equations**  
The fundamental equations of fluid motion, expressing conservation of mass and momentum (and optionally energy). LBM recovers the incompressible Navier-Stokes equations in the limit of small Ma and low Kn.

### P

**Poiseuille Flow**  
Steady, fully developed, pressure-driven flow between parallel plates or in a pipe. Has an exact analytical solution (parabolic velocity profile). Used as a primary validation case.

**Pressure (in LBM)**  
In incompressible LBM, pressure `p = ρ · c_s²` where `c_s = 1/√3` in lattice units. Not an independent variable — derived from the density distribution.

### R

**Rayleigh Number (Ra)**  
`Ra = g·β·ΔT·L³ / (ν·α)`. Governs natural convection; ratio of buoyancy to diffusive forces. Used to validate thermal cavity simulations.

**Reynolds Number (Re)**  
`Re = U·L / ν`. Ratio of inertial to viscous forces. Determines the flow regime: laminar (low Re) vs. turbulent (high Re).

**Residual**  
A measure of how much the solution is changing between successive timesteps. When the residual is small, the simulation has converged.

### S

**SoA (Structure of Arrays)**  
Memory layout where each property is stored in its own contiguous array: `x: [x0, x1, ...], y: [y0, y1, ...]`. Contrast with AoS. Preferred in AeroCore for SIMD efficiency.

**SoaMesh**  
AeroCore's mesh data structure, storing vertices and normals in SoA layout. See [technical/mesh-io.md](technical/mesh-io.md).

**Streaming Step**  
In LBM: propagation of distribution functions from each cell to its neighbors in the corresponding velocity direction. Involves data exchange between cells — the communication step.

**STL (STereoLithography)**  
A triangulated surface mesh file format, widely supported by CAD tools. Stores triangle vertices and normals. Used in AeroCore for geometry import.

**Strouhal Number (St)**  
`St = f·D/U`. Dimensionless vortex shedding frequency. Used to validate unsteady wake simulations past cylinders.

### T

**τ (Relaxation Time)**  
The BGK collision model parameter. Controls how fast the distribution relaxes to equilibrium. Related to viscosity: `ν = c_s²·(τ - 0.5)`. Must satisfy `τ > 0.5` for stability.

### V

**Validation**  
Comparison of simulation results to known reference solutions (analytical or benchmark) to establish confidence in the solver's accuracy.

**Verification**  
Demonstration that the numerical implementation correctly solves the intended equations (coding correctness), separate from physical accuracy.

**Viscosity (ν — kinematic)**  
`ν = μ/ρ`. Resistance of a fluid to shearing deformation. In LBM lattice units: `ν = c_s²·(τ - 0.5)`.

**VTK (Visualization Toolkit)**  
An open-source file format for scientific visualization, supported by ParaView and many other tools. AeroCore plans to output simulation fields in VTK format.

**Voxelization**  
The process of converting a triangulated surface mesh (STL) into a 3D grid of binary occupancy values (solid=1, fluid=0). AeroCore uses this to define solid boundaries for LBM.

---

---

# Glossário — Terminologia CFD / LBM

> [English above ↑](#glossary--cfd--lbm-terminology)

---

### A

**Advecção (Advection)**  
Transporte de uma grandeza (ex.: temperatura, traçador) pela velocidade do escoamento em massa. Um dos dois termos da equação de advecção-difusão.

**AoS (Array of Structs)**  
Layout de memória onde cada elemento armazena todas as suas propriedades juntas: `[{x,y,z}, {x,y,z}, ...]`. Contraste com SoA.

### B

**BGK (Bhatnagar-Gross-Krook)**  
O modelo de colisão LBM mais simples. Relaxa a função de distribuição em direção ao equilíbrio local a uma taxa única `1/τ`.

**Bounding Box (Caixa Delimitadora)**  
A menor caixa retangular alinhada aos eixos que contém toda a geometria.

**Bounce-Back (Condição de Contorno de Reflexão)**  
Condição de contorno em LBM que reflete as funções de distribuição entrantes de volta na direção de onde vieram. Impõe sem-deslizamento em paredes sólidas.

**Aproximação de Boussinesq**  
Aproximação para escoamentos impulsionados por empuxo: trata o fluido como incompressível, exceto no termo de empuxo.

### C

**CFD (Dinâmica dos Fluidos Computacional)**  
Simulação numérica de escoamentos de fluidos governados pelas equações de Navier-Stokes.

**CGNS (CFD General Notation System)**  
Formato de arquivo padrão industrial para armazenar malhas computacionais estruturadas e não estruturadas.

**Comprimento Característico (Characteristic Length)**  
Escala de comprimento representativa da geometria ou escoamento, usada para calcular números adimensionais como o número de Reynolds.

**Etapa de Colisão (Collision Step)**  
Em LBM: a redistribuição local de funções de distribuição de probabilidade para relaxar em direção ao equilíbrio termodinâmico.

**Convergência (Convergence)**  
A propriedade de que o resíduo da simulação diminui em direção a zero conforme a simulação atinge um estado estacionário.

### D

**D2Q9**  
Conjunto de velocidades de Lattice Boltzmann 2D com 9 velocidades discretas.

**D3Q19**  
Conjunto de velocidades de Lattice Boltzmann 3D com 19 velocidades discretas.

**FDF Dupla (DDF — Double Distribution Function)**  
Abordagem LBM para escoamentos térmicos com duas funções de distribuição separadas.

### E

**Distribuição de Equilíbrio (Equilibrium Distribution)**  
Em LBM, a distribuição de Maxwell-Boltzmann expandida até segunda ordem em velocidade.

### F

**f (função de distribuição)**  
A variável fundamental em LBM. Representa a densidade de probabilidade de encontrar partículas em uma posição e tempo dados, movendo-se em uma direção específica.

### L

**Método de Lattice Boltzmann (LBM)**  
Técnica de simulação mesoscópica para dinâmica de fluidos. Evolui funções de distribuição de probabilidade em uma rede regular.

**Norma L2**  
`‖e‖₂ = √(Σ eᵢ²/N)`. O erro quadrático médio.

**Norma L∞ (Norma Máxima)**  
`‖e‖∞ = max|eᵢ|`. O erro pontual máximo no domínio.

### M

**Número de Mach (Ma)**  
`Ma = U / c_s`. LBM é válido para `Ma < 0,3`.

**MLUPS (Milhões de Atualizações de Rede por Segundo)**  
Métrica de performance para solvers LBM.

### N

**Condição de Contorno Sem-Deslizamento (No-Slip)**  
A velocidade do fluido é igual à velocidade da parede no contorno sólido.

**Número de Nusselt (Nu)**  
`Nu = h·L/k`. Coeficiente adimensional de transferência de calor.

**Equações de Navier-Stokes**  
As equações fundamentais do movimento de fluidos.

### P

**Escoamento de Poiseuille (Poiseuille Flow)**  
Escoamento estacionário, completamente desenvolvido, conduzido por pressão entre placas paralelas ou em um tubo. Tem solução analítica exata (perfil de velocidade parabólico).

### R

**Número de Rayleigh (Ra)**  
`Ra = g·β·ΔT·L³ / (ν·α)`. Governa a convecção natural.

**Número de Reynolds (Re)**  
`Re = U·L / ν`. Razão das forças inerciais para viscosas.

**Resíduo (Residual)**  
Medida de quanto a solução está mudando entre passos de tempo sucessivos.

### S

**SoA (Structure of Arrays)**  
Layout de memória onde cada propriedade é armazenada em seu próprio array contíguo. Preferido no AeroCore por eficiência SIMD.

**SoaMesh**  
Estrutura de dados de malha do AeroCore, armazenando vértices e normais em layout SoA.

**Etapa de Streaming (Streaming Step)**  
Em LBM: propagação das funções de distribuição de cada célula para seus vizinhos.

**STL (STereoLithography)**  
Formato de arquivo de malha de superfície triangulada, amplamente suportado por ferramentas CAD.

**Número de Strouhal (St)**  
`St = f·D/U`. Frequência adimensional de desprendimento de vórtices.

### T

**τ (Tempo de Relaxação)**  
Parâmetro do modelo de colisão BGK. `ν = c_s²·(τ - 0,5)`. Deve satisfazer `τ > 0,5` para estabilidade.

### V

**Validação (Validation)**  
Comparação dos resultados da simulação com soluções de referência conhecidas.

**Verificação (Verification)**  
Demonstração de que a implementação numérica resolve corretamente as equações pretendidas.

**Viscosidade Cinemática (ν)**  
`ν = μ/ρ`. Em unidades de rede LBM: `ν = c_s²·(τ - 0,5)`.

**VTK (Visualization Toolkit)**  
Formato de arquivo aberto para visualização científica, suportado pelo ParaView.

**Voxelização (Voxelization)**  
Processo de converter uma malha de superfície triangulada (STL) em uma grade 3D de valores binários de ocupação (sólido=1, fluido=0).
