# PhysicLaw: Core Concepts & Architecture

This document describes the foundational physics concepts, mathematical formulations, and compute architecture used in the PhysicLaw engine. It is a rigorous contributor reference: every formula here is derivable from the cited sources, and every implementation detail is grounded in the source code.

For the philosophical motivation behind these choices — including the research vision that goes beyond what is implemented here — read `PHILOSOPHY.md` first.

---

## 1. Physics Philosophy: Variational vs. Vectorial

PhysicLaw is built on **Geometric Mechanics** — the formulation of classical physics as geometry on smooth manifolds. This section traces the full theoretical chain from Newton to the symplectic structure of phase space.

### 1.1 The Newtonian (Vectorial) Approach — What We Reject

The conventional formulation F = ma specifies dynamics by assigning a force vector to each particle at each instant. Its pathologies for long-horizon simulation are well-documented:

- **Energy drift**: explicit integrators inject or dissipate energy monotonically.
- **No conservation guarantee**: a hand-coded force is not guaranteed to be conservative.
- **Coordinate dependence**: F = ma holds only in inertial Cartesian frames.

### 1.2 The Lagrangian Formulation

**Configuration space** Q is the smooth manifold of all generalized coordinates q = (q₁, ..., qₙ). For a system of N particles in ℝ³, Q = ℝ^{3N}. For a system including rigid bodies, Q = ℝ^{3N} × SO(3)^M.

The **tangent bundle** TQ pairs each configuration q with a velocity q̇. The **Lagrangian** L: TQ → ℝ is:

```
L(q, q̇) = T(q̇) − V(q) = ½ Σᵢ mᵢ ‖q̇ᵢ‖² − V(q)
```

The **action functional** over a path q: [t₁, t₂] → Q is:

```
S[q] = ∫_{t₁}^{t₂} L(q(t), q̇(t)) dt
```

**Hamilton's Principle** (Principle of Stationary Action): the physical path satisfies δS = 0, i.e., the first variation of S vanishes. This yields the **Euler-Lagrange equations**:

```
d/dt(∂L/∂q̇ᵢ) − ∂L/∂qᵢ = 0    for each i = 1, ..., n
```

For L = T − V with Cartesian coordinates, these reduce to F = ma. In generalized coordinates, they handle constraints and curved geometry automatically.

### 1.3 Noether's Theorem

If the Lagrangian L is invariant under a smooth one-parameter family of configuration-space maps φₛ: Q → Q (i.e., L(φₛ(q), d/dt φₛ(q)) = L(q, q̇) for all s), then the **momentum map**

```
J = Σᵢ (∂L/∂q̇ᵢ) · Xᵢ(q),    where X = dφₛ/ds|_{s=0}
```

is conserved along every Euler-Lagrange solution: dJ/dt = 0.

**Concrete instances:**

| Symmetry of L | Conserved quantity |
|---|---|
| Invariance under translation q → q + δ | Linear momentum p = Σ mᵢvᵢ |
| Invariance under rotation q → Rq | Angular momentum L = Σ qᵢ × mᵢvᵢ |
| Invariance under time shift t → t + s | Energy H = T + V |

Because PhysicLaw laws specify V(q) and the engine derives forces as F = −∇V, any potential that is translationally or rotationally symmetric will automatically conserve the corresponding quantity — as a consequence of the structure, not of a manual conservation check.

### 1.4 The Hamiltonian Formulation

Define the **generalized momentum** conjugate to qᵢ:

```
pᵢ = ∂L/∂q̇ᵢ = mᵢ q̇ᵢ
```

The **Legendre transform** maps from TQ to the **cotangent bundle** T*Q, yielding the **Hamiltonian**:

```
H(q, p) = Σᵢ pᵢ q̇ᵢ − L(q, q̇) = T(p) + V(q)
```

where T(p) = Σᵢ pᵢ²/(2mᵢ). Hamilton's equations of motion on T*Q:

```
dqᵢ/dt = +∂H/∂pᵢ = pᵢ/mᵢ
dpᵢ/dt = −∂H/∂qᵢ = −∂V/∂qᵢ = Fᵢ
```

**The symplectic 2-form** on T*Q:

```
ω = Σᵢ dpᵢ ∧ dqᵢ
```

is preserved by Hamiltonian flow: if Φ_t: T*Q → T*Q is the time-t map of the Hamiltonian vector field X_H, then Φ_t* ω = ω (the pullback of ω by Φ_t equals ω). This **symplecticity** is equivalent to Liouville's theorem (phase space volume preservation) and to the existence of Poincaré invariants.

**Conservation of H:** Along any trajectory, dH/dt = 0 when H has no explicit time dependence. This follows from Hamilton's equations directly:

```
dH/dt = Σᵢ (∂H/∂qᵢ · q̇ᵢ + ∂H/∂pᵢ · ṗᵢ)
      = Σᵢ (∂H/∂qᵢ · ∂H/∂pᵢ − ∂H/∂pᵢ · ∂H/∂qᵢ) = 0
```

### 1.5 Discrete Variational Mechanics — Current Bridge, Not Destination

**Discrete Variational Mechanics** (Marsden & West, 2001) provides a principled way to construct structure-preserving numerical integrators by discretizing the variational principle, not the equations of motion.

Given a discrete path {q₀, q₁, ..., q_N}, define a **discrete Lagrangian** L_d: Q × Q → ℝ:

```
L_d(qₖ, qₖ₊₁) ≈ ∫_{tₖ}^{tₖ₊₁} L(q(t), q̇(t)) dt
```

**Discrete Hamilton's Principle:** δ Σₖ L_d(qₖ, qₖ₊₁) = 0 for variations fixing the endpoints. This yields the **Discrete Euler-Lagrange (DEL) equations**:

```
D₂ L_d(qₖ₋₁, qₖ) + D₁ L_d(qₖ, qₖ₊₁) = 0
```

The **discrete Legendre transforms** define discrete momenta:
```
p₊ₖ = −D₁ L_d(qₖ, qₖ₊₁)    (right discrete momentum at step k)
p⁻ₖ = +D₂ L_d(qₖ₋₁, qₖ)    (left discrete momentum at step k)
```

The DEL equation is exactly p⁻ₖ = p₊ₖ — discrete momentum matching. The one-step map (qₖ, p₊ₖ) → (qₖ₊₁, p₊ₖ₊₁) is **exactly symplectic**: it preserves the discrete symplectic form Ωd = dqₖ ∧ dp₊ₖ with zero accumulated error.

**In PhysicLaw,** the Velocity Verlet integrator uses the midpoint-rule L_d:

```
L_d(qₖ, qₖ₊₁) = h · L((qₖ + qₖ₊₁)/2, (qₖ₊₁ − qₖ)/h)
```

The force computation via AD is the discrete realization of −D₁V appearing in the DEL equations.

> **Important caveat:** DVM is our current engineering bridge — the best currently-understood method for conservative simulation. It is still Δt-parameterized and still approximates the continuous system. The open research agenda (Section 8) is the pursuit of methods that transcend this.

---

## 2. Fluid Dynamics: SPH (Smoothed Particle Hydrodynamics)

We use a **Lagrangian** (particle-based) approach to simulate fluids: **Weakly Compressible SPH (WCSPH)**.

*Note: the SPH fluid solver currently uses a force-based (vectorial) approach on the GPU compute pipeline. Its SPH force decomposition is not fully grounded in the variational framework — see PHILOSOPHY.md for why this is an open problem.*

### 2.1 The Smoothed Approximation

Any field quantity A(r) at position r is approximated by a weighted sum over neighboring particles:

```
A(r) ≈ Σⱼ mⱼ (Aⱼ/ρⱼ) W(r − rⱼ, h)
```

Where:
- mⱼ: mass of particle j
- ρⱼ: density of particle j
- W: smoothing kernel with support radius h

### 2.2 Kernels

1. **Poly6 Kernel** (density computation):
   - Used for ρ estimation. Smooth everywhere.
   - W ∝ (h² − r²)³

2. **Spiky Kernel** (pressure gradient):
   - Used for ∇W. Non-zero gradient near r = 0, preventing particle clustering.
   - ∇W ∝ (h − r)²

### 2.3 Governing Equations

1. **Density** (recomputed each frame from neighbors):
   ```
   ρᵢ = Σⱼ mⱼ W(rᵢ − rⱼ, h)
   ```

2. **Equation of State (Tait)** — links density to pressure (weakly compressible):
   ```
   P = B((ρ/ρ₀)^γ − 1),    γ = 7
   ```
   B is the stiffness constant controlling the effective speed of sound.

3. **Force contributions:**
   - **Pressure**: F_press = −∇P, pushes particles to maintain rest density ρ₀
   - **Viscosity**: F_visc = μ ∇²v, smooths velocity differences
   - **Gravity**: constant external acceleration g

---

## 3. Position-Based Dynamics (PBD/XPBD)

*Note: PBD is a reference architecture, not our primary method. It is documented here for contrast.*

**PBD** (Müller et al.) abandons force integration in favor of direct constraint projection:

1. Predict a tentative position x* using velocity/inertia alone.
2. Solve constraints C(x) = 0 by projecting x* onto the constraint manifold.
3. Update velocity from the positional correction: Δv = Δx / dt.

**Pros:** Unconditionally stable; stiffness controlled by iteration count.
**Cons:** Stiffness is dt-dependent (fixed in XPBD); not energy-preserving; not symplectic.

PhysicLaw uses a variational constraint framework (geometric projection after a symplectic step) rather than PBD's position correction loop.

---

## 4. Numerical Integration & Its Limitations

The integrator is the engine's approximation of the exact Hamiltonian flow Φ_t. This section describes what our current integrators can and cannot do.

### 4.1 Symplectic Euler (First-Order)

**Implementation** (`solve/mod.rs`, `SymplecticEuler`):

```
1. Compute forces:  Fᵢ = −∂V/∂qᵢ  via AD (seeding q[i].der = 1.0)
2. Velocity update: vᵢ ← vᵢ + (Fᵢ/mᵢ)·Δt   (momentum update)
3. Position update: qᵢ ← qᵢ + vᵢ·Δt          (position update, uses new v)
4. Constraint projection
```

The critical ordering (velocity before position) is what makes this symplectic rather than explicit. Equivalent to DVM with the discrete Lagrangian:

```
L_d(qₖ, qₖ₊₁) = h · L(qₖ, (qₖ₊₁ − qₖ)/h)
```

**Order:** 1st order in Δt. **Symplectic:** yes. **Time-reversible:** no.

### 4.2 Velocity Verlet / Störmer-Verlet (Second-Order)

**Implementation** (`solve/mod.rs`, `VelocityVerlet`):

```
1. Half-kick:   vᵢ ← vᵢ + (Fᵢ(qₖ)/mᵢ)·(Δt/2)
2. Drift:       qᵢ ← qᵢ + vᵢ·Δt
3. Constraint projection (on updated positions)
4. Recompute:   Fᵢ(qₖ₊₁) via AD
5. Half-kick:   vᵢ ← vᵢ + (Fᵢ(qₖ₊₁)/mᵢ)·(Δt/2)
```

Equivalent to DVM with the midpoint-rule L_d. **Order:** 2nd order in Δt. **Symplectic:** yes. **Time-reversible:** yes (for symmetric potentials). The preferred integrator for conservative systems.

The rigid body rotation substep uses `SO3::retract(rot, ω·Δt)` — the Lie group exponential map — ensuring rotation matrices remain in SO(3) exactly.

### 4.3 What Symplectic Integration Cannot Solve

It is important to be precise about what symplectic integration guarantees and what it does not:

**Guaranteed:**
- The discrete symplectic form is preserved exactly at each step.
- Energy error is bounded (does not grow secularly) — it oscillates around a shadow Hamiltonian H̃ = H + O(Δt²) for Verlet.
- Momentum maps corresponding to exact symmetries of the discrete Lagrangian are preserved exactly.

**Not guaranteed:**
- **Exact trajectory tracking.** The numerical trajectory is the exact trajectory of H̃, not H. Phase error accumulates: two identical systems with different Δt diverge in trajectory even if their energy is bounded.
- **Absence of numerical viscosity.** Temporal sampling introduces an effective viscosity in SPH even with zero physical viscosity.
- **Sub-Δt event resolution.** Events (collisions, contact) occurring within a single step are missed or misplaced by exactly Δt.
- **CFL compliance.** The engine cannot enforce Δt < h/v_max automatically; this is the user's responsibility. Violation causes tunneling.

### 4.4 Adaptive Sub-Stepping (Current Mitigation)

The **CFL condition** Δt < h/v_max prevents tunneling and force explosion:

```
Δt_CFL = λ · h / v_max,    λ ∈ (0, 1)  (safety factor)
```

Current implementation computes v_max via a GPU reduction each frame and adjusts sub-step count. This is a *mitigation*, not a solution: it reduces the problem but does not eliminate the fundamental Δt-dependence.

### 4.5 Future Directions (Research Targets)

The following methods are research targets for replacing or augmenting the current Δt-stepping architecture. They are not yet implemented.

- **Gauss-Legendre collocation on manifolds:** Implicit Runge-Kutta methods using Gauss-Legendre quadrature nodes. These are symplectic and of arbitrarily high order. The cost is solving a nonlinear system per step, but the accuracy gain can more than compensate for larger Δt.
- **Generating function methods** (Feng Kang, 1984): represent the exact flow map Φ_t as a generating function S(q₀, q₁, t) and evaluate it directly, bypassing step-by-step propagation.
- **Event-driven integration:** advance to exact event times for topology-changing interactions (collisions, contact), interpolate or solve exactly for smooth force fields in between.
- **Lie group integrators** (Munthe-Kaas, Iserles): extend DVM to Lie group configuration spaces for rigid bodies and field theories, replacing the current splitting approach.

---

## 5. Compute Architecture (GPU)

The simulation runs on the GPU using **WGPU Compute Shaders** (WGSL). The CPU-side engine (`moo`) interfaces with the GPU via the `ComputeEngine`.

### 5.1 Two-Pass SPH

Particle interactions require a two-pass algorithm to avoid race conditions:

1. **Density Pass:** Read positions, compute ρᵢ = Σⱼ mⱼ W(rᵢ − rⱼ, h). Write to `density_buffer`.
2. **Force Integration Pass:** Read ρ and positions, compute pressure + viscosity forces, update velocity and position with Symplectic Euler.

### 5.2 Spatial Hashing (Neighbor Search)

Naive O(N²) neighbor checks are replaced with **Uniform Grid Spatial Hashing**:

1. **Hash:** Map each particle position to a grid cell index.
2. **Sort:** Sort particles by cell index (Bitonic Sort — `sort.wgsl`).
3. **Offsets:** Build a compact offset table pointing to each cell in the sorted array (`grid.wgsl`).
4. **Query:** Each particle checks only the 27 neighboring cells (3×3×3) instead of all N particles.

Total cost: O(N log N) for sort + O(N · k̄) for neighbor queries, where k̄ is the average neighbor count.

### 5.3 Ping-Pong Buffers

To avoid GPU read-write race conditions in parallel compute:

- `ParticleBuffer A` (read) → Compute → `ParticleBuffer B` (write)
- Swap A and B
- Repeat

This is the standard GPU double-buffering pattern. The `ComputeEngine` manages buffer lifetimes and swap coordination.

---

## 6. Advanced Continuum & Field Theories (Phase 18)

These are research items for high-fidelity scientific simulation beyond current game physics accuracy.

### 6.1 Non-Linear Constitutive Models (Hyperelasticity)

Standard spring constraints (Hooke's Law, F = −kx) model linear elasticity valid only for small deformations. For large-deformation soft bodies (rubber, biological tissue), we use **Hyperelastic** models derived from a Strain Energy Density function Ψ.

**Neo-Hookean Model:**
```
Ψ = C₁(I₁ − 3) + D₁(J − 1)²
```
Where I₁ is the first invariant of the deformation gradient tensor F, and J = det(F). This models non-linear stiffening: the material resists harder the more it is stretched.

### 6.2 Relativistic Gravity (Post-Newtonian)

Newtonian gravity (F = GMm/r²) is an approximation valid for slow speeds and weak fields. For cosmic precision (Mercury perihelion precession, binary pulsar timing), a perturbative post-Newtonian potential is applied:

```
Φ_PN ∝ L²/r³
```

This approximates first-order general relativistic corrections without a full GR tensor solver.

### 6.3 Field-Based Multi-Physics

Unified field approach for coupled simulation:
- **Thermal:** Advection-diffusion on the particle field: dT/dt = α ∇²T
- **Electromagnetism:** Maxwell-Vlasov solver for charged fluid particles (plasma confinement)

---

## 7. Hamiltonian Formulation & Phase Space

This section connects the abstract Hamiltonian theory to PhysicLaw's concrete data structures.

### 7.1 Phase Space and the PhaseSpace Struct

The **phase space** T*Q for a system with n degrees of freedom is a 2n-dimensional manifold. In PhysicLaw, the `PhaseSpace` struct is the direct computational realization of a point in T*Q:

```rust
// Conceptually:
PhaseSpace {
    q:    Vec<f64>,   // generalized coordinates  ∈ Q
    v:    Vec<f64>,   // velocities (≈ p/m)       ∈ T*Q
    mass: Vec<f64>,   // mass per DOF (constants)
    t:    f64,        // simulation time
    // Rigid body extensions:
    rot:     Vec<Quat>,  // SO(3) orientations
    ang_v:   Vec<Vec3>,  // angular velocities
    inertia: Vec<f64>,   // scalar moments of inertia
}
```

The canonical momentum is pᵢ = mᵢ vᵢ; the struct stores velocity rather than momentum for implementation convenience, but the Hamiltonian structure is unchanged.

### 7.2 The Hamiltonian in the Engine

The Hamiltonian H = T + V is never stored explicitly. It is evaluated on-demand:

```
H = Σᵢ pᵢ²/(2mᵢ) + V_total(q) = Σᵢ ½mᵢvᵢ² + laws.potential(q, mass).val
```

The `EnergyProbe` computes this quantity at each frame to verify conservation. In a correctly-integrated symplectic system, H should oscillate with amplitude O(Δt²) around a constant value — it should not drift monotonically. Monotonic drift indicates either:
1. A non-conservative law (force without a valid potential), or
2. A non-symplectic integration method.

### 7.3 Conservation Monitoring via the EnergyProbe

The `EnergyProbe` serves as an empirical Noether check. If a law implementing V(q) correctly respects a symmetry, the corresponding momentum map should be conserved to machine precision. Energy conservation (H = const) is the special case corresponding to time-translation symmetry.

For a system with translation-invariant V (e.g., gravity between particles depends only on rᵢ − rⱼ):

```
J_translation = Σᵢ mᵢ vᵢ = const    (total momentum conserved)
```

This can be verified analytically. Any deviation indicates a bug in the law's potential function.

### 7.4 Geometric Structure of Rigid Body Phase Space

For rigid bodies, the configuration space includes SO(3). The full phase space for a single rigid body is T*SO(3) — the cotangent bundle of the rotation group. This is a 6-dimensional symplectic manifold.

The current implementation uses a splitting approach:
- Translational DOFs: standard Verlet on ℝ³
- Rotational DOFs: Euler equations on so(3) (the Lie algebra), integrated with SO3::retract

This splitting is an approximation to the exact symplectic flow on T*SO(3). A proper Lie group variational integrator (Lee, 2009) would treat both simultaneously on the manifold, preserving the full symplectic structure without the splitting approximation.

---

## 8. Open Research Questions

These are the genuine open problems at the frontier of PhysicLaw's research agenda. They are designated as Phase 18+ items — not because they are unimportant, but because their difficulty requires the engine to be mature enough to experiment with them safely.

### 8.1 What Does a Δt-Free Physics Engine Look Like?

The fundamental question: can we represent and simulate Hamiltonian mechanics without ever committing to a time-step size?

Candidate answers:
- **Generating functions:** If S(q₀, q₁; t) can be evaluated efficiently, the exact flow Φ_t is a function call, not an iteration. The Hamilton-Jacobi equation governs S; computing it for arbitrary potentials V(q) is an open problem in numerical analysis.
- **Jacobi's principle (timeless geodesics):** Recast mechanics as geodesic flow on Q with the Jacobi metric ds² = 2(E − V)T(dq). Time becomes derived, not fundamental. Requires a fundamentally different simulation loop.

### 8.2 Event-Driven SPH

Current SPH advances all particles at fixed Δt. An event-driven alternative:
- Compute, for each particle pair (i, j), the time t*_{ij} at which their separation equals the smoothing radius h (onset of interaction).
- Advance all particles to t*_{ij}, apply the interaction exactly.
- Recompute next events.

Challenge: SPH particles interact continuously (not at discrete contact events), so event times are roots of transcendental equations ‖rᵢ(t) − rⱼ(t)‖ = h. Analytical solutions exist only for ballistic trajectories (no force). With force fields, the event time requires an iterative solver — which introduces a new approximation, but one with controllable accuracy independent of Δt.

### 8.3 Continuous-Time Rigid Body Dynamics

For torque-free rigid body motion, Euler's equations:
```
I ω̇ + ω × (Iω) = 0
```
have exact closed-form solutions in terms of Jacobi elliptic functions. For simple cases (axisymmetric bodies), the solution is exact rotation at constant angular velocity. These exact solutions could replace the numerical Euler integration in `VelocityVerlet`, eliminating all angular velocity integration error.

For driven rigid bodies (non-zero torque), exact solutions don't exist in closed form, but **Lie group variational integrators** (Lee, 2009) on SO(3) preserve the Hamiltonian structure of rotational motion exactly — at the cost of solving a nonlinear system per step.

### 8.4 Spectrally-Accurate Structure-Preserving Integrators

**Gauss-Legendre Runge-Kutta** methods of order 2s (using s internal stages) are symplectic for all s. They achieve spectral accuracy: error decreases exponentially with s for smooth problems, not just polynomially with Δt.

For PhysicLaw, this means: instead of taking many small Verlet steps, take one large Gauss-Legendre step of high order. The potential savings are enormous for smooth force fields (gravity, spring potentials), where the solution is analytic and highly regular.

**Munthe-Kaas methods** extend Gauss-Legendre quadrature to Lie group configuration spaces — making them directly applicable to the SO(3) rigid body DOFs.

### 8.5 Variational Discretization of SPH

The current SPH solver computes forces directly (non-variational). The research question: can SPH be reformulated as a variational method — with a discrete Lagrangian L_d on the space of particle configurations — so that the SPH update is symplectic?

Español & Revenga (2003) and Bonet & Kulasegaram (2000) have shown that variational SPH formulations exist for certain kernel choices. Integrating this with the existing LawRegistry framework would bring the fluid solver under the same structural guarantees as the rigid body and particle systems.

---

*For the reading list that supports these concepts, see `REFERENCES.md`. For the philosophical vision that motivates this research agenda, see `PHILOSOPHY.md`.*
