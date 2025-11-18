pub mod enemy;
pub mod player;
pub mod states;

use states::GameState;

#[derive(Debug)]
pub struct Game {
    pub state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
        }
    }

    pub fn update(&mut self) {
        tracing::trace!(state = ?self.state, "game update placeholder");
    }
}
