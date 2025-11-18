
# 2D Rust Engine Detailed Plan

## Phase 0 — Project Setup & Goals
- [ ] Target macOS first: configure `cargo` toolchain, notarization signing requirements, and prefer Metal backend defaults in `wgpu`.
- [ ] Select crates: `winit` for window/input, `wgpu` for rendering, `rodio` for audio, `image` + `serde`/`ron` for assets.
- [ ] Scaffold repo layout (`src/engine/*`, `src/game/*`, `assets/`) and configure `cargo fmt`/`clippy`.
- [ ] Establish logging (`tracing`), config loading, and a profiling flag so later phases have observability.

## Phase 1 — Platform Skeleton (1–2 weekends)
- [ ] Create the `main.rs` entry that initializes logging, config, and an `Engine` struct.
- [ ] Wire a `winit` window, event loop, and centralized input mapper (keyboard + mouse).
- [ ] Implement frame timing utilities (fixed timestep accumulator, FPS counter).
- [ ] Deliverable: closing the window exits cleanly; console prints tick/render metrics.

## Phase 2 — Rendering Core (2–3 weeks)
- [ ] Integrate `wgpu`, request adapters/devices, and handle swapchain resize events.
- [ ] Render pipeline milestone ladder: triangle → textured quad → batched quads.
- [ ] Wrap GPU objects in a `Renderer` API (`begin_frame`, `draw_sprite`, `end_frame`) plus a 2D orthographic camera helper.
- [ ] Deliverable: multiple moving quads rendered with clear separation between engine/game code.

## Phase 3 — Resources & Sprites (2–4 weeks)
- [ ] Implement `ResourceManager` to load/cache textures, shaders, and sprite sheets (via `image`).
- [ ] Define `TextureHandle`, `Sprite`, and `Animation` structs with lifetime-safe ownership.
- [ ] Add sprite layers/z-order and simple animation playback driven by elapsed time.
- [ ] Deliverable: textured sprites with animations sourced from an atlas, reloaded without leaking GPU memory.

## Phase 4 — Scene / ECS Layer (3–4 weeks)
- [ ] Choose data model: lightweight ECS (ID allocator, component stores, system scheduler) or scene graph with parent/child transforms.
- [ ] Implement core components (Transform, Velocity, Sprite, Collider placeholder) and systems (movement, animation, render submission).
- [ ] Provide scripting hooks/state machine for game states (Menu, Playing, Paused).
- [ ] Deliverable: entities spawn/despawn, systems tick in deterministic order, and rendering pulls from ECS data.

## Phase 5 — Audio & UI Glue (1–2 weeks)
- [ ] Wrap `rodio` for background music + SFX channels with volume controls.
- [ ] Implement bitmap font or SDF text rendering for HUDs plus simple UI widgets (buttons, labels).
- [ ] Expose event hooks so gameplay code can trigger sounds/UI updates.
- [ ] Deliverable: interactive HUD (score, health) and contextual audio tied to gameplay events.

## Phase 6 — Proof-of-Concept Game (2–3 weeks)
- [ ] Select demo genre (top-down shooter or platformer) and lock content requirements (levels, enemies, win/lose conditions).
- [ ] Build level data pipeline (tilemap or JSON scenes) feeding into ECS/scene graph.
- [ ] Implement collision helpers, entity factories, projectiles, and simple AI.
- [ ] Deliverable: fully playable vertical slice showcasing every subsystem and recorded demo video.

## Phase 7 — Polish & Learning (ongoing)
- [ ] Add profiling overlays, asset packaging, and build scripts for macOS/Linux targets.
- [ ] Document architecture decisions and subsystem APIs in `docs/`.
- [ ] Identify gaps before moving to Unity/Unreal (editor tooling, 3D, networking) and queue future spikes.
