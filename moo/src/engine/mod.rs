pub mod audio;
pub mod core;
pub mod platform;
pub mod renderer;
pub mod resources;
pub mod scene;

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use audio::AudioEngine;
use core::EngineConfig;
use platform::PlatformLayer;
use renderer::Renderer;
use resources::ResourceManager;
use scene::SceneGraph;
use wgpu::SurfaceError;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::games::Game;
use crate::ui::UiElement;

pub struct EngineApp {
    config: EngineConfig,
    platform: PlatformLayer,
    resources: ResourceManager,
    scene: SceneGraph,
    audio: AudioEngine,
    game: Box<dyn Game>,
    ui_elements: Vec<UiElement>,
}

impl EngineApp {
    pub fn new(config: EngineConfig, game: impl Game + 'static) -> Self {
        Self {
            platform: PlatformLayer::new(),
            resources: ResourceManager::default(),
            scene: SceneGraph::default(),
            audio: AudioEngine::new(),
            game: Box::new(game),
            config,
            ui_elements: Vec::new(),
        }
    }

    pub fn run(self) -> Result<()> {
        tracing::info!(
            target: "engine",
            app = %self.config.app_name,
            game = %self.game.name(),
            "Engine starting"
        );

        let event_loop = EventLoop::new().context("failed to create event loop")?;
        let mut engine = self;
        let window_prefs = engine.game.window_descriptor();
        let window_width = window_prefs.width.unwrap_or(engine.config.window_width) as f64;
        let window_height = window_prefs.height.unwrap_or(engine.config.window_height) as f64;
        let window_title = window_prefs
            .title
            .clone()
            .unwrap_or_else(|| engine.window_title());

        let window = Arc::new(
            WindowBuilder::new()
                .with_title(window_title)
                .with_resizable(window_prefs.resizable)
                .with_inner_size(LogicalSize::new(window_width, window_height))
                .build(&event_loop)
                .context("failed to create window")?,
        );
        let mut renderer: Option<Renderer> = None;
        let mut last_frame = Instant::now();

        event_loop
            .run(move |event, target| match event {
                Event::Resumed => {
                    if renderer.is_none() {
                        match pollster::block_on(Renderer::new(window.clone())) {
                            Ok(new_renderer) => {
                                tracing::info!("renderer initialized");
                                renderer = Some(new_renderer);
                            }
                            Err(err) => {
                                tracing::error!(%err, "failed to initialize renderer");
                                target.exit();
                            }
                        }
                    }
                }
                Event::AboutToWait => {
                    if renderer.is_some() {
                        window.request_redraw();
                    }
                }
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            tracing::info!("window close requested");
                            target.exit();
                        }
                        WindowEvent::Resized(size) => {
                            if let Some(renderer) = renderer.as_mut() {
                                renderer.resize(size);
                            }
                        }
                        WindowEvent::ScaleFactorChanged {
                            mut inner_size_writer,
                            ..
                        } => {
                            let new_size = window.inner_size();
                            let _ = inner_size_writer.request_inner_size(new_size);
                            if let Some(renderer) = renderer.as_mut() {
                                renderer.resize(new_size);
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            if let Some(renderer) = renderer.as_mut() {
                                let now = Instant::now();
                                let delta = now.duration_since(last_frame);
                                last_frame = now;

                                engine.tick(delta);
                                match renderer.render(&engine.ui_elements) {
                                    Ok(()) => {}
                                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                                        renderer.resize(window.inner_size());
                                    }
                                    Err(SurfaceError::OutOfMemory) => {
                                        tracing::error!("GPU out of memory, shutting down engine");
                                        target.exit();
                                    }
                                    Err(SurfaceError::Timeout) => {
                                        tracing::warn!("surface timeout, retrying next frame");
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            })
            .map_err(|err| anyhow!(err))?;

        tracing::info!(target: "engine", "Engine shutdown complete");
        Ok(())
    }

    fn window_title(&self) -> String {
        format!("{} — {}", self.config.app_name, self.game.name())
    }

    fn tick(&mut self, delta: std::time::Duration) {
        let scene_nodes = self.scene.len();
        let texture_count = self.resources.texture_count();
        tracing::debug!(
            target: "engine",
            frame_delta_ms = %delta.as_millis(),
            scene_nodes,
            texture_count,
            "frame tick"
        );
        self.platform.pump_events();
        self.game.update();
        self.ui_elements = self.game.ui_elements();
        self.audio.silence();
    }
}
