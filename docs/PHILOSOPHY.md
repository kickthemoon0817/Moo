# PhysicLaw: Philosophy & Research Vision

> *"Time is not an ingredient of physics. It is what we read off a clock that is itself a physical system."*
>
> — Julian Barbour, *The End of Time*

---

## 1. The Crisis of Digital Physics

Every physics simulation is an act of translation: we take the continuous, smooth universe — governed by differential equations over ℝ — and project it onto the discrete, finite world of a computer. This translation is unavoidable, but it is not neutral. The choices we make in how we discretize determine not just numerical accuracy, but which physics is *representable at all*.

The dominant paradigm in real-time simulation — used by every major game engine, most robotics simulators, and many scientific codes — is the **Newtonian explicit integrator**:

```
a = F/m   →   v_{t+Δt} = v_t + a·Δt   →   x_{t+Δt} = x_t + v_{t+Δt}·Δt
```

This is mathematically simple and intuitively direct. It is also structurally broken for any long-horizon use.

**The energy drift problem.** Explicit Euler integration is not symplectic — it does not preserve the geometric structure of Hamiltonian flow. Energy leaks into or out of the system at a rate proportional to Δt. Long simulations (planetary orbits, molecular dynamics, structural analysis) become unphysical within seconds or minutes of simulation time. This is not fixable by tuning Δt; it is an inherent property of the method.

**The force specification problem.** In Newtonian formulations, developers specify *forces* directly. But not every vector field F(q) is a conservative gradient — it is trivially easy to write a force function that violates energy conservation or Newton's third law without realizing it. The engine has no mechanism to detect or prevent this. Every custom interaction is a latent source of spurious energy injection.

**The rate-dependence problem.** With F=ma at fixed Δt, the simulation's behavior changes when you change Δt in ways that are not physically motivated. Increase Δt and springs become unstable. Decrease Δt and oscillatory modes appear that have no counterpart in the continuous system. The *physics is parameterized by a computational artifact*.

**The tunneling problem.** At any finite Δt, a particle moving faster than h/Δt (where h is the interaction radius) can pass through a barrier between one frame and the next, registering no collision. The CFL condition — the requirement that Δt < h/v_max — is not a physical law. It is a symptom of the conflict between the discrete simulation and continuous causality.

PhysicLaw was founded on the recognition that this paradigm is not a good starting point for a physics engine that aims at correctness.

---

## 2. The Variational Turn — What We Have Built

The correct starting point for classical mechanics is not F = ma. It is the **Principle of Stationary Action**:

> *Among all paths from configuration q₁ at time t₁ to q₂ at time t₂, the physical path is the one for which the action S = ∫L dt is stationary: δS = 0.*

Here **L: TQ → ℝ** is the **Lagrangian**, defined on the tangent bundle of configuration space Q (the manifold of all generalized coordinates and velocities). For most mechanical systems:

```
L(q, q̇) = T(q̇) − V(q) = ½m‖q̇‖² − V(q)
```

The **Euler-Lagrange equations** derived from δS = 0:

```
d/dt(∂L/∂q̇ᵢ) − ∂L/∂qᵢ = 0
```

reduce to F = ma only in Cartesian coordinates with L = T − V. In generalized coordinates, they automatically account for constraints, curvature of the configuration manifold, and non-inertial frames — without the user ever specifying pseudo-forces.

### Noether's Theorem

The deepest result connecting the Lagrangian formulation to conservation laws:

> *Every continuous symmetry of the Lagrangian corresponds to exactly one conserved quantity.*

Concretely: if L(q, q̇) is invariant under a one-parameter family of diffeomorphisms φₛ: Q → Q (with generator X_q = ∂φₛ/∂s|_{s=0}), then the **momentum map**

```
J = Σᵢ (∂L/∂q̇ᵢ) · (Xq)ᵢ
```

is conserved along every solution. Specific cases:
- L invariant under spatial translation → linear momentum conserved
- L invariant under rotation → angular momentum conserved
- L invariant under time translation → total energy (= H = T + V) conserved

This is not merely a computational convenience — it is the *explanation* for why these quantities are conserved. Any simulation method that respects the Lagrangian structure inherits these conservation laws automatically.

### The Hamiltonian Picture

The Lagrangian lives on TQ (positions × velocities). Via the **Legendre transform**:

```
pᵢ = ∂L/∂q̇ᵢ    (generalized momentum conjugate to qᵢ)
H(q, p) = Σᵢ pᵢ q̇ᵢ − L(q, q̇) = T + V    (Hamiltonian)
```

we pass to the **Hamiltonian** on the cotangent bundle T*Q (positions × momenta), governed by Hamilton's equations:

```
dqᵢ/dt = +∂H/∂pᵢ
dpᵢ/dt = −∂H/∂qᵢ
```

The phase space T*Q carries a canonical **symplectic 2-form**:

```
ω = Σᵢ dpᵢ ∧ dqᵢ
```

Hamiltonian flow is a one-parameter family of **symplectomorphisms** — maps that preserve ω. This is the geometric content of Hamiltonian mechanics: the volume of any region in phase space is invariant (Liouville's theorem), and the symplectic structure itself is an exact invariant. Energy conservation, when it holds, is a consequence of time-translation symmetry via Noether — but symplecticity is a *stronger* geometric statement that holds regardless of whether H is conserved.

### Discrete Variational Mechanics

The key insight of Marsden & West (2001) is that the correct way to discretize a Hamiltonian system is *not* to discretize the equations of motion, but to **discretize the variational principle itself**.

Replace the continuous path q: [t₀, t_N] → Q with a discrete sequence {q₀, q₁, ..., q_N} ∈ Q. Define a **discrete Lagrangian** L_d: Q × Q → ℝ approximating the exact action over one step:

```
L_d(qₖ, qₖ₊₁) ≈ ∫_{tₖ}^{tₖ₊₁} L(q(t), q̇(t)) dt
```

The **Discrete Hamilton's Principle** (δ Σₖ L_d = 0 over interior nodes) yields the **Discrete Euler-Lagrange (DEL) equations**:

```
D₂ L_d(qₖ₋₁, qₖ) + D₁ L_d(qₖ, qₖ₊₁) = 0
```

where D₁, D₂ denote partial derivatives with respect to the first and second slot. The discrete Legendre transform extracts discrete momenta pₖ. The resulting one-step map (qₖ, pₖ) → (qₖ₊₁, pₖ₊₁) is **exactly symplectic** — not approximately, not up to truncation error, but exactly. It preserves a discrete symplectic form Ωd = dqₖ ∧ dpₖ with no accumulated error.

**In PhysicLaw**, the Velocity Verlet integrator is the realization of DVM with the midpoint-rule discrete Lagrangian:

```
L_d(qₖ, qₖ₊₁) = h · L((qₖ + qₖ₊₁)/2, (qₖ₊₁ − qₖ)/h)
```

which yields the classical Störmer-Verlet update in half-kick/drift/half-kick form. The force computation — `Fᵢ = −∂V/∂qᵢ` obtained by seeding `q[i].der = 1.0` through the `LawRegistry` — is the engine's realization of the variational derivative −D₁V. Every law must provide a potential V(q); the engine differentiates it automatically via dual number arithmetic.

**This is a genuine advance over Newtonian simulation.** Energy error is bounded (not growing). Momentum maps are preserved to machine precision for exact symmetries. The physics is specified correctly: as energy, not force.

**But it is not the destination.**

---

## 3. The Deeper Problem: Discretization Is Not Neutral

Discrete Variational Mechanics is a structurally superior bridge between the continuous universe and the digital machine. But it is still a bridge — and Δt is still a pillar of that bridge. Δt remains a free parameter of the simulation: a computational artifact injected into the model, not derived from physics.

Consider what Δt actually means:

- It is the **minimum time scale** resolvable by the engine.
- Events occurring on timescales shorter than Δt are invisible by construction.
- The simulation's resolution in time is set by an engineering decision, not by the physics.

Even symplectic integrators cannot escape the consequences of this:

**Phase error accumulation.** A symplectic integrator preserves the symplectic form exactly, but it does *not* follow the exact trajectory. By backward error analysis, a p-th order symplectic integrator follows the exact trajectory of a nearby "shadow Hamiltonian":

```
H̃ = H + Δt^p · H_p(q, p) + O(Δt^{p+1})
```

For Velocity Verlet, p = 2. This means two simulations of the same system with different Δt will have trajectories that diverge — not catastrophically, but persistently and irreversibly. For systems where the *phase* of a trajectory matters (not just energy), this is a hard limitation.

**Numerical viscosity.** In an SPH fluid simulation, the effective viscosity of a symplectic integrator depends on Δt even when physical viscosity is set to zero. This numerical viscosity is a Δt-dependent artifact, not a property of the fluid. It is not a bug — it is an inherent consequence of temporal sampling.

**The CFL condition as symptom.** The requirement Δt < h/v_max is not a physical law. It is the engine telling us: *"my temporal resolution is insufficient to track this particle."* The correct response is not to decrease Δt — it is to ask why we are using a temporal resolution at all. The continuous physics has no such condition.

**Representational limits.** With any fixed Δt, the simulation cannot represent the full topology of continuous trajectories. A system undergoing infinitely many impacts in finite time (Zeno-type behavior, which occurs in rigid body contact mechanics) requires an infinite number of Δt steps to represent faithfully. The discrete simulation approximates it with a finite — and Δt-dependent — number of impacts.

The question PhysicLaw poses to itself is not "how do we make Δt smaller?" It is more fundamental:

> **Can we represent physics in a form where Δt is not a parameter?**

---

## 4. The Vision: Toward Timeless Physics

The research agenda of PhysicLaw is the development of simulation architectures that transcend Δt as a computational primitive. This is the intellectual horizon that orients every major design decision. Three technical directions are under active conceptual development:

### 4.1 Event-Driven Simulation

Instead of advancing the state at fixed time intervals, an event-driven engine:

1. Maintains a **priority queue of the next event** for each pair of interacting entities (collision time, phase boundary crossing, constraint activation).
2. Advances simulation time to the next event t* — **exactly**, not approximately.
3. Resolves the event (applies the exact state change) at t*.
4. Recomputes next events for all affected entities.

For hard-sphere systems, this is the oldest form of molecular dynamics (Alder & Wainwright, 1959) and it is *exact* in the collision geometry — no tunneling is possible because collision times are computed analytically. The simulation has no "frame rate" in the temporal sense; it has an *event rate* determined by the physics, not by a clock.

The challenge for PhysicLaw: the continuous force fields of the LawRegistry (gravity, spring potentials, SPH pressure) do not produce discrete events. The research question is how to hybridize event-driven accuracy at topological changes (collisions, contact, phase boundaries) with variational integration for smooth force fields — and what the combined architecture's geometric properties are.

### 4.2 Generating Function Representation

The exact flow of a Hamiltonian system, Φ_t: T*Q → T*Q, is a symplectic map for every t ≥ 0. Every symplectic map (locally) admits a **generating function** S(q₀, q₁; t) such that:

```
p₀ = −∂S/∂q₀    (initial momentum recovered from generating function)
p₁ = +∂S/∂q₁    (final momentum recovered from generating function)
```

The **Hamilton-Jacobi equation** governs how S evolves with t:

```
∂S/∂t + H(q₁, ∂S/∂q₁) = 0,    S(q, q; 0) = 0
```

If S can be evaluated (exactly or approximately in some function class), then advancing the simulation to any time t requires only solving two equations for q₁ given (q₀, p₀, t). No stepping. No accumulated phase error. The simulation "jumps" to t directly.

This is the foundation of Feng Kang's (1984) generating function methods for symplectic integration and of modern high-order methods via backward error analysis. The research question for PhysicLaw: can the generating function for a **composition of laws** from the LawRegistry be represented tractably — even approximately — for practically useful potential classes?

### 4.3 Relational and Timeless Mechanics

The most conceptually radical direction is motivated by a foundational question: in the Lagrangian L(q, q̇), what *is* t?

In Newtonian mechanics, t is an absolute external parameter — the clock ticks independent of the system. In a **closed** system, however, there is no external clock: time must be defined by the *change in configuration* of the system itself. This is the core insight of **Mach's principle** and its modern formalization in **Shape Dynamics** (Barbour, Gryb, Koslowski) and **Jacobi's principle**.

Jacobi's reformulation of mechanics eliminates time as a parameter entirely. The physical trajectory is the geodesic in Q with respect to a metric defined by the Jacobi metric:

```
ds² = 2(E − V(q)) · T(dq)
```

where E is the total energy. The parameter along the geodesic is an arc length in Q, not time in ℝ. Time is *recovered* from the geodesic as:

```
t = ∫ ds / √(2(E − V(q)))
```

In this picture, the simulation's fundamental operation would be `advance(q, p, δq)` — advance until the configuration has moved by a prescribed amount in Q — and time would be an *output* of the computation, not an *input*.

For PhysicLaw, this suggests a long-term architecture where:
- The simulation state is a curve in Q, not a sequence of (q, p) pairs indexed by t.
- Physical predictions (in terms of t) are fully recoverable by arc-length integration.
- The "frame rate" is a rendering concern, not a physics concern.

These three directions are not mutually exclusive. They may converge on a single architecture that unifies event detection, generating function evaluation, and geodesic advance. **They are the open questions that define the research frontier of this project.**

---

## 5. The Law-Centric Principle

Every interaction in PhysicLaw must be expressed as a **potential energy function** V: Q → ℝ. Never as a force directly.

This is not a stylistic preference. It is a structural guarantee with three consequences:

**Conservation is automatic.** A force F(q) is conservative if and only if F = −∇V for some scalar field V. By requiring every `Law` to implement `potential(q: &[Dual], mass: &[f64]) -> Dual`, the engine ensures this condition by construction. There is no way to add a non-conservative force through the Law interface.

**AD computes gradients exactly.** The engine differentiates V(q) via **forward-mode automatic differentiation** using dual numbers: numbers of the form a + bε where ε² = 0. The arithmetic rules — (a + bε)(c + dε) = ac + (ad + bc)ε — propagate derivatives exactly through arbitrary compositions of arithmetic operations. To compute ∂V/∂qᵢ:

```rust
q[i].der = 1.0;                          // seed the i-th coordinate
let v = laws.potential(&q, &mass);       // dual arithmetic propagates
let force_i = -v.der;                    // ∂V/∂qᵢ, exact to f64 precision
q[i].der = 0.0;                          // restore
```

The chain rule is enforced by the dual arithmetic — not by human bookkeeping, not by finite differences, not by symbolic expansion. The result is exact to machine precision.

**Laws compose by superposition.** The `LawRegistry` implements:

```
V_total(q) = V₁(q) + V₂(q) + ... + Vₙ(q)
```

Each law is independent and composable. Adding a new physical interaction requires implementing one method. The integrator, the energy probe, and the constraint system are all unaware of which laws are active — they interact only through the scalar V_total. This is the correct encapsulation boundary for physics.

### Noether Enforcement at the Interface Level

By Noether's theorem, if V is invariant under a spatial symmetry (e.g., translation invariance: V(q + δ) = V(q) for all δ), then the corresponding momentum is conserved. If every law implements V correctly, this conservation is a theorem — not an empirical observation subject to tuning.

Any contribution that adds a "force" without a potential **breaks this guarantee** and silently corrupts the energy balance of any simulation that includes it. The Law interface exists precisely to prevent this.

The invariant is simple: if you cannot write it as V(q), it does not belong in a Law. If the interaction is genuinely dissipative (viscosity, damping, contact friction), it belongs in a `Constraint` — and the energy dissipation should be explicitly tracked, not silently injected.

---

## 6. Contributor Invariants

These constraints are derived from the foregoing analysis. They are non-negotiable because violating them does not merely introduce inaccuracy — it destroys the structural properties that make PhysicLaw different from a conventional engine.

**I. All interactions are potentials.**
Every `Law` implementation provides `potential(&self, q: &[Dual], mass: &[f64]) -> Dual`. There are no exceptions. If you want to add a force, define the potential it derives from first. If no such potential exists, the force is non-conservative and must not be added as a `Law`.

**II. All integrators must be symplectic.**
New integrators must preserve the symplectic 2-form ω = Σ dpᵢ ∧ dqᵢ. The criterion: the Jacobian J of the one-step map (qₖ, pₖ) → (qₖ₊₁, pₖ₊₁) must satisfy J^T Ω J = Ω, where Ω is the standard symplectic matrix. Equivalently, the method must be derivable from a discrete Lagrangian L_d via the DEL equations. An integrator that is merely stable but not symplectic introduces secular energy drift and is unacceptable for long-horizon simulations.

**III. Rotation lives on SO(3), not ℝ³.**
Angular state must be updated using `SO3::retract(rot, ω·dt)` — the manifold-aware exponential map — not by adding Euler angle increments. Euler angle parameterization has coordinate singularities (gimbal lock) and the angle increments do not form a group under addition. The retract operation ensures the rotation matrix remains in SO(3) exactly at every step, with no normalization required.

**IV. Constraints are projections, not forces.**
The constraint system projects the integrated state onto the constraint manifold after each step. This is a geometric projection — it should minimize the perturbation to the variational trajectory while satisfying C(q) = 0. Adding a "restoring force" that fights a constraint is a sign that the constraint and a law are computing against each other; the system will oscillate and may not converge.

**V. Δt-dependence in a law is a defect.**
If a law's behavior changes as Δt changes (holding physical parameters fixed), it is expressing a numerical artifact, not physics. Test this explicitly: does your potential V(q) contain dt anywhere? If so, rewrite it. The potential V must depend only on configuration q and physical constants.

**VI. Design for replaceability of the integrator.**
Every API decision must accommodate the eventual replacement of the time-stepping integrator with an event-driven or generating-function evaluator. Does your law assume the system is evaluated at uniform Δt intervals? Does your constraint assume a specific ordering of the integration substeps? If so, you are encoding the Δt paradigm into an interface that should be integrator-agnostic. Design for generality — the timeless architecture is not hypothetical, it is the target.

---

*This document is the intellectual foundation of PhysicLaw. The code is its current, necessarily imperfect approximation.*
