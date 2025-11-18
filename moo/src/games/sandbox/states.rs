#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Playing,
    Paused,
    GameOver,
}
