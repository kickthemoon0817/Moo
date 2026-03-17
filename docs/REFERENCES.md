# PhysicLaw: Reading Curriculum

This document is organized as a **self-study curriculum**, not a flat bibliography. The tracks are ordered by dependency: each track assumes familiarity with the preceding ones. A contributor joining PhysicLaw should read them in the order listed.

The final track — **Research Frontier** — points toward what PhysicLaw intends to *develop*, not import. These references describe methods and ideas that are not yet implemented, but that define the intellectual horizon of the project.

---

## Track 1: Prerequisites — Classical Mechanics

These are the foundational texts. You need a working understanding of Lagrangian and Hamiltonian mechanics before the geometric and discrete tracks will make sense.

- **[Book] *Classical Mechanics*** — Herbert Goldstein, Charles Poole, John Safko (3rd ed.)
  - The standard graduate reference. Chapters 1–9 cover generalized coordinates, Lagrangian and Hamiltonian mechanics, canonical transformations, and Hamilton-Jacobi theory.
  - Read before anything else.

- **[Book] *Mechanics*** — Landau & Lifshitz (Course of Theoretical Physics, Vol. 1)
  - Concise, brutal, and correct. The derivation of the Euler-Lagrange equations from the principle of stationary action in §2 is canonical.
  - Particularly valuable for its treatment of symmetry and conservation laws (§6–7) and small oscillations (§21–23).

- **[Book] *Mathematical Methods of Classical Mechanics*** — V.I. Arnold (2nd ed.)
  - Arnold's treatment is the correct modern one: mechanics as geometry. Symplectic manifolds, Lagrangian submanifolds, and canonical transformations are developed rigorously.
  - The first half (Newtonian and Lagrangian mechanics) should be read at this stage; the symplectic geometry chapters belong to Track 2.

---

## Track 2: Geometric Mechanics — The Theoretical Backbone

These texts formalize what PhysicLaw is actually built on: mechanics as geometry on smooth manifolds. Do not implement anything in PhysicLaw without having at least skimmed the relevant sections.

- **[Book] *Foundations of Mechanics*** — Ralph Abraham & Jerrold Marsden (2nd ed.)
  - The definitive treatment of symplectic geometry and Hamiltonian mechanics on manifolds. Symplectic forms, Poisson brackets, momentum maps, and symplectic reduction are all developed from first principles.
  - Heavy reading. Chapters 3–5 (symplectic manifolds, Hamiltonian systems) are essential; the rest is reference.

- **[Book] *Introduction to Mechanics and Symmetry*** — J.E. Marsden & T.S. Ratiu (2nd ed.)
  - More accessible than Abraham & Marsden. The treatment of Noether's theorem (Chapter 11) and momentum maps (Chapter 12) is the one to internalize.
  - The Lie group chapters (13–15) are directly relevant to the SO(3) rigid body implementation.

- **[Book] *Mathematical Methods of Classical Mechanics*** — V.I. Arnold (2nd ed., Part 2)
  - Arnold's chapters on symplectic manifolds (Chapter 8) and canonical transformations (Chapter 9) complement the Marsden texts with a more geometric, less coordinate-heavy perspective.

- **[Paper] "Symplectic Geometry"** — Alan Weinstein (1981), *Bulletin of the AMS*
  - A short, readable survey of symplectic geometry that clarifies what "symplectic" actually means and why it matters for mechanics. Good entry point before the heavier books.

---

## Track 3: Discrete Methods — Current Implementation Basis

These are the papers and books that directly underpin what is currently in `moo/src/core/solve/`. Understanding them is required before modifying or extending the integration layer.

> **Note on limitations:** The methods in this track are the best currently-known structure-preserving discretizations of Hamiltonian mechanics. They remain fundamentally Δt-parameterized approximations. They preserve the symplectic form exactly but do *not* follow the exact continuous trajectory — they follow the trajectory of a nearby "shadow Hamiltonian." This distinction is not pedantic; it matters for phase-sensitive applications and for understanding the research agenda in Track 5.

- **[Paper] "Discrete Mechanics and Variational Integrators"** — Marsden & West (2001)
  - *[Caltech Library](https://authors.library.caltech.edu/2372/1/MaWe2001.pdf)*
  - The foundational paper for Discrete Variational Mechanics. Derives symplectic integrators (including Verlet) from a discrete Lagrangian and discrete Hamilton's principle. This is the theoretical justification for PhysicLaw's `VelocityVerlet` integrator.
  - Read Sections 1–3 for the core theory; Section 4 for discrete Noether's theorem.

- **[Book] *Geometric Numerical Integration*** — Ernst Hairer, Christian Lubich, Gerhard Wanner (2nd ed.)
  - The standard reference for structure-preserving algorithms. Chapter IV (symplectic integrators), Chapter VI (backward error analysis), and Chapter IX (long-term behavior) are directly relevant.
  - The backward error analysis in Chapter IX explains why symplectic integrators conserve a *shadow Hamiltonian* rather than the true one — and what this means for phase error.

- **[Paper] "Lie Group Variational Integrators for the Full Body Problem"** — T. Lee et al. (2007/2009)
  - How to integrate on Lie groups (specifically SO(3) for rigid bodies) using a variational approach. The current SO(3) retract in PhysicLaw is a first approximation; this paper shows the full structure-preserving treatment.
  - Directly relevant to Section 8.3 of CONCEPTS.md.

- **[Paper] "RATTLE: A 'velocity' version of the SHAKE algorithm for molecular dynamics calculations"** — Andersen (1983), *J. Computational Physics*
  - The standard reference for constrained Hamiltonian dynamics (SHAKE/RATTLE). Relevant to understanding how constraints should interact with symplectic integration — and why PhysicLaw's post-step projection approach is an approximation.

---

## Track 4: Continuum Mechanics — Fluids and Fields

Required reading for understanding the SPH fluid solver and the Phase 18 continuum targets.

- **[Paper] "Smoothed Particle Hydrodynamics"** — J.J. Monaghan (1992), *Annual Review of Astronomy and Astrophysics*
  - The foundational review of SPH. Essential for understanding the density summation ρ = Σ m W and the kernel gradient approximations used for pressure forces.

- **[Paper] "Particle-Based Fluid Simulation for Interactive Applications"** — Müller, Charypar, Gross (2003)
  - *[Author page](https://matthias-research.github.io/pages/publications/sca03.pdf)*
  - Introduces the Poly6 and Spiky kernels used directly in PhysicLaw's `sph.wgsl`. The WCSPH pressure model (Tait equation) is described here.

- **[Paper] "Variational and momentum preservation aspects of smooth particle hydrodynamic formulations"** — Bonet & Kulasegaram (2000), *Computer Methods in Applied Mechanics and Engineering*
  - Shows that SPH can be derived from a variational principle, yielding a symmetrized pressure force that conserves momentum exactly. Relevant to the open question of whether PhysicLaw's SPH solver can be made variational (CONCEPTS.md Section 8.5).

- **[Blog] Ten Minute Physics** — Matthias Müller (YouTube)
  - Digestible breakdown of PBD and XPBD. Useful for understanding the constraint-based approach that PhysicLaw deliberately differs from.

---

## Track 5: Research Frontier — Where PhysicLaw Intends to Go

These references describe the ideas and methods that are *not yet implemented* in PhysicLaw but that define the intellectual direction of Phase 18+ development. They correspond to the "timeless physics" vision in PHILOSOPHY.md.

> This track is for contributors who want to participate in the research program, not just the engineering. The papers here do not describe techniques ready to implement — they describe open problems and directions.

### 5.1 Event-Driven Simulation

- **[Paper] "Studies in Molecular Dynamics. I. General Method"** — B.J. Alder & T.E. Wainwright (1959), *Journal of Chemical Physics*
  - The first molecular dynamics simulation. Used an event-driven (collision-by-collision) approach for hard spheres — exact collision geometry, no time-step artifacts. The original existence proof that event-driven simulation is viable.

- **[Paper] "Simulating Billiards Is Hard"** — B. Lubachevsky (1991), *Journal of Computational Physics*
  - Shows that even for hard-sphere event-driven MD, the scheduling problem has computational subtleties. Relevant for understanding the algorithmic complexity of extending event-driven methods to soft potentials.

### 5.2 Generating Functions for Symplectic Maps

- **[Paper] "The symplectic methods for the computation of Hamiltonian equations"** — Feng Kang (1984/1985)
  - The original paper introducing symplectic integrators based on generating functions. Feng Kang independently discovered symplectic integration and grounded it in the generating function formulation of canonical transformations.
  - Harder to obtain; the ideas are well-summarized in Hairer et al. Chapter VI.

- **[Paper] "Generating functions for the solution of separable Hamiltonian equations"** — R. de Vogelaere (1956), *University of Notre Dame Report*
  - Historical precursor to Feng Kang. Shows how generating functions naturally yield symplectic maps. Context for understanding what PhysicLaw would need to implement a generating-function evaluator.

### 5.3 Timeless and Relational Mechanics

- **[Book] *The End of Time: The Next Revolution in Physics*** — Julian Barbour (1999)
  - Barbour's case that time is not fundamental — it is derived from the ratio of change in different physical systems. Develops the Machian / relational alternative to Newtonian absolute time.
  - Not a technical reference: a conceptual and philosophical one. Read Chapter 1–3 for the central argument before diving into the technical papers.

- **[Paper] "Mach's principle and the structure of dynamical theories"** — Barbour & Bertotti (1982), *Proceedings of the Royal Society*
  - The technical formulation of relational mechanics (Barbour-Bertotti theory). Shows how to construct a Machian mechanics in which the time parameter is eliminated by a gauge choice. The direct precursor to Shape Dynamics.

- **[Paper] "Shape Dynamics: An Introduction"** — H. Gomes (2011), *arXiv:1109.5178*
  - Modern formulation of timeless mechanics. Shape dynamics recasts GR (and classical mechanics) as dynamics on the space of 3-geometries, with no preferred time foliation. The analog for the N-body problem is directly applicable to PhysicLaw's particle simulation.

- **[Book] *Mathematical Aspects of Classical and Celestial Mechanics*** — Arnold, Kozlov, Neishtadt (3rd ed.), Chapter 5 (Integrable Systems and Jacobi's Theorem)
  - Jacobi's principle: classical mechanics without an external time parameter, reformulated as geodesic flow on Q with the Jacobi metric. The rigorous version of the timeless simulation idea.

### 5.4 Discrete Differential Geometry (Exact Geometric Algorithms)

- **[Course Notes] "Discrete Differential Geometry: An Applied Introduction"** — Keenan Crane et al.
  - *[Available at cs.cmu.edu/~kmcrane](https://www.cs.cmu.edu/~kmcrane/Projects/DDG/)*
  - DDG provides exact discrete analogs of continuous geometric objects (curvature, parallel transport, exterior calculus). Relevant for implementing exact geometric collision detection and constraint projection on manifolds — without the usual numerical drift.

### 5.5 High-Order Lie Group Integrators

- **[Paper] "Runge-Kutta methods on Lie groups"** — H. Munthe-Kaas (1998), *BIT Numerical Mathematics*
  - Extends Runge-Kutta methods to Lie group configuration spaces. The foundation for spectrally-accurate integration of the SO(3) rigid body DOFs (CONCEPTS.md Section 8.4).

- **[Paper] "On the solution of linear differential equations in Lie groups"** — A. Iserles & S.P. Nørsett (1999), *Philosophical Transactions of the Royal Society*
  - Magnus expansion methods for Lie group ODEs. Provides error analysis for the SO3::retract splitting approach and shows how to improve it systematically.

### 5.6 Variational SPH

- **[Paper] "Smoothed dissipative particle dynamics"** — P. Español & M. Revenga (2003), *Physical Review E*
  - Derives SPH from a variational principle (GENERIC framework). Shows that variational SPH conserves angular momentum exactly — a property that standard SPH formulations do not satisfy. Key reference for CONCEPTS.md Section 8.5.

---

## Track 6: Implementation — Engineering the Engine

These are practical engineering references for the GPU compute, collision detection, and Rust/WGPU layers.

- **[GDC Talk] "Iterative Dynamics with Temporal Coherence"** — Erin Catto / Box2D (2006)
  - *[Slides](https://box2d.org/files/ErinCatto_SequentialImpulses_GDC2006.pdf)*
  - The industry standard for constraint solving via sequential impulses (Jacobian / impulse method). PhysicLaw uses variational potentials instead of impulses, but understanding this approach is essential for implementing `Constraint::project` correctly.

- **[Blog] Gaffer on Games: Physics for Game Developers** — Glenn Fiedler
  - *[Link](https://gafferongames.com/categories/game-physics/)*
  - Definitive guide for network-synced physics, stable integrator implementation in C++, and practical physics engine architecture. Translates well to Rust.

- **[Book] *Real-Time Collision Detection*** — Christer Ericson
  - The reference for intersection tests: Sphere-Sphere, AABB-Triangle, SAT (Separating Axis Theorem), GJK. Required reading for any contributor extending the collision detection layer.

- **[Book] *Game Physics Engine Development*** — Ian Millington
  - Practical C++ architecture. Covers broadphase vs. narrowphase collision, contact resolution, and constraint systems. Read alongside Ericson.

- **[Guide] Learn WGPU** — *[sotrh.github.io/learn-wgpu](https://sotrh.github.io/learn-wgpu/)*
  - Essential for understanding the `khe` visualization backend and `moo`'s `ComputeEngine`.

- **[Blog] Wicked Engine Devlog**
  - Modern GPU compute physics implementation details (DirectX/HLSL, translatable to WGSL). Spatial hashing, parallel reduction, and GPU-resident simulation patterns.

- **[Crate] Glam** — *[docs.rs/glam](https://docs.rs/glam/)*
  - The linear algebra library used throughout PhysicLaw. SIMD-accelerated Vec2/Vec3/Vec4/Quat/Mat4. Read the Quat documentation carefully before working with the SO(3) rigid body code.

---

## Study Path Recommendations

**New contributor, physics background:**
Track 1 → Track 2 → Track 3 → Read PHILOSOPHY.md → Track 5 (selective)

**New contributor, engineering/CS background:**
Track 1 (Goldstein chapters 1–3) → Track 3 → Track 6 → CONCEPTS.md → Track 2 (as needed)

**Researcher joining the frontier work:**
Tracks 1–3 (full) → Track 5 (full) → Track 4 (5.6 specifically) → Design discussions

**Contributor adding a new Law:**
Track 1 (Lagrangian mechanics sections) → Track 3 (Marsden & West §1–3) → PHILOSOPHY.md §5 → CONCEPTS.md §1–2
