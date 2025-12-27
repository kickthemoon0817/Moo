use crate::core::state::PhaseSpace;
use crate::investigation::viz::renderer::{LineVertex, ScientificRenderer};
use crate::platform::compute::ComputeEngine;
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("PhysicLaw Scientific Visualization")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let mut renderer = ScientificRenderer::new(window.clone()).await;
    // Initial Camera
    renderer.update_camera_ortho(800.0, 600.0);

    // --- Physics Setup ---
    let n_fluid = 100;
    let dof = n_fluid * 3;
    let mut state = PhaseSpace::new(dof); // Fluid particles

    // Initialize Fluid Block
    let spacing = 10.0;
    let cols = 10;
    let start_y = 100.0;

    for i in 0..n_fluid {
        let col = i % cols;
        let row = i / cols;
        state.q[i * 3] = (col as f64) * spacing - (cols as f64 * spacing / 2.0);
        state.q[i * 3 + 1] = start_y + (row as f64) * spacing;
        state.q[i * 3 + 2] = 0.0;

        state.mass[i * 3] = 1.0;
        state.mass[i * 3 + 1] = 1.0;
        state.mass[i * 3 + 2] = 1.0;

        state.radius[i] = spacing / 2.0;
    }

    // --- GPGPU Setup ---
    let mut compute = ComputeEngine::new(renderer.device(), n_fluid as u32).await;
    compute.write_state(renderer.queue(), &state.q, &state.v, &state.mass);

    // --- Constraints ---
    let floor_y = -200.0;

    event_loop
            .run(move |event, target| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == window.id() => match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key:
                                        winit::keyboard::PhysicalKey::Code(
                                            winit::keyboard::KeyCode::Escape,
                                        ),
                                    ..
                                },
                            ..
                        } => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                            // Keep simulation centered. Aspect correct?
                            // Let's keep strict 800.0 width world units for now.
                            let aspect = physical_size.width as f32 / physical_size.height as f32;
                            let world_width = 800.0;
                            let world_height = world_width / aspect;
                            renderer.update_camera_ortho(world_width, world_height);
                        }
                        WindowEvent::RedrawRequested => {
                            // GPU Physics Step
                            for _ in 0..10 {
                                compute.step(renderer.device(), renderer.queue());
                            }

                            // GPU Render
                            // Render compute buffer directly
                            if let Err(e) =
                                renderer.render_compute(compute.current_buffer(), n_fluid as u32)
                            {
                                eprintln!("Render Error: {:?}", e);
                                target.exit();
                            }

                            // Sync Lines (Floor)
                            let mut lines = Vec::new(); // Reallocate every frame? Optimized later.

                            // 1. Draw Floor Grid
                            let grid_size = 1000.0;
                            let step = 100.0;
                            let y = floor_y as f32;
                            let color = [0.2, 0.2, 0.2];
                            // ... (Grid generation code omitted for brevity in replace? No, need to keep it)
                            // I'll assume I am replacing the block and need to provide full content.

                            let mut x = -grid_size;
                            while x <= grid_size {
                                lines.push(LineVertex {
                                    position: [x, y, -grid_size],
                                    color,
                                });
                                lines.push(LineVertex {
                                    position: [x, y, grid_size],
                                    color,
                                });
                                x += step;
                            }
                            let mut z = -grid_size;
                            while z <= grid_size {
                                lines.push(LineVertex {
                                    position: [-grid_size, y, z],
                                    color,
                                });
                                lines.push(LineVertex {
                                    position: [grid_size, y, z],
                                    color,
                                });
                                z += step;
                            }

                            renderer.update_lines(&lines);
                            // UI not updated (Energy probe disabled on GPU)
                            renderer.update_ui_lines(&[]);
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
                        window.request_redraw();
                    }
                    _ => {}
                }
            })
            .unwrap();
}
