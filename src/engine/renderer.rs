#[derive(Debug, Default)]
pub struct Renderer {
    frame_count: u64,
}

impl Renderer {
    pub fn new() -> Self {
        Self { frame_count: 0 }
    }

    pub fn begin_frame(&mut self) {
        tracing::trace!(frame = self.frame_count, "begin frame");
    }

    pub fn end_frame(&mut self) {
        self.frame_count += 1;
        tracing::trace!(frame = self.frame_count, "end frame");
    }
}
