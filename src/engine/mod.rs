pub mod audio;
pub mod core;
pub mod platform;
pub mod renderer;
pub mod resources;
pub mod scene;

use anyhow::Result;
use audio::AudioEngine;
use core::EngineConfig;
use platform::PlatformLayer;
use renderer::Renderer;
use resources::ResourceManager;
use scene::SceneGraph;

pub struct EngineApp {
    config: EngineConfig,
    platform: PlatformLayer,
    renderer: Renderer,
    resources: ResourceManager,
    scene: SceneGraph,
    audio: AudioEngine,
}

impl EngineApp {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            platform: PlatformLayer::new(),
            renderer: Renderer::new(),
            resources: ResourceManager::default(),
            scene: SceneGraph::default(),
            audio: AudioEngine::new(),
            config,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        tracing::info!(target: "engine", app = %self.config.app_name, "Engine starting");
        self.tick_placeholder();
        tracing::info!(target: "engine", "Engine shutdown complete");
        Ok(())
    }

    fn tick_placeholder(&mut self) {
        tracing::debug!("tick placeholder executed");
        self.platform.pump_events();
        self.audio.silence();
    }
}
