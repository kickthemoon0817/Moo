#[derive(Debug, Default)]
pub struct AudioEngine {
    muted: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self { muted: true }
    }

    pub fn silence(&mut self) {
        tracing::trace!("audio engine in silent mode");
    }
}
