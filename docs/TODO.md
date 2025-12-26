# PhysicLaw: Project Roadmap

Status: **Architectural Planning Phase Complete**

## High-Level Objectives
- [ ] Initialize Multi-crate Rust Workspace
- [ ] Implement `core/pl-geometry` and `core/pl-math` (Foundation)
- [ ] Implement `core/pl-solve` (Variational Integrator)
- [ ] Create the `investigation/pl-viz` scientific renderer

## Directory Structure to Implement
```text
/
├── core/
│   ├── pl-geometry (Manifolds, SE3/SO3)
│   ├── pl-math (AD, Tensors)
│   ├── pl-state (SoA state storage)
│   └── pl-solve (Symplectic solvers)
├── laws/
│   ├── pl-registry (Law composition)
│   └── pl-classical (Multi-body physics)
├── platform/
│   ├── pl-compute (GPGPU dispatch)
│   └── pl-storage (HDF5/Data IO)
└── investigation/
    ├── pl-probe (Symmetry checks)
    └── pl-viz (WGPU visualization)
```
