use crate::renderer::Renderer;
use moo::simulation::Simulation;
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

pub struct AsyncInitData {
    pub renderer: Renderer,
    pub sim: Simulation,
    pub gui: Gui,
}

use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;

pub struct Gui {
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
    
    // Viewport
    texture_id: Option<egui::TextureId>,
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
            texture_id: None,
        }
    }
}

// Gui struct unchanged

use winit::application::ApplicationHandler;
use winit::window::WindowId;

struct App {
    #[allow(unused)]
    proxy: EventLoopProxy<AsyncInitData>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sim: Option<Simulation>,
    gui: Option<Gui>,

    // UI State for initialization
    init_width: f64,
    init_height: f64,
}

impl App {
    fn new(proxy: EventLoopProxy<AsyncInitData>) -> Self {
        Self {
            proxy,
            window: None,
            renderer: None,
            sim: None,
            gui: None,
            init_width: 800.0,
            init_height: 600.0,
        }
    }
}

impl ApplicationHandler<AsyncInitData> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("Khe (version 0.0.4)")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.init_width,
                    self.init_height,
                ));

            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            self.window = Some(window.clone());

            // WASM: Append Canvas
            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowExtWebSys;
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| {
                        let dst = doc.get_element_by_id("khe-canvas")?;
                        let canvas = window.canvas()?;
                        dst.append_child(&canvas).ok()?;
                        Some(())
                    })
                    .expect("Failed to append canvas to document");
            }

            let window_clone = window.clone();
            #[cfg(target_arch = "wasm32")]
            let proxy = self.proxy.clone();

            // Native: Sync Init using pollster
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (renderer, sim, gui) = pollster::block_on(async move {
                    let mut renderer = Renderer::new(window_clone.clone()).await;
                    renderer.update_camera_ortho(800.0, 600.0);
                    let sim = Simulation::new(renderer.device(), 4096).await;

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
                    let mut gui = Gui::new(egui_ctx, egui_state, egui_renderer);
                    
                    // Register Offscreen Texture
                    gui.texture_id = Some(renderer.register_texture(&mut gui.renderer));

                    (renderer, sim, gui)
                });
                self.renderer = Some(renderer);
                self.sim = Some(sim);
                self.gui = Some(gui);
            }

            // WASM: Async Init using spawn_local
            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut renderer = Renderer::new(window_clone.clone()).await;
                    renderer.update_camera_ortho(800.0, 600.0);
                    let sim = Simulation::new(renderer.device(), 4096).await;

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
                    let mut gui = Gui::new(egui_ctx, egui_state, egui_renderer);
                    
                    // Register Offscreen Texture
                    gui.texture_id = Some(renderer.register_texture(&mut gui.renderer));

                    proxy
                        .send_event(AsyncInitData { renderer, sim, gui })
                        .expect("Failed to send init event");
                });
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AsyncInitData) {
        self.renderer = Some(event.renderer);
        self.sim = Some(event.sim);
        self.gui = Some(event.gui);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                // Safely get components
                if self.renderer.is_none() || self.sim.is_none() {
                    return;
                }
                
                let renderer = self.renderer.as_mut().unwrap();
                let sim = self.sim.as_mut().unwrap();
                
                // Update params from UI
                let dt = 10.0f32.powf(gui.dt_log);

                let mut world_mouse = [0.0, 0.0];
                let mut is_interacting = false;

                // Egui Frame
                let raw_input = gui.state.take_egui_input(window);
                let full_output = gui.ctx.run(raw_input, |ctx| {
                    // 1. Side Panel (Settings)
                    egui::SidePanel::left("settings_panel")
                        .resizable(true)
                        .default_width(200.0)
                        .show(ctx, |ui| {
                            ui.heading("Configuration");
                            ui.separator();

                            if ui
                                .button(if gui.paused {
                                    "▶ Resume"
                                } else {
                                    "⏸ Pause"
                                })
                                .clicked()
                            {
                                gui.paused = !gui.paused;
                            }
                            if ui.button("⟲ Reset").clicked() {
                                sim.reset(renderer.queue());
                            }

                            ui.separator();
                            ui.label("Simulation Parameters");
                            ui.add(
                                egui::Slider::new(&mut gui.steps_per_frame, 1..=20)
                                    .text("Steps/Frame"),
                            );
                            ui.add(egui::Slider::new(&mut gui.dt_log, -4.0..=-1.0).text("Log(dt)"));

                            ui.separator();
                            ui.label("Stats");
                            ui.label(format!("Particles: {}", sim.n_particles));
                            ui.label(format!("FPS: {:.1}", 60.0)); // TODO: Real FPS
                        });

                    // 2. Central Panel (Viewport)
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // Draw Offscreen Texture
                        if let Some(texture_id) = gui.texture_id {
                            let available_size = ui.available_size();
                            let img = egui::Image::new(egui::load::SizedTexture::new(
                                texture_id,
                                available_size,
                            ));
                            let response = ui.add(img);

                            // Input Translation: Viewport(UI) -> World
                            let rect = response.rect;
                            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                if rect.contains(mouse_pos) {
                                    // Local coordinates in the Image widget
                                    let local_x = mouse_pos.x - rect.min.x;
                                    let local_y = mouse_pos.y - rect.min.y;

                                    // Normalized Device Coordinates (-1 to 1)
                                    // Note: Inverted Y (Screen y goes down, World y goes up) -- Wait, Projection is orthographic.
                                    // Renderer Ortho: -half_w to half_w. 0,0 is center.
                                    // Texture: 0,0 is Top-Left (Vulkan/WGPU y down? No, WGPU NDC y is up, but standard texture sampling is y down 0..1)
                                    
                                    // Let's assume standard UV Mapping where (0,0) is Top-Left of the image.
                                    let uv_x = local_x / rect.width();
                                    let uv_y = local_y / rect.height();

                                    // Map UV to World (using current camera settings)
                                    let aspect = rect.width() / rect.height();
                                    
                                    // NOTE: We update camera to match the window aspect, but here the aspect might differ due to SidePanel.
                                    // Ideally we should update camera based on THIS rect size.
                                    // For now, let's assume loose coupling.
                                    
                                    let view_width = 800.0; // Fixed World Width for zoom
                                    let view_height = view_width / aspect;

                                    let world_x = (uv_x * 2.0 - 1.0) * (view_width / 2.0);
                                    let world_y = (1.0 - uv_y * 2.0) * (view_height / 2.0); // Flip Y

                                    world_mouse = [world_x, world_y];
                                    
                                    // Only interact if hovering viewport
                                    if ui.input(|i| i.pointer.primary_down()) {
                                        is_interacting = true;
                                    }
                                }
                            }
                        }
                    });
                });

                // Update params (using newly calculated world_mouse)
                sim.compute.write_params(
                    renderer.queue(),
                    moo::platform::compute::SimConfig {
                        dt,
                        h: 25.0,
                        rho0: 0.01,
                        stiffness: 2000.0,
                        viscosity: 200.0,
                        mouse_pos: world_mouse, // Updated from Viewport logic
                        mouse_pressed: is_interacting,
                    },
                );

                if !gui.paused {
                    for _ in 0..gui.steps_per_frame {
                        sim.step(renderer.device(), renderer.queue());
                    }
                }

                gui.state
                    .handle_platform_output(window, full_output.platform_output);
                let clipped_primitives = gui
                    .ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [renderer.size.width, renderer.size.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                // Handle Texture Deltas (Fonts, User Images)
                for (id, image_delta) in &full_output.textures_delta.set {
                    gui.renderer.update_texture(renderer.device(), renderer.queue(), *id, image_delta);
                }
                for id in &full_output.textures_delta.free {
                    gui.renderer.free_texture(id);
                }

                if let Err(e) = renderer.render_compute(
                    sim.compute.current_buffer(),
                    sim.n_particles,
                    Some(&mut gui.renderer),
                    &clipped_primitives,
                    &screen_descriptor,
                ) {
                    eprintln!("Render Error: {:?}", e);
                    event_loop.exit();
                }
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
    let event_loop = EventLoop::<AsyncInitData>::with_user_event()
        .build()
        .unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy);
    let _ = event_loop.run_app(&mut app);
}
