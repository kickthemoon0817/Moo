use std::sync::mpsc::{Receiver, SendError, Sender, channel};
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SimCommand {
    Pause,
    Resume,
    Step(u32),
    SetDt(f32),
    SetGravity(f32, f32),
    Reset,
}

/// Shared atomic counters readable by gRPC handlers and updated by the simulation loop.
pub struct SimMetrics {
    pub step_count: AtomicU64,
    pub particle_count: AtomicU32,
}

impl SimMetrics {
    pub fn new(particle_count: u32) -> Arc<Self> {
        Arc::new(Self {
            step_count: AtomicU64::new(0),
            particle_count: AtomicU32::new(particle_count),
        })
    }
}

pub struct CommandQueue {
    receiver: Receiver<SimCommand>,
}

pub struct CommandSender {
    sender: Sender<SimCommand>,
}

impl CommandQueue {
    pub fn new() -> (Self, CommandSender) {
        let (sender, receiver) = channel();
        (
            Self { receiver },
            CommandSender { sender },
        )
    }

    pub fn try_recv(&self) -> Option<SimCommand> {
        self.receiver.try_recv().ok()
    }
}

impl CommandSender {
    pub fn send(&self, cmd: SimCommand) -> Result<(), SendError<SimCommand>> {
        self.sender.send(cmd)
    }
}
