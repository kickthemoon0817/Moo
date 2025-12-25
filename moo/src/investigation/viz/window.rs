use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::investigation::viz::renderer::ScientificRenderer;
use crate::core::state::PhaseSpace;
use crate::core::solve::{Integrator, VelocityVerlet};
use crate::laws::registry::LawRegistry;
use crate::laws::classical::Gravity;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("PhysicLaw Scientific Visualization")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let mut renderer = ScientificRenderer::new(&window).await;

    // --- Physics Setup ---
    // 3-Body Problem (Figure-8 Stability or Random)
    // Let's do a simple stable-ish config
    let mut state = PhaseSpace::new(3 * 3); // 3 Particles, 3 DOF each
    
    // P1 (Center)
    state.q[0] = 0.0; state.q[1] = 0.0; state.q[2] = 0.0;
    state.mass[0] = 1000.0;
    
    // P2 (Orbiting)
    state.q[3] = 200.0; state.q[4] = 0.0; state.q[5] = 0.0;
    state.v[4] = 2.0; // Initial velocity Y
    state.mass[1] = 10.0;

    // P3 (Orbiting farther)
    state.q[6] = -300.0; state.q[7] = 100.0; state.q[8] = 0.0;
    state.v[7] = -1.5;
    state.mass[2] = 20.0;

    let mut registry = LawRegistry::new();
    registry.add(Gravity::new(100.0)); // G=100

    let mut solver = VelocityVerlet;
    // ---------------------

    let _ = event_loop.run(move |event, target| {
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
                            physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                            ..
                        },
                    ..
                } => target.exit(),
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Physics Step
                    // Run multiple substeps per frame for stability/speed
                    for _ in 0..10 {
                        solver.step(&mut state, &registry, 0.016 / 10.0);
                    }
                    
                    // Sync to Renderer
                    renderer.update_instances(&state.q, 3);

                    // Draw
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
