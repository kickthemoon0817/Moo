use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
};
use std::sync::Arc;
use crate::investigation::viz::renderer::{ScientificRenderer, LineVertex, UiVertex};
use crate::core::state::PhaseSpace;
use crate::core::solve::{Integrator, VelocityVerlet};
use crate::core::solve::constraints::{Constraint, FloorConstraint, SphereConstraint};
use crate::laws::registry::LawRegistry;
use crate::laws::classical::Gravity;
use crate::investigation::probe::{Probe, EnergyProbe};
use std::collections::VecDeque;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("PhysicLaw Scientific Visualization")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let mut renderer = ScientificRenderer::new(window.clone()).await;

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
        state.q[i*3] = (col as f64) * spacing - (cols as f64 * spacing / 2.0); // Center X
        state.q[i*3+1] = start_y + (row as f64) * spacing;
        state.q[i*3+2] = 0.0;
        
        state.mass[i*3] = 1.0;
        state.mass[i*3+1] = 1.0;
        state.mass[i*3+2] = 1.0;
        
        state.radius[i] = spacing / 2.0; // Visual radius
    }

    let mut registry = LawRegistry::new();
    registry.add(Gravity::new(500.0)); // Strong gravity to pull them down
    
    // SPH Law
    // h = smoothing length ~ 1.5 * spacing
    // rho0 = mass / volume ~ 1.0 / spacing^3? In 2D/3D it depends.
    // Let's tune rho0 for stability.
    // spacing 10 -> vol ~ 1000. m=1 -> rho ~ 0.001? 
    // Let's use h=25.0, rho0=0.002.
    use crate::laws::continuum::SPH;
    registry.add(SPH::new(25.0, 0.002, 10000.0));

    // --- Constraints ---
    let floor_y = -200.0;
    let mut constraints: Vec<Box<dyn Constraint>> = Vec::new();
    constraints.push(Box::new(FloorConstraint::new(floor_y, 0.5))); // Low restitution for fluid dampening
    constraints.push(Box::new(SphereConstraint::new(0.5))); // Particle collisions (backup for SPH)

    let mut solver = VelocityVerlet;
    
    // --- Probe / Graph Setup ---
    let probe = EnergyProbe;
    let mut energy_history: VecDeque<f64> = VecDeque::new();
    let history_len = 500;
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
                    for _ in 0..10 {
                        solver.step(&mut state, &registry, &constraints, 0.016 / 10.0);
                    }

                    // Probe Data
                    let energy = probe.measure(&state, &registry);
                    energy_history.push_back(energy);
                    if energy_history.len() > history_len {
                        energy_history.pop_front();
                    }
                    
                    // Sync to Renderer (Particles)
                    renderer.update_instances(&state.q, state.dof / 3);

                    // Sync Lines 
                    let mut lines = Vec::new();
                    
                    // 1. Draw Floor Grid
                    let grid_size = 1000.0;
                    let step = 100.0;
                    let y = floor_y as f32;
                    let color = [0.2, 0.2, 0.2];
                    
                    let mut x = -grid_size;
                    while x <= grid_size {
                        lines.push(LineVertex { position: [x, y, -grid_size], color });
                        lines.push(LineVertex { position: [x, y,  grid_size], color });
                        x += step;
                    }
                    let mut z = -grid_size;
                    while z <= grid_size {
                        lines.push(LineVertex { position: [-grid_size, y, z], color });
                        lines.push(LineVertex { position: [ grid_size, y, z], color });
                        z += step;
                    }

                    // 2. Draw Axes for Rigid Bodies
                    let axis_len = 30.0;
                    for i in 0..state.rot.len() {
                        let idx = i * 3;
                        let cx = state.q[idx];
                        let cy = state.q[idx+1];
                        let cz = state.q[idx+2];
                        let center = glam::Vec3::new(cx as f32, cy as f32, cz as f32);

                        let rot = state.rot[i]; 
                        let rot_f = glam::Quat::from_xyzw(
                            rot.x as f32, rot.y as f32, rot.z as f32, rot.w as f32
                        );

                        let x_axis = rot_f * glam::Vec3::X * axis_len;
                        let y_axis = rot_f * glam::Vec3::Y * axis_len;
                        let z_axis = rot_f * glam::Vec3::Z * axis_len;

                        lines.push(LineVertex { position: center.into(), color: [1.0, 0.0, 0.0] });
                        lines.push(LineVertex { position: (center + x_axis).into(), color: [1.0, 0.0, 0.0] });

                        lines.push(LineVertex { position: center.into(), color: [0.0, 1.0, 0.0] });
                        lines.push(LineVertex { position: (center + y_axis).into(), color: [0.0, 1.0, 0.0] });

                        lines.push(LineVertex { position: center.into(), color: [0.0, 0.0, 1.0] });
                        lines.push(LineVertex { position: (center + z_axis).into(), color: [0.0, 0.0, 1.0] });
                    }
                    
                    renderer.update_lines(&lines);

                    // 3. Draw UI Graph (Energy)
                    let mut ui_lines = Vec::new();
                    if !energy_history.is_empty() {
                         // Normalize
                         let min_e = energy_history.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                         let max_e = energy_history.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                         let mut range = max_e - min_e;
                         if range.abs() < 1e-6 { range = 1.0; } // Avoid div by zero
                         
                         // Screen Space: x [-0.9, 0.9], y [0.6, 0.9] (Top Area)
                         let start_x = -0.9;
                         let width = 1.8;
                         let start_y = 0.6;
                         let height = 0.3;
                         
                         for (i, &e) in energy_history.iter().enumerate() {
                             let t = i as f64 / (history_len - 1) as f64;
                             let x = start_x + t * width;
                             // Normalize e to [0, 1] relative to window
                             let norm_e = (e - min_e) / range;
                             let y = start_y + norm_e * height;
                             
                             ui_lines.push(UiVertex { 
                                 position: [x as f32, y as f32], 
                                 color: [1.0, 1.0, 0.0] // Yellow
                             });
                         }
                    }
                    renderer.update_ui_lines(&ui_lines);

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
