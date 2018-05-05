pub mod command;
pub mod geometry;
pub mod settings;

use self::command::{BuildingType, Command};
use self::geometry::Point;
use self::settings::GameSettings;

use std::ops::Fn;
use std::cmp;

#[derive(Debug, Clone)]
struct GameState {
    status: GameStatus,
    player: Player,
    opponent: Player,
    player_buildings: Vec<Building>,
    opponent_buildings: Vec<Building>,
    player_missiles: Vec<Missile>,
    opponent_missiles: Vec<Missile>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Continue,
    PlayerWon,
    OpponentWon,
    Draw,
    InvalidMove
}

impl GameStatus {
    fn is_complete(&self) -> bool {
        *self != GameStatus::Continue
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    energy: u16,
    health: u16
}

#[derive(Debug, Clone)]
struct Building {
    pos: Point,
    health: u16,
    construction_time_left: u8,
    weapon_damage: u16,
    weapon_speed: u8,
    weapon_cooldown_time_left: u8,
    weapon_cooldown_period: u8,
    energy_generated_per_turn: u16
}

impl Building {
    fn new(pos: Point, building: BuildingType) -> Building {
        match building {
            BuildingType::Defense => Building {
                pos: pos,
                health: 20,
                construction_time_left: 3,
                weapon_damage: 0,
                weapon_speed: 0,
                weapon_cooldown_time_left: 0,
                weapon_cooldown_period: 0,
                energy_generated_per_turn: 0
            },
            BuildingType::Attack => Building {
                pos: pos,
                health: 5,
                construction_time_left: 1,
                weapon_damage: 5,
                weapon_speed: 1,
                weapon_cooldown_time_left: 0,
                weapon_cooldown_period: 3,
                energy_generated_per_turn: 0
            },
            BuildingType::Energy => Building {
                pos: pos,
                health: 5,
                construction_time_left: 1,
                weapon_damage: 0,
                weapon_speed: 0,
                weapon_cooldown_time_left: 0,
                weapon_cooldown_period: 0,
                energy_generated_per_turn: 3
            }
        }
        
    }

    fn is_constructed(&self) -> bool {
        self.construction_time_left == 0
    }

    fn is_shooty(&self) -> bool {
        self.is_constructed() && self.weapon_damage >= 0
    }
}

#[derive(Debug, Clone)]
struct Missile {
    pos: Point,
    damage: u16,
    speed: u8,
}

impl Missile {
    fn is_stopped(&self) -> bool {
        self.speed == 0
    }
}

impl GameState {
    pub fn simulate(&self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameState {
        if self.status.is_complete() {
            return self.clone();
        }
        
        let mut state = self.clone();
        GameState::perform_command(&mut state.player_buildings, player_command, &settings.size);
        GameState::perform_command(&mut state.opponent_buildings, opponent_command, &settings.size);

        GameState::update_construction(&mut state.player_buildings);
        GameState::update_construction(&mut state.opponent_buildings);

        GameState::add_missiles(&mut state.player_buildings, &mut state.player_missiles);
        GameState::add_missiles(&mut state.opponent_buildings, &mut state.opponent_missiles);

        GameState::move_missiles(&mut state.player_missiles, |p| p.move_right(&settings.size),
                                 &mut state.opponent_buildings, &mut state.opponent);
        GameState::move_missiles(&mut state.opponent_missiles, |p| p.move_left(),
                                 &mut state.player_buildings, &mut state.player);

        GameState::add_energy(&mut state.player, settings, &state.player_buildings);
        GameState::add_energy(&mut state.opponent, settings, &state.opponent_buildings);

        GameState::update_status(&mut state);
        state
    }

    fn perform_command(buildings: &mut Vec<Building>, command: Command, size: &Point) -> bool {
        match command {
            Command::Nothing => { true },
            Command::Build(p, b) => {
                let occupied = buildings.iter().any(|b| b.pos == p);
                let in_range = p.x < size.x && p.y < size.y;
                buildings.push(Building::new(p, b));
                !occupied && in_range
            },
        }
    }

    fn update_construction(buildings: &mut Vec<Building>) {
        for building in buildings.iter_mut().filter(|b| !b.is_constructed()) {
            building.construction_time_left -= 1;
        }
    }

    fn add_missiles(buildings: &mut Vec<Building>, missiles: &mut Vec<Missile>) {
        for building in buildings.iter_mut().filter(|b| b.is_shooty()) {
            if building.weapon_cooldown_time_left > 0 {
                building.weapon_cooldown_time_left -= 1;
            } else {
                missiles.push(Missile {
                    pos: building.pos,
                    speed: building.weapon_speed,
                    damage: building.weapon_damage,
                });
                building.weapon_cooldown_time_left = building.weapon_cooldown_period;
            }
        }
    }

    fn move_missiles<F>(missiles: &mut Vec<Missile>, move_fn: F, opponent_buildings: &mut Vec<Building>, opponent: &mut Player)
    where F: Fn(Point) -> Option<Point> {
        for missile in missiles.iter_mut() {
            for _ in 0..missile.speed {
                match move_fn(missile.pos) {
                    None => {
                        let damage = cmp::min(missile.damage, opponent.health);
                        opponent.health -= damage;
                        missile.speed = 0;
                    },
                    Some(point) => {
                        missile.pos = point;
                        for hit in opponent_buildings.iter_mut().filter(|b| b.is_constructed() && b.pos == point && b.health > 0) {
                            let damage = cmp::min(missile.damage, hit.health);
                            hit.health -= damage;
                            missile.speed = 0;                    
                        }
                    }
                }
                
                if missile.speed == 0 {
                    break;
                }
            }
        }
        missiles.retain(|m| m.speed > 0);
        opponent_buildings.retain(|b| b.health > 0);
    }

    fn add_energy(player: &mut Player, settings: &GameSettings, buildings: &Vec<Building>) {
        player.energy += settings.energy_income;
        player.energy += buildings.iter().map(|b| b.energy_generated_per_turn).sum::<u16>();
    }

    fn update_status(state: &mut GameState) {
        let player_dead = state.player.health == 0;
        let opponent_dead = state.player.health == 0;
        state.status = match (player_dead, opponent_dead) {
            (true, true) => GameStatus::Draw,
            (true, false) => GameStatus::PlayerWon,
            (false, true) => GameStatus::OpponentWon,
            (false, false) => GameStatus::Continue,
        };
    }
}
