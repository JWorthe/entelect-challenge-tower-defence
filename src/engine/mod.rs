pub mod command;
pub mod geometry;
pub mod settings;

use self::command::{Command, BuildingType};
use self::geometry::Point;
use self::settings::{GameSettings, BuildingSettings};

use std::ops::FnMut;
use std::cmp;

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub status: GameStatus,
    pub player: Player,
    pub opponent: Player,
    pub player_unconstructed_buildings: Vec<UnconstructedBuilding>,
    pub player_buildings: Vec<Building>,
    pub unoccupied_player_cells: Vec<Point>,
    pub opponent_unconstructed_buildings: Vec<UnconstructedBuilding>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub energy: u16,
    pub health: u8,
    pub energy_generated: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnconstructedBuilding {
    pub pos: Point,
    pub health: u8,
    pub construction_time_left: u8,
    pub weapon_damage: u8,
    pub weapon_speed: u8,
    pub weapon_cooldown_period: u8,
    pub energy_generated_per_turn: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub pos: Point,
    pub health: u8,
    pub weapon_damage: u8,
    pub weapon_speed: u8,
    pub weapon_cooldown_time_left: u8,
    pub weapon_cooldown_period: u8,
    pub energy_generated_per_turn: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct Missile {
    pub pos: Point,
    pub damage: u8,
    pub speed: u8,
}

impl GameState {
    pub fn new(
        player: Player, opponent: Player,
        player_unconstructed_buildings: Vec<UnconstructedBuilding>, player_buildings: Vec<Building>,
        opponent_unconstructed_buildings: Vec<UnconstructedBuilding>, opponent_buildings: Vec<Building>,
        player_missiles: Vec<Missile>, opponent_missiles: Vec<Missile>,
        settings: &GameSettings) -> GameState {
        
        let unoccupied_player_cells = GameState::unoccupied_cells(
            &player_buildings, &player_unconstructed_buildings, Point::new(0, 0), Point::new(settings.size.x/2, settings.size.y)
        );
        let unoccupied_opponent_cells = GameState::unoccupied_cells(
            &opponent_buildings, &opponent_unconstructed_buildings, Point::new(settings.size.x/2, 0), Point::new(settings.size.x, settings.size.y)
        );
        GameState {
            status: GameStatus::Continue,
            player, opponent,
            player_unconstructed_buildings, player_buildings, unoccupied_player_cells,
            opponent_unconstructed_buildings, opponent_buildings, unoccupied_opponent_cells,
            player_missiles, opponent_missiles
        }
    }

    /**
     * Sorts the various arrays. Generally not necessary, but useful
     * for tests that check equality between states.
     */
    pub fn sort(&mut self) {
        self.player_unconstructed_buildings.sort_by_key(|b| b.pos);
        self.player_buildings.sort_by_key(|b| b.pos);
        self.unoccupied_player_cells.sort();
        self.opponent_unconstructed_buildings.sort_by_key(|b| b.pos);
        self.opponent_buildings.sort_by_key(|b| b.pos);
        self.unoccupied_opponent_cells.sort();
        self.player_missiles.sort_by_key(|b| b.pos);
        self.opponent_missiles.sort_by_key(|b| b.pos);
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

        GameState::update_construction(&mut self.player_unconstructed_buildings, &mut self.player_buildings, &mut self.player);
        GameState::update_construction(&mut self.opponent_unconstructed_buildings, &mut self.opponent_buildings, &mut self.opponent);

        GameState::add_missiles(&mut self.player_buildings, &mut self.player_missiles);
        GameState::add_missiles(&mut self.opponent_buildings, &mut self.opponent_missiles);

        GameState::move_missiles(&mut self.player_missiles, |p| p.wrapping_move_right(),
                                 &mut self.opponent_buildings, &mut self.opponent,
                                 &mut self.unoccupied_opponent_cells,
                                 &settings);
        GameState::move_missiles(&mut self.opponent_missiles, |p| p.wrapping_move_left(),
                                 &mut self.player_buildings, &mut self.player,
                                 &mut self.unoccupied_player_cells,
                                 &settings);

        GameState::add_energy(&mut self.player);
        GameState::add_energy(&mut self.opponent);

        GameState::perform_command(&mut self.player_unconstructed_buildings, &mut self.player_buildings,  &mut self.player, &mut self.unoccupied_player_cells, settings, player_command, &settings.size);
        GameState::perform_command(&mut self.opponent_unconstructed_buildings, &mut self.opponent_buildings, &mut self.opponent, &mut self.unoccupied_opponent_cells, settings, opponent_command, &settings.size);
        
        GameState::update_status(self);
    }

    fn perform_command(unconstructed_buildings: &mut Vec<UnconstructedBuilding>, buildings: &mut Vec<Building>, player: &mut Player, unoccupied_cells: &mut Vec<Point>, settings: &GameSettings, command: Command, size: &Point) {
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
                if blueprint.construction_time > 0 {
                    unconstructed_buildings.push(UnconstructedBuilding::new(p, blueprint));
                } else {
                    let building = Building::new(p, blueprint);
                    player.energy_generated += building.energy_generated_per_turn;
                    buildings.push(building);
                }

                let to_remove_index = unoccupied_cells.iter().position(|&pos| pos == p).unwrap();
                unoccupied_cells.swap_remove(to_remove_index);
            },
        }
    }

    fn update_construction(unconstructed_buildings: &mut Vec<UnconstructedBuilding>, buildings: &mut Vec<Building>, player: &mut Player) {
        for building in unconstructed_buildings.iter_mut() {
            building.construction_time_left -= 1;
            if building.is_constructed() {
                player.energy_generated += building.energy_generated_per_turn;
                buildings.push(building.to_building());
            }
        }
        unconstructed_buildings.retain(|b| !b.is_constructed());
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

    fn move_missiles<F>(missiles: &mut Vec<Missile>, mut wrapping_move_fn: F, opponent_buildings: &mut Vec<Building>, opponent: &mut Player, unoccupied_cells: &mut Vec<Point>, settings: &GameSettings)
    where F: FnMut(&mut Point) {
        for missile in missiles.iter_mut() {
            for _ in 0..missile.speed {
                wrapping_move_fn(&mut missile.pos);
                if missile.pos.x >= settings.size.x {
                    let damage = cmp::min(missile.damage, opponent.health);
                    opponent.health -= damage;
                    missile.speed = 0;
                }
                else {
                    for b in 0..opponent_buildings.len() {
                        // TODO latest game engine may be checking building health here
                        if opponent_buildings[b].pos == missile.pos {
                            let damage = cmp::min(missile.damage, opponent_buildings[b].health);
                            opponent_buildings[b].health -= damage;
                            missile.speed = 0;
                            if opponent_buildings[b].health == 0 {
                                unoccupied_cells.push(opponent_buildings[b].pos);
                                opponent.energy_generated -= opponent_buildings[b].energy_generated_per_turn;
                                opponent_buildings.swap_remove(b);
                                break;
                            }
                        }
                    }
                }
                
                if missile.speed == 0 {
                    break;
                }
            }
        }
        swap_retain(missiles, |m| m.speed > 0);
    }

    fn add_energy(player: &mut Player) {
        player.energy += player.energy_generated;
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

    fn unoccupied_cells(buildings: &[Building], unconstructed_buildings: &[UnconstructedBuilding], bl: Point, tr: Point) -> Vec<Point> {
        let mut result = Vec::with_capacity((tr.y-bl.y) as usize * (tr.x-bl.x) as usize);
        for y in bl.y..tr.y {
            for x in bl.x..tr.x {
                let pos = Point::new(x, y);
                if !buildings.iter().any(|b| b.pos == pos) && !unconstructed_buildings.iter().any(|b| b.pos == pos) {
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
        let mut result = Vec::with_capacity(3);
        for b in BuildingType::all().iter() {
            if settings.building_settings(*b).price <= energy {
                result.push(*b);
            }
        }
        result
    }
}

impl GameStatus {
    fn is_complete(&self) -> bool {
        *self != GameStatus::Continue
    }
}

impl Player {
    pub fn new(energy: u16, health: u8, settings: &GameSettings, buildings: &[Building]) -> Player {
        Player {
            energy,
            health,
            energy_generated: settings.energy_income + buildings.iter().map(|b| b.energy_generated_per_turn).sum::<u16>()
        }
    }
    
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

impl UnconstructedBuilding {
    pub fn new(pos: Point, blueprint: &BuildingSettings) -> UnconstructedBuilding {
        UnconstructedBuilding {
            pos,
            health: blueprint.health,
            construction_time_left: blueprint.construction_time,
            weapon_damage: blueprint.weapon_damage,
            weapon_speed: blueprint.weapon_speed,
            weapon_cooldown_period: blueprint.weapon_cooldown_period,
            energy_generated_per_turn: blueprint.energy_generated_per_turn
        }
    }
    
    fn is_constructed(&self) -> bool {
        self.construction_time_left == 0
    }

    fn to_building(&self) -> Building {
        Building {
            pos: self.pos,
            health: self.health,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_time_left: 0,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn
        }
    }
}

impl Building {
    pub fn new(pos: Point, blueprint: &BuildingSettings) -> Building {
        Building {
            pos,
            health: blueprint.health,
            weapon_damage: blueprint.weapon_damage,
            weapon_speed: blueprint.weapon_speed,
            weapon_cooldown_time_left: 0,
            weapon_cooldown_period: blueprint.weapon_cooldown_period,
            energy_generated_per_turn: blueprint.energy_generated_per_turn
        }
    }
    
    fn is_shooty(&self) -> bool {
        self.weapon_damage > 0
    }
}


fn swap_retain<T, F>(v: &mut Vec<T>, mut pred: F)
    where F: FnMut(&T) -> bool
{
    let mut new_len = v.len();
    for i in (0..v.len()).rev() {
        if !pred(&v[i]) {
            new_len -= 1;
            v.swap(i, new_len);
        }
    }
    v.truncate(new_len);
}
