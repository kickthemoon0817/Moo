use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone)]
pub enum SimCommand {
    Pause,
    Resume,
    Step(u32),
    SetDt(f32),
    SetGravity(f32, f32),
    Reset,
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
    pub fn send(&self, cmd: SimCommand) {
        let _ = self.sender.send(cmd);
    }
}
