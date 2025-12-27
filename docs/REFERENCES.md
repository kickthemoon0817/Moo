# PhysicLaw: Comprehensive Reading List

This document bridges the gap between **Geometric Mechanics** (Academic) and **Real-time Physics Engine Architecture** (Practical). It serves as a guide for understanding the "Law-Centric" philosophy of PhysicLaw.

## Foundations: Geometric & Variational Mechanics
*The theoretical backbone of PhysicLaw. Why symplectic integrators matter.*

*   **[Paper] Discrete Variational Mechanics** (Marsden & West, 2001)
    *   *The Holy Grail*. Defines how to discretize the Lagrangian directly to preserve energy and momentum naturally.
    *   [Link (Caltech)](https://authors.library.caltech.edu/2372/1/MaWe2001.pdf)
*   **[Book] Geometric Numerical Integration** (Hairer, Lubich, Wanner)
    *   The standard reference for structure-preserving algorithms. Covers Symplectic Euler, Verlet, and why Runge-Kutta often fails for long-term orbital stability.
*   **[Article] The Symplectic Expectation**
    *   Explains why energy drift is unavoidable in standard integrators and how symplectic methods bound this error globally.

## Continuum Mechanics: Fluids & Fields
*From Navier-Stokes to particle-based estimation.*

*   **[Paper] Smoothed Particle Hydrodynamics** (Monaghan, 1992)
    *   The foundational review of SPH. Essential for understanding the density summation $\rho = \sum m W$ and gradients.
*   **[Paper] Particle-Based Fluid Simulation for Interactive Applications** (Müller et al.)
    *   Introduces the "Poly6" and "Spiky" kernels used in PhysicLaw's `pl-continuum`.
    *   [Link](https://matthias-research.github.io/pages/publications/sca03.pdf)
*   **[Blog] Ten Minute Physics** (Matthias Müller)
    *   Excellent, digestible breakdown of PBD (Position Based Dynamics) and XPBD, offering a contrast to our variational approach.
    *   [YouTube Channel](https://www.youtube.com/c/TenMinutePhysics)

## Rigid Body Dynamics & Constaints
*Handling rotation (SO3) and collisions without exploding.*

*   **[Paper] Lie Group Variational Integrators** (Lee, 2009)
    *   How to integrate rotation matrices/quaternions strictly on the manifold, avoiding normalization drift.
*   **[GDC Talk] Iterative Dynamics with Temporal Coherence** (Erin Catto / Box2D)
    *   The industry standard for solving constraints (sequential impulses). While PhysicLaw uses variational potentials, understanding the Jacobian / Impulse method is crucial for implementing `Constraint::project`.
    *   [Slides](https://box2d.org/files/ErinCatto_SequentialImpulses_GDC2006.pdf)
*   **[Blog] Gaffer on Games: Physics in 3D** (Glenn Fiedler)
    *   The definitive guide for network-synced physics and stable RK4/Verlet implementations in C++.
    *   [Link](https://gafferongames.com/categories/game-physics/)

## Practical Engineering & Architecture
*How to build a physics engine that actually runs.*

*   **[Book] Game Physics Engine Development** (Ian Millington)
    *   Practical C++ architecture. Covers "Broadphase" vs "Narrowphase" collision detection.
*   **[Blog] Wicked Engine Devlog**
    *   Modern rendering and compute shader physics implementation details, often using DirectX/HLSL (translatable to WGSL).
*   **[Resource] Real-Time Collision Detection** (Ericson)
    *   The bible of intersection tests (Sphere-Sphere, AABB-Triangle, SAT).

## Rust & GPGPU (Implementation)
*   **[Guide] Learn WGPU**
    *   Essential for understanding the `pl-viz` and `pl-compute` backend.
    *   [Link](https://sotrh.github.io/learn-wgpu/)
*   **[Crate] Glam**
    *   The linear algebra library used in PhysicLaw. Optimized for SIMD.
