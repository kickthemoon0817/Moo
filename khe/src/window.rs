use crate::renderer::{LineVertex, Renderer};
use moo::simulation::Simulation; 
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;

struct Gui {
    ctx: egui::Context,
    state: EguiState,
    renderer: EguiRenderer,
    
    // UI State
    paused: bool,
    steps_per_frame: usize,
    dt_log: f32, // Log scale for dt
    
    // Interaction
    cursor_pos: Option<[f32; 2]>,
    mouse_pressed: bool,
}

impl Gui {
    fn new(ctx: egui::Context, state: EguiState, renderer: EguiRenderer) -> Self {
        Self {
            ctx,
            state,
            renderer,
            paused: false,
            steps_per_frame: 10,
            dt_log: -2.3, // ~0.005
            cursor_pos: None,
            mouse_pressed: false,
        }
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Khe (version 0.0.1)")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let mut renderer = Renderer::new(window.clone()).await;
    renderer.update_camera_ortho(800.0, 600.0);

    // --- Simulation (Engine) ---
    // Decoupled from Window
    let n_fluid = 4096; // Increased from 100 for proper liquid demo!
    let mut sim = Simulation::new(renderer.device(), n_fluid).await;
    
    // --- GUI Setup ---
    let egui_ctx = egui::Context::default();
    let mut egui_state = EguiState::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window, 
        Some(window.scale_factor() as f32), 
        None
    );
    
    let egui_renderer = EguiRenderer::new(
        renderer.device(),
        wgpu::TextureFormat::Bgra8UnormSrgb, // Surface format
        None,
        1,
    );
    
    let mut gui = Gui::new(egui_ctx, egui_state, egui_renderer);


    // --- Constraints ---
    let floor_y = -200.0;

    event_loop
        .run(move |event, target| {
            // Pass event to egui
           if let Event::WindowEvent { event, .. } = &event {
               let _ = gui.state.on_window_event(&window, event);
           }
           
           match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                            let aspect = physical_size.width as f32 / physical_size.height as f32;
                            let world_width = 800.0;
                            let world_height = world_width / aspect;
                            renderer.update_camera_ortho(world_width, world_height);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            gui.cursor_pos = Some([position.x as f32, position.y as f32]);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if *button == MouseButton::Left {
                                gui.mouse_pressed = *state == ElementState::Pressed;
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            // Update params from UI
                            let dt = 10.0f32.powf(gui.dt_log);
                            
                            // Unproject Mouse
                            let mut world_mouse = [0.0, 0.0];
                            let mut is_interacting = false;
                            
                            if let Some(pos) = gui.cursor_pos {
                                // Basic Ortho Unproject (Hardcoded params specific to update_camera_ortho)
                                let width = renderer.size.width as f32;
                                let height = renderer.size.height as f32;
                                let aspect = width / height;
                                let view_width = 800.0;
                                let view_height = view_width / aspect;
                                
                                // Normalize -1..1 (Top-Left Origin for winit)
                                let ndc_x = (pos[0] / width) * 2.0 - 1.0;
                                let ndc_y = 1.0 - (pos[1] / height) * 2.0; // Y is up in World, down in Screen
                                
                                // Map to World
                                world_mouse[0] = ndc_x * (view_width / 2.0);
                                world_mouse[1] = ndc_y * (view_height / 2.0);
                                
                                // Only interact if not over UI (egui consumes pointer?)
                                // For now, assume if we are not hovering a window.
                                // Egui handles this via `ctx.wants_pointer_input()`.
                                if !gui.ctx.wants_pointer_input() {
                                    is_interacting = gui.mouse_pressed;
                                }
                            }

                            sim.compute.write_params(
                                renderer.queue(), 
                                dt, 25.0, 0.01, 2000.0, 200.0,
                                world_mouse,
                                is_interacting
                            );

                            // GPU Physics Steps
                            if !gui.paused {
                                for _ in 0..gui.steps_per_frame {
                                    sim.step(renderer.device(), renderer.queue());
                                }
                            }

                            // Prepare GUI
                            let raw_input = gui.state.take_egui_input(&window);
                            let full_output = gui.ctx.run(raw_input, |ctx| {
                                egui::Window::new("Moo Control Panel")
                                    .default_pos([10.0, 10.0])
                                    .show(ctx, |ui| {
                                        ui.heading("Simulation Control");
                                        if ui.button(if gui.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                                            gui.paused = !gui.paused;
                                        }
                                        if ui.button("⟲ Reset").clicked() {
                                            sim.reset(renderer.queue());
                                        }
                
                                        ui.add(egui::Slider::new(&mut gui.steps_per_frame, 1..=20).text("Steps/Frame"));
                                        ui.add(egui::Slider::new(&mut gui.dt_log, -4.0..=-1.0).text("Log(dt)"));
                                        
                                        ui.separator();
                                        ui.label(format!("Particles: {}", sim.n_particles));
                                        ui.label(format!("FPS: {:.1}", 60.0)); // Mock FPS
                                    });
                            });
                             
                             gui.state.handle_platform_output(&window, full_output.platform_output);
                             let clipped_primitives = gui.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                            
                             let screen_descriptor = egui_wgpu::ScreenDescriptor {
                                size_in_pixels: [renderer.size.width, renderer.size.height],
                                pixels_per_point: window.scale_factor() as f32,
                            };

                            // Render All (Simulation + GUI)
                             if let Err(e) =
                                renderer.render_compute(
                                    sim.compute.current_buffer(), 
                                    n_fluid as u32,
                                    Some(&mut gui.renderer),
                                    &clipped_primitives,
                                    &screen_descriptor
                                )
                            {
                                eprintln!("Render Error: {:?}", e);
                                target.exit();
                            }
                            
                            // Sync Lines (Floor)
                            // ... (standard floor logic)
                            let grid_size = 1000.0;
                            let y = floor_y as f32;
                            let color = [0.2, 0.2, 0.2];
                            let mut lines = Vec::new();
                            lines.push(LineVertex { position: [-grid_size, y, 0.0], color });
                            lines.push(LineVertex { position: [grid_size, y, 0.0], color });
                            renderer.update_lines(&lines);
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
