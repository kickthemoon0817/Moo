#![allow(deprecated)]
use crate::renderer::Renderer;
use moo::simulation::Simulation; 
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, window::Window};
// WindowBuilder removed in 0.30? replaced by Window::builder?
// No, Window::default_attributes().
// Let's check window creation code.

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

use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sim: Option<Simulation>,
    gui: Option<Gui>,
    
    // UI State for initialization
    init_width: f64,
    init_height: f64,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sim: None,
            gui: None,
            init_width: 800.0,
            init_height: 600.0,
        }
    }


}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("Khe (version 0.0.4)")
                .with_inner_size(winit::dpi::LogicalSize::new(self.init_width, self.init_height));
                
            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            self.window = Some(window.clone());
            
            // Blocking Async Init for Simplicity
            // In a real app, you might show a loading screen or spawn a thread.
            // wgpu creation is async.

            let window_clone = window.clone();
            
            // SAFETY: We are blocking the main thread, so `self` is pinned here.
            // We just need to mutate the Option fields.
            // pollster::block_on is a common way to run async code in sync contexts (winit 0.30)
            pollster::block_on(async move {
                 // But we can't move `self` into async block easily without Arc<Mutex>.
                 // Or we construct components and assign them back.
                 // Let's construct components and return them.
                 
                 let mut renderer = Renderer::new(window_clone.clone()).await;
                 renderer.update_camera_ortho(800.0, 600.0); // Hardcoded init
                 
                 let n_fluid = 4096;
                 let sim = Simulation::new(renderer.device(), n_fluid).await;
                 
                  let egui_ctx = egui::Context::default();
                  let egui_state = EguiState::new(
                    egui_ctx.clone(),
                    egui::ViewportId::ROOT,
                    &window_clone, 
                    Some(window_clone.scale_factor() as f32), 
                    None,
                    Some(2048),
                 );
                 
                  let egui_renderer = EguiRenderer::new(
                    renderer.device(),
                    wgpu::TextureFormat::Bgra8UnormSrgb, 
                    egui_wgpu::RendererOptions::default(), 
                 );
                 
                 let gui = Gui::new(egui_ctx, egui_state, egui_renderer);
                 
                 (renderer, sim, gui)
            });
            
            // Wait, pollster::block_on runs the future.
            let (renderer, sim, gui) = pollster::block_on(async {
                  let mut renderer = Renderer::new(window.clone()).await;
                  renderer.update_camera_ortho(800.0, 600.0);
                  let sim = Simulation::new(renderer.device(), 4096).await;
                  
                  let egui_ctx = egui::Context::default();
                  let egui_state = EguiState::new(
                    egui_ctx.clone(),
                    egui::ViewportId::ROOT,
                    &window, 
                    Some(window.scale_factor() as f32), 
                    None,
                    Some(2048),
                 );
                 let egui_renderer = EguiRenderer::new(
                        renderer.device(),
                        wgpu::TextureFormat::Bgra8UnormSrgb, 
                        egui_wgpu::RendererOptions::default(), 
                 );
                 let gui = Gui::new(egui_ctx, egui_state, egui_renderer);
                 
                 (renderer, sim, gui)
            });
            
            self.renderer = Some(renderer);
            self.sim = Some(sim);
            self.gui = Some(gui);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(w) => w,
            None => return,
        };
        
        let gui = match self.gui.as_mut() {
            Some(g) => g,
            None => return,
        };
        
        // Pass to egui
        let _ = gui.state.on_window_event(window, &event);
        
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                     renderer.resize(physical_size);
                     let aspect = physical_size.width as f32 / physical_size.height as f32;
                     let world_width = 800.0;
                     let world_height = world_width / aspect;
                     renderer.update_camera_ortho(world_width, world_height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                gui.cursor_pos = Some([position.x as f32, position.y as f32]);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    gui.mouse_pressed = state == ElementState::Pressed;
                }
            }
            WindowEvent::RedrawRequested => {
                let renderer = self.renderer.as_mut().unwrap();
                let sim = self.sim.as_mut().unwrap();
                
                 // Update params from UI
                let dt = 10.0f32.powf(gui.dt_log);
                
                // Unproject Mouse
                let mut world_mouse = [0.0, 0.0];
                let mut is_interacting = false;
                
                if let Some(pos) = gui.cursor_pos {
                    let width = renderer.size.width as f32;
                    let height = renderer.size.height as f32;
                    let aspect = width / height;
                    let view_width = 800.0;
                    let view_height = view_width / aspect;
                    
                    let ndc_x = (pos[0] / width) * 2.0 - 1.0;
                    let ndc_y = 1.0 - (pos[1] / height) * 2.0; 
                    
                    world_mouse[0] = ndc_x * (view_width / 2.0);
                    world_mouse[1] = ndc_y * (view_height / 2.0);
                    
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

                if !gui.paused {
                    for _ in 0..gui.steps_per_frame {
                        sim.step(renderer.device(), renderer.queue());
                    }
                }

                // Egui Frame
                let raw_input = gui.state.take_egui_input(window);
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
                            ui.label(format!("FPS: {:.1}", 60.0)); 
                        });
                });
                 
                gui.state.handle_platform_output(window, full_output.platform_output);
                let clipped_primitives = gui.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [renderer.size.width, renderer.size.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                if let Err(e) =
                    renderer.render_compute(
                        sim.compute.current_buffer(), 
                        sim.n_particles,
                        Some(&mut gui.renderer),
                        &clipped_primitives,
                        &screen_descriptor
                    )
                {
                    eprintln!("Render Error: {:?}", e);
                    event_loop.exit();
                }
                
                // Floor Lines Sync removed (legacy)
            }
             _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
