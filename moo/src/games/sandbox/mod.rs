pub mod enemy;
pub mod player;
pub mod start_window;
pub mod states;

use crate::games::{Game, GameWindowDescriptor};
use crate::ui::UiElement;
use start_window::StartWindow;
use states::GameState;

#[derive(Debug)]
pub struct SandboxGame {
    pub state: GameState,
    start_window: StartWindow,
    start_logged: bool,
}

impl SandboxGame {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
            start_window: StartWindow::default(),
            start_logged: false,
        }
    }
}

impl Game for SandboxGame {
    fn name(&self) -> &str {
        "Sandbox"
    }

    fn update(&mut self) {
        if !self.start_logged {
            tracing::info!(
                target: "sandbox",
                title = self.start_window.title,
                subtitle = self.start_window.subtitle,
                width = self.start_window.width,
                height = self.start_window.height,
                "start window presented"
            );
            self.start_logged = true;
        }
        tracing::trace!(state = ?self.state, "sandbox game update placeholder");
    }

    fn window_descriptor(&self) -> GameWindowDescriptor {
        GameWindowDescriptor {
            title: Some(format!(
                "{} — {}",
                self.start_window.title, self.start_window.subtitle
            )),
            width: Some(self.start_window.width),
            height: Some(self.start_window.height),
            resizable: self.start_window.resizable,
        }
    }

    fn ui_elements(&self) -> Vec<UiElement> {
        self.start_window.button_elements()
    }
}
