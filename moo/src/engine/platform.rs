#[derive(Debug)]
pub struct PlatformLayer {
    boot_timestamp: std::time::Instant,
}

impl PlatformLayer {
    pub fn new() -> Self {
        Self {
            boot_timestamp: std::time::Instant::now(),
        }
    }

    pub fn pump_events(&mut self) {
        tracing::trace!("platform pump_events placeholder");
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.boot_timestamp.elapsed()
    }
}

impl Default for PlatformLayer {
    fn default() -> Self {
        Self::new()
    }
}
