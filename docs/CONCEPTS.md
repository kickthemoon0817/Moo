# PhysicLaw: Core Concepts & Architecture

This document describes the foundational physics concepts, mathematical formulations, and compute architecture used in the PhysicLaw engine.

## 1. Physics Philosophy: Variational vs. Vectorial
PhysicLaw aims to bridge the gap between **Geometric Mechanics** (structure-preserving) and real-time physics simulation.

*   **Vectorial (Newtonian)**: $F = ma$. Intuitive but prone to energy drift in numerical integration.
*   **Variational (Lagrangian)**: $\delta \int L dt = 0$. Derives equations of motion from energy principles. By discretizing the Lagrangian directly (Discrete Variational Mechanics), we can obtain integrators that naturally preserve momentum and symplectic form (phase space volume).

While our fluid solver currently uses a standard Force-based SPH approach (Vectorial), the rigid body and constraint systems aim to adhere to Variational principles where possible to ensure long-term stability.

## 2. Fluid Dynamics: SPH (Smoothed Particle Hydrodynamics)
We use a **Lagrangian** (particle-based) approach to simulate fluids, specifically **Weakly Compressible SPH (WCSPH)**.

### The Smoothed Approximation
Any field quantity $A(\mathbf{r})$ at position $\mathbf{r}$ is approximated by a weighted sum over neighboring particles:

$$ A(\mathbf{r}) \approx \sum_j m_j \frac{A_j}{\rho_j} W(\mathbf{r} - \mathbf{r}_j, h) $$

Where:
*   $m_j$: Mass of particle $j$.
*   $\rho_j$: Density of particle $j$.
*   $W$: Smoothing kernel function with radius $h$.

### Kernels
We use specific kernels designed for stability (avoiding division by zero or negative pressures):

1.  **Poly6 Kernel** (Density Calculation):
    *   Used for $\rho$ estimation.
    *   Smooth everywhere, $W \propto (h^2 - r^2)^3$.
2.  **Spiky Kernel** (Pressure Force):
    *   Used for gradient calculation ($\nabla W$).
    *   Non-zero gradient near $r=0$, preventing clustering instability.
    *   $\nabla W \propto (h - r)^2$.

### Governing Equations
1.  **Density**: Recomputed every frame based on neighbors.
2.  **Equation of State (Tait)**: Links density to pressure to enforce incompressibility (stiff fluid).
    $$ P = B \left( \left( \frac{\rho}{\rho_0} \right)^\gamma - 1 \right) $$
    *   $\gamma = 7$ (standard for water).
    *   $B$: Stiffness constant (controls speed of sound).
3.  **Forces**:
    *   **Pressure**: $-\nabla P$. Pushes particles apart to maintain rest density $\rho_0$.
    *   **Viscosity**: Smoothes velocity differences (Laplacian of velocity) to simulate thickness/friction.
    *   **Gravity**: Constant external force.

## 3. Position Based Dynamics (PBD/XPBD)
*Note: While our current fluid solver is Force-Based (SPH), PBD is a key concept in our broader architecture references.*

**PBD** (Müller et al.) abandons force integration for constraints. Instead, it:
1.  Predicts a tentative position $\mathbf{x}^*$ using just velocity/inertia.
2.  Solves constraints $C(\mathbf{x}) = 0$ by directly projecting $\mathbf{x}^*$ to a valid $\mathbf{x}$.
3.  Updates velocity based on the positional correction $\Delta \mathbf{x}$.

**Pros**: Unconditionally stable, easy to control (stiffness approx. by iteration count).
**Cons**: Stiffness depends on time-step and iteration count (fixed by XPBD).

## 4. Time Integration
For the SPH simulation, we use **Symplectic Euler** (Semi-Implicit Euler):

1.  $\mathbf{v}_{t+1} = \mathbf{v}_t + \mathbf{a}(\mathbf{x}_t) \cdot \Delta t$
2.  $\mathbf{x}_{t+1} = \mathbf{x}_t + \mathbf{v}_{t+1} \cdot \Delta t$

**Why?**
*   **Symplectic**: It preserves the symplectic 2-form area $dp \wedge dq$ in phase space.
*   **Energy Stability**: Unlike Explicit Euler (which gains energy) or Implicit Euler (which damps energy constantly), Symplectic Euler bounds the energy error, making it stable for long-term orbital or oscillatory motion without artificial damping.
*   **Performance**: Explicit-like cost (cheap) with Implicit-like stability properties for conservation.

## 5. Compute Architecture (GPU)
The simulation runs entirely on the GPU using **WGPU Compute Shaders** (WGSL).

### Two-Pass SPH
To handle particle interactions efficiently:
1.  **Density Pass**: Read positions, calculate $\rho$ for all particles. Write to `density_buffer`.
2.  **Force Integration Pass**: Read $\rho$ and positions, calculate Forces ($F_{press}, F_{visc}, F_{grav}$), update Velocity and Position.

### Neighbor Search (Grid)
Naive $O(N^2)$ checks are too slow. We use **Uniform Grid Spatial Hashing**:
1.  **Hash**: Map particle position to a grid cell index.
2.  **Sort**: Sort particles by cell index (Bitonic Sort or Radix Sort on GPU).
3.  **Offsets**: Build an offset table pointing to the start/end of each cell in the sorted array.
4.  **Query**: Each particle only checks 27 neighbor cells ($3 \times 3 \times 3$) instead of all $N$ particles.

### Ping-Pong Buffers
Since we cannot read and write to the same buffer safely in parallel (race conditions):
*   `ParticleBuffer A` (Read) -> `Compute` -> `ParticleBuffer B` (Write)
*   **Swap**
*   `ParticleBuffer B` (Read) -> `Compute` -> `ParticleBuffer A` (Write)
