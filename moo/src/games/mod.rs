pub mod sandbox;

use crate::ui::UiElement;

#[derive(Debug, Clone)]
pub struct GameWindowDescriptor {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: bool,
}

impl Default for GameWindowDescriptor {
    fn default() -> Self {
        Self {
            title: None,
            width: None,
            height: None,
            resizable: true,
        }
    }
}

pub trait Game: Send {
    fn name(&self) -> &str;
    fn update(&mut self);

    fn window_descriptor(&self) -> GameWindowDescriptor {
        GameWindowDescriptor::default()
    }

    fn ui_elements(&self) -> Vec<UiElement> {
        Vec::new()
    }
}

pub use sandbox::SandboxGame;
