pub mod command;
pub mod geometry;
pub mod settings;
pub mod expressive_engine;
pub mod bitwise_engine;

use self::command::{Command};
use self::geometry::Point;
use self::settings::{GameSettings};

pub trait GameState: Clone + Sync {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus;
    
    fn player(&self) -> &Player;
    fn opponent(&self) -> &Player;
    fn player_has_max_teslas(&self) -> bool;
    fn opponent_has_max_teslas(&self) -> bool;
    fn unoccupied_player_cells(&self) -> &[Point];
    fn unoccupied_opponent_cells(&self) -> &[Point];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Continue,
    PlayerWon,
    OpponentWon,
    Draw
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub energy: u16,
    pub health: u8,
    pub energy_generated: u16,
}
