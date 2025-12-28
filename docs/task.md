# Task: PhysicLaw Development Roadmap

## Phase 1: Foundational Design & Structuring [x]
- [x] Research limitations of existing engines
- [x] Adopt "Refined Repository Structure" (Modules inside `moo/src` for single-directory simplicity)
- [x] Define "Investigation" vs "Exposition" philosophy

## Phase 2: Core Engineering [/]
- [x] Design foundation (`pl-geometry` and `pl-math`) & Integrate into `moo` crate
- [x] Implement `pl-state` Phase Space container with SIMD alignment
- [x] Implement `pl-solve` Symplectic Integrator kernal

## Phase 3: Laws & Composition [/]
- [x] Develop `pl-registry` for Lagrangian summation
- [/] Implement foundational `pl-classical` (Rigid body) and `pl-continuum` (Fluids)
      - [x] Gravity (N-body)
      - [x] Spring (Harmonic)
      - [x] Rigid Body Dynamics (SO(3) Manifold + Euler Equations)

## Phase 3.5: Constraints & Collisions [x]
- [x] Define `Constraint` trait (Projections)
- [x] Implement `FloorConstraint` and `SphereConstraint` (Collision)
- [x] Integrate constraint solver into `VelocityVerlet`

## Phase 4: Investigation & Analysis [x]
- [x] Design `pl-probe` for real-time Fourier analysis & Noether checks
      - [x] Energy Probe
      - [x] Real-time Graphing (UI Overlay)
- [x] Implement `pl-viz` scientific renderer (WGPU)
      - [x] Particles (Soft Circles)
      - [x] Rigid Bodies (Vector Frames/Axes)
## Phase 5: Continuum Laws (Fluids) [x]
- [x] Implement `pl-continuum` SPH (Smoothed Particle Hydrodynamics)
      - [x] Implement Density Kernel (Poly6)
      - [x] Implement Lagrangian Pressure Potential (Equation of State)
      - [ ] Implement Viscosity (Non-Conservative Force or Constraint) -- Defer to Refinement Phase

## Phase 6: High-Performance Computing (GPGPU) [x]
- [x] Design `pl-compute` module (Scaffolding Complete)
- [x] Implement Compute Shader for N-Body Gravity (`nbody.wgsl`)
      - [x] Implement ComputeEngine and Pipeline
      - [x] Interop with `pl-viz` (Shared Buffers to avoid CPU-GPU copy)
- [x] Implement Compute Shader for N-Body Gravity (`nbody.wgsl`)
      - [x] Implement ComputeEngine and Pipeline
      - [x] Interop with `pl-viz` (Shared Buffers to avoid CPU-GPU copy)

## Phase 7: GPU Fluid Dynamics (SPH)
- [x] Implement `sph.wgsl` (Compute Shader)
      - [x] Kernel 1: Density Calculation (Poly6)
      - [x] Kernel 2: Pressure Force & Integration (Spiky + Symplectic Euler)
- [x] Update `ComputeEngine` for Multi-Pass Support
      - [x] Add `density_buffer`
      - [x] Implement `step_sph()` with multi-dispatch
- [ ] Integration & Tuning
      - [ ] Tune Smoothing Length ($h$) and Rest Density ($\rho_0$) for 100 particles

## Verification [x]
- [x] Benchmark numerical stability (Energy conservation < 1e-2 verified with VelocityVerlet)
- [x] Verify `SphereConstraint` logic (Fixed singularity handling in `dev/0.0.3`)
- [x] Validate `PhaseSpace` Mass Layout (Fixed per-DOF usage in tests `dev/0.0.3`)

## Refinement (dev/0.0.3) [x]
- [x] Fix Window Panic (Empty Rigid Body list)
- [x] Fix Renderer Safety (Transmute removed, Arc<Window> used)
- [x] Fix Gravity/Constraint Singularities (Softening)
- [x] Validate against analytical solutions (Orbits verified)
- [x] Fix SPH Race Condition (Split Density/Force steps into separate Compute Passes)

## Phase 8: SPH Refinement (Viscosity & Tuning) [x]
- [x] Tune Parameters
      - [x] Set `rho0` to 0.01 (match Spacing=10.0, Mass=1.0)
      - [x] Tune `stiffness` and `h`
- [x] Implement Viscosity
      - [x] Add `viscosity` coefficient to `SimParams`
      - [x] Implement Viscosity Force in `sph.wgsl` (Laplacian smoothing)

## Phase 9: Optimization (Spatial Grid) [x]
- [x] Implement Spatial Indexing ($O(N)$)
      - [x] `sort.wgsl`: Bitonic Sort
      - [x] `grid.wgsl`: Grid Index & Offsets
      - [x] Update `ComputeEngine`: Buffers & Dispatch
      - [x] `sph.wgsl`: Neighbor Search using Grid

## Phase 10: Advanced Rendering (Liquid) [x]
- [x] Implement Screen Space Fluids (SSF) / Lit Sprites
      - [x] Point Sprites (Billboard Quads)
      - [x] Lit Sphere Shading (Normal + Specular)
      - [x] Camera Implementation (Orthographic)

## Phase 11: Application Architecture & GUI [x]
- [x] Refactor Architecture for Dual Mode (App vs Script)
      - [x] Extract `Simulation` struct (Headless Engine)
      - [x] Decouple `Renderer` from `EventLoop` logic
- [x] Implement Interactive GUI (`egui`)
      - [x] Add `Gui` overlay
      - [x] Controls: Pause/Play, Reset, Parameters (dt, Gravity)
      - [x] Stats: FPS, Particle Count

## Phase 12: Real-time Interaction (Mouse) [x]
- [x] Implement Mouse Picking
      - [x] Deproject Mouse Coordinates to World Space
      - [x] Spatial Query (GPU-based Interaction)
- [x] Implement Particle Interaction
      - [x] "Grab and Throw" Physics (Spring Force in `sph.wgsl`)

## Phase 13: Architecture Split (Moo & Khe) [x]
- [x] Create `Khe` (Visualization App)
      - [x] Setup Cargo Workspace
      - [x] Move `renderer.rs`, `window.rs`, and shaders to `Khe`
- [x] Purify `Moo` (Physics Engine)
      - [x] Remove GUI/Window dependencies
      - [x] Export `Simulation`, `ComputeEngine`, `PhaseSpace`
      
## Phase 14: Maintenance & Upgrade (v0.0.4) [x]
- [x] Create `dev/0.0.4` branch
- [x] Upgrade deps (`wgpu` 27, `egui` 0.33, `winit` 0.30)
- [x] Migrate to Rust 2024
- [x] Fix breaking API changes (Pipelines, Lifetimes)
- [x] Commit and Push

## Phase 15: Web Assembly (WASM) Support [x]
- [x] Dependency Fixes
      - [x] Add `wasm-bindgen`, `web-sys`, `console_error_panic_hook`
      - [x] Enable `getrandom/js` feature
- [x] Async Refinement
      - [x] Remove `pollster::block_on` (Incompatible with Browser Main Thread)
      - [x] Implement `start` entry point for WASM
- [x] Web Infrastructure
      - [x] Create `index.html` with Canvas
      - [x] Configure `wasm-pack` build
- [x] Validation
      - [x] Build for `wasm32` (WebAssembly)
      - [x] Run in Browser (Local Server)
## Phase 16: Application Shell & Advanced Rendering
- [x] Application Architecture
      - [x] Implement Docking/Panel Layout (Viewport, Hierarchy, Inspector)
      - [x] Preferences/Settings Window
- [x] Viewport Implementation
      - [x] Refactor `Renderer` to support Offscreen Rendering (Render to Texture)
      - [x] Integrate Viewport Texture into `egui` (Image Widget)
      - [x] Handle Mouse Input Translation (Window -> Viewport Coordinates)
- [x] Automated Visual Verification
      - [x] Implement Texture Readback (GPU -> CPU)
      - [x] Add Screenshot Keybinding/Trigger
      - [x] Verify Output Image (Render Pipeline Verified. Physics Engine Pending Fix).
- [x] Maintenance
      - [x] cargo fmt & clippy
      - [x] git commit
- [ ] Debug Physics Kernel (Stability)
    - [ ] Fix SPH explosion (NaNs/Inf)
    - [ ] Tune `dt` and `spacing` parameters
    - [ ] Verify dynamic simulation
- [ ] Advanced Visuals (Inside Viewport)
      - [ ] Screen Space Fluid Rendering (SSFR) / Metaballs
      - [ ] Refraction & Reflection
      - [ ] Soft Shadows

## Phase 17: External Control & MCP Integration
- [ ] Architecture Design
    - [ ] Design Command Queue (Lock-free/Low-lock handoff)
    - [ ] Design Control API (`Moo` Library API)
    - [ ] Select Control Channel (gRPC / Cap'n Proto over TCP for High Performance)
- [ ] Core Library Implementation (`moo-physics`)
    - [ ] Implement `sim.start`, `sim.pause`, `sim.step`, `sim.set_params` commands
    - [ ] Implement `sim.get_state`, `sim.capture`, `sim.export` querys
    - [ ] Expose Control Channel in `Moo` Engine
- [ ] Scripting & Bindings (Light Programmatic Method)
    - [ ] Implement `moo-py`: Python bindings via `pyo3`
    - [ ] Direct Access Mode: Run Sim instance within Python process
    - [ ] Enable `import moo; sim = moo.Simulation()` workflow
- [ ] CLI Client (`khe-cli`)
    - [ ] Implement thin CLI wrapper using gRPC client
    - [ ] Commands: `start`, `attach`, `status`, `stop`
- [ ] MCP Server (`khemoo-mcp`)
    - [ ] Implement MCP Server (stdio-based)
    - [ ] Implement Tools: `khemoo.khe.run.*` (Lifecycle), `khemoo.moo.sim.*` (Control)
    - [ ] Implement Resources: `khemoo://runs/{id}/...` (State/Metrics)
- [ ] Skills & Usage
    - [ ] `khemoo-sim-control`: Runtime control of simulation
    - [ ] `khemoo-sim-analysis`: Metrics and Validation
    - [ ] `khemoo-scenarios`: Scenario Templating

## Phase 18: Advanced Non-Linear & Multi-Physics
- [ ] Goal: High-Fidelity Simulation of "Real World" complexities
- [ ] Advanced Continuum Mechanics
    - [ ] Implement Hyperelasticity (Neo-Hookean / Mooney-Rivlin) for soft bodies
    - [ ] Implement Plasticity & Failure models (von Mises / Drucker-Prager)
- [ ] Field Theories & Relativistic Effects
    - [ ] Implement Post-Newtonian Gravity corrections (General Relativity approximation)
    - [ ] Explore non-linear Electro-Magnetism (Maxwell-Vlasov solver)
- [ ] Multi-Physics Coupling
    - [ ] Fluid-Structure Interaction (FSI)
    - [ ] Thermal Diffusion & Convection
