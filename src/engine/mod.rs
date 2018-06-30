pub mod command;
pub mod geometry;
pub mod settings;
pub mod expressive_engine;


use self::command::{Command};
use self::geometry::Point;
use self::settings::{GameSettings};

pub trait GameState: Clone + Sync {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus;
    
    fn player(&self) -> &Player;
    fn opponent(&self) -> &Player;
    fn player_has_max_teslas(&self) -> bool;
    fn opponent_has_max_teslas(&self) -> bool;
    fn unoccupied_player_cells(&self) -> &Vec<Point>;
    fn unoccupied_opponent_cells(&self) -> &Vec<Point>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Continue,
    PlayerWon,
    OpponentWon,
    Draw
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub energy: u16,
    pub health: u8,
    pub energy_generated: u16,
}

impl Player {
    pub fn new(energy: u16, health: u8, settings: &GameSettings, buildings: &[expressive_engine::Building]) -> Player {
        Player {
            energy,
            health,
            energy_generated: settings.energy_income + buildings.iter().map(|b| b.energy_generated_per_turn).sum::<u16>()
        }
    }
}
