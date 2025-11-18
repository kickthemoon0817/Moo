#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub app_name: String,
    pub window_width: u32,
    pub window_height: u32,
    pub target_fps: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            app_name: "Moo Engine".to_string(),
            window_width: 1280,
            window_height: 720,
            target_fps: 60,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FrameTiming {
    pub delta_seconds: f32,
    pub fps: f32,
}

pub struct FixedTimestep {
    frame_duration: f32,
    accumulator: f32,
}

impl FixedTimestep {
    pub fn from_fps(fps: u32) -> Self {
        let frame_duration = 1.0 / fps.max(1) as f32;
        Self {
            frame_duration,
            accumulator: 0.0,
        }
    }

    pub fn accumulate(&mut self, delta: f32) {
        self.accumulator += delta;
    }

    pub fn should_step(&mut self) -> bool {
        if self.accumulator >= self.frame_duration {
            self.accumulator -= self.frame_duration;
            true
        } else {
            false
        }
    }
}
