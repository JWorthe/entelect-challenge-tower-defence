pub mod command;
pub mod geometry;
pub mod settings;

use self::command::{Command, BuildingType};
use self::geometry::Point;
use self::settings::{GameSettings, BuildingSettings};

use std::ops::Fn;
use std::cmp;

#[derive(Debug, Clone)]
pub struct GameState {
    pub status: GameStatus,
    pub player: Player,
    pub opponent: Player,
    pub player_buildings: Vec<Building>,
    pub unoccupied_player_cells: Vec<Point>,
    pub opponent_buildings: Vec<Building>,
    pub unoccupied_opponent_cells: Vec<Point>,
    pub player_missiles: Vec<Missile>,
    pub opponent_missiles: Vec<Missile>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Continue,
    PlayerWon,
    OpponentWon,
    Draw
}

#[derive(Debug, Clone)]
pub struct Player {
    pub energy: u16,
    pub health: u16
}

#[derive(Debug, Clone)]
pub struct Building {
    pub pos: Point,
    pub health: u16,
    pub construction_time_left: u8,
    pub weapon_damage: u16,
    pub weapon_speed: u8,
    pub weapon_cooldown_time_left: u8,
    pub weapon_cooldown_period: u8,
    pub energy_generated_per_turn: u16
}

#[derive(Debug, Clone)]
pub struct Missile {
    pub pos: Point,
    pub damage: u16,
    pub speed: u8,
}

impl GameState {
    pub fn new(player: Player, opponent: Player, player_buildings: Vec<Building>, opponent_buildings: Vec<Building>, player_missiles: Vec<Missile>, opponent_missiles: Vec<Missile>, settings: &GameSettings) -> GameState {
        let unoccupied_player_cells = GameState::unoccupied_cells(
            &player_buildings, Point::new(0, 0), Point::new(settings.size.x/2, settings.size.y)
        );
        let unoccupied_opponent_cells = GameState::unoccupied_cells(
            &opponent_buildings, Point::new(settings.size.x/2, 0), Point::new(settings.size.x, settings.size.y)
        );
        GameState {
            status: GameStatus::Continue,
            player: player,
            opponent: opponent,
            player_buildings: player_buildings,
            unoccupied_player_cells: unoccupied_player_cells,
            opponent_buildings: opponent_buildings,
            unoccupied_opponent_cells: unoccupied_opponent_cells,
            player_missiles: player_missiles,
            opponent_missiles: opponent_missiles
        }
    }

    pub fn simulate(&self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameState {
        let mut state = self.clone();
        state.simulate_mut(settings, player_command, opponent_command);
        state
    }

    pub fn simulate_mut(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) {
        if self.status.is_complete() {
            return;
        }

        GameState::perform_command(&mut self.player_buildings, &mut self.player, &mut self.unoccupied_player_cells, settings, player_command, &settings.size);
        GameState::perform_command(&mut self.opponent_buildings, &mut self.opponent, &mut self.unoccupied_opponent_cells, settings, opponent_command, &settings.size);

        GameState::update_construction(&mut self.player_buildings);
        GameState::update_construction(&mut self.opponent_buildings);

        GameState::add_missiles(&mut self.player_buildings, &mut self.player_missiles);
        GameState::add_missiles(&mut self.opponent_buildings, &mut self.opponent_missiles);

        GameState::move_missiles(&mut self.player_missiles, |p| p.move_right(&settings.size),
                                 &mut self.opponent_buildings, &mut self.opponent,
                                 &mut self.unoccupied_opponent_cells);
        GameState::move_missiles(&mut self.opponent_missiles, |p| p.move_left(),
                                 &mut self.player_buildings, &mut self.player,
                                 &mut self.unoccupied_player_cells);

        GameState::add_energy(&mut self.player, settings, &self.player_buildings);
        GameState::add_energy(&mut self.opponent, settings, &self.opponent_buildings);

        GameState::update_status(self);
    }

    fn perform_command(buildings: &mut Vec<Building>, player: &mut Player, unoccupied_cells: &mut Vec<Point>, settings: &GameSettings, command: Command, size: &Point) {
        match command {
            Command::Nothing => { },
            Command::Build(p, b) => {
                let blueprint = settings.building_settings(b);

                // This is used internally. I should not be making
                // invalid moves!
                debug_assert!(!buildings.iter().any(|b| b.pos == p));
                debug_assert!(p.x < size.x && p.y < size.y);
                debug_assert!(player.energy >= blueprint.price);

                player.energy -= blueprint.price;
                buildings.push(Building::new(p, blueprint));
                unoccupied_cells.retain(|&pos| pos != p);
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

    fn move_missiles<F>(missiles: &mut Vec<Missile>, move_fn: F, opponent_buildings: &mut Vec<Building>, opponent: &mut Player, unoccupied_cells: &mut Vec<Point>,)
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
                        for hit in opponent_buildings.iter_mut().filter(|b| b.is_constructed() && b.pos == point/* && b.health > 0*/) { //TODO surely this health>0 belongs? Not what the real game engine is doing unfortunately
                            let damage = cmp::min(missile.damage, hit.health);
                            hit.health -= damage;
                            missile.speed = 0;
                        }
                    }
                }

                /*
                check is necessary if speed could be > 1, which isn't the case yet
                if missile.speed == 0 {
                    break;
                }
                 */
            }
        }
        missiles.retain(|m| m.speed > 0);

        for b in opponent_buildings.iter().filter(|b| b.health == 0) {
            unoccupied_cells.push(b.pos);
        }
        opponent_buildings.retain(|b| b.health > 0);
    }

    fn add_energy(player: &mut Player, settings: &GameSettings, buildings: &Vec<Building>) {
        player.energy += settings.energy_income;
        player.energy += buildings.iter().map(|b| b.energy_generated_per_turn).sum::<u16>();
    }

    fn update_status(state: &mut GameState) {
        let player_dead = state.player.health == 0;
        let opponent_dead = state.opponent.health == 0;
        state.status = match (player_dead, opponent_dead) {
            (true, true) => GameStatus::Draw,
            (false, true) => GameStatus::PlayerWon,
            (true, false) => GameStatus::OpponentWon,
            (false, false) => GameStatus::Continue,
        };
    }

    pub fn unoccupied_player_cells_in_row(&self, y: u8) -> Vec<Point> {
        self.unoccupied_player_cells.iter().filter(|p| p.y == y).cloned().collect()
    }

    fn unoccupied_cells(buildings: &[Building], bl: Point, tr: Point) -> Vec<Point> {
        let mut result = Vec::with_capacity((tr.y-bl.y) as usize * (tr.x-bl.x) as usize);
        for y in bl.y..tr.y {
            for x in bl.x..tr.x {
                let pos = Point::new(x, y);
                if !buildings.iter().any(|b| b.pos == pos) {
                    result.push(pos);
                }
            }
        }
        result
    }

    pub fn player_affordable_buildings(&self, settings: &GameSettings) -> Vec<BuildingType> {
        GameState::affordable_buildings(self.player.energy, settings)
    }

    pub fn opponent_affordable_buildings(&self, settings: &GameSettings) -> Vec<BuildingType> {
        GameState::affordable_buildings(self.opponent.energy, settings)
    }

    fn affordable_buildings(energy: u16, settings: &GameSettings) -> Vec<BuildingType> {
        BuildingType::all().iter()
            .filter(|&b| settings.building_settings(*b).price <= energy)
            .cloned()
            .collect()
    }
}

impl GameStatus {
    fn is_complete(&self) -> bool {
        *self != GameStatus::Continue
    }
}

impl Player {
    pub fn can_afford_all_buildings(&self, settings: &GameSettings) -> bool {
        self.can_afford_attack_buildings(settings) &&
            self.can_afford_defence_buildings(settings) &&
            self.can_afford_energy_buildings(settings)
    }

    pub fn can_afford_attack_buildings(&self, settings: &GameSettings) -> bool {
        self.energy >= settings.attack.price
    }
    pub fn can_afford_defence_buildings(&self, settings: &GameSettings) -> bool {
        self.energy >= settings.defence.price
    }
    pub fn can_afford_energy_buildings(&self, settings: &GameSettings) -> bool {
        self.energy >= settings.energy.price
    }

}

impl Building {
    pub fn new(pos: Point, blueprint: &BuildingSettings) -> Building {
        Building {
            pos: pos,
            health: blueprint.health,
            construction_time_left: blueprint.construction_time,
            weapon_damage: blueprint.weapon_damage,
            weapon_speed: blueprint.weapon_speed,
            weapon_cooldown_time_left: 0,
            weapon_cooldown_period: blueprint.weapon_cooldown_period,
            energy_generated_per_turn: blueprint.energy_generated_per_turn
        }
    }

    fn is_constructed(&self) -> bool {
        self.construction_time_left == 0
    }

    fn is_shooty(&self) -> bool {
        self.is_constructed() && self.weapon_damage > 0
    }
}


