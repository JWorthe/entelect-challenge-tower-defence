#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Continue,
    PlayerWon,
    OpponentWon,
    Draw
}
