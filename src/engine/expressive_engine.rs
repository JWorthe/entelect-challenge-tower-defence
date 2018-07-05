use std::ops::FnMut;
use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings, BuildingSettings};
use engine::{GameStatus, Player, GameState};

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressiveGameState {
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

impl GameState for ExpressiveGameState {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus {
        if self.status.is_complete() {
            return self.status;
        }

        ExpressiveGameState::perform_construct_command(&mut self.player_unconstructed_buildings, &mut self.player_buildings,  &mut self.player, &mut self.unoccupied_player_cells, settings, player_command, &settings.size);
        ExpressiveGameState::perform_construct_command(&mut self.opponent_unconstructed_buildings, &mut self.opponent_buildings, &mut self.opponent, &mut self.unoccupied_opponent_cells, settings, opponent_command, &settings.size);
        ExpressiveGameState::perform_deconstruct_command(&mut self.player_unconstructed_buildings, &mut self.player_buildings,  &mut self.player, &mut self.unoccupied_player_cells, player_command);
        ExpressiveGameState::perform_deconstruct_command(&mut self.opponent_unconstructed_buildings, &mut self.opponent_buildings, &mut self.opponent, &mut self.unoccupied_opponent_cells, opponent_command);
        
        ExpressiveGameState::update_construction(&mut self.player_unconstructed_buildings, &mut self.player_buildings, &mut self.player);
        ExpressiveGameState::update_construction(&mut self.opponent_unconstructed_buildings, &mut self.opponent_buildings, &mut self.opponent);

        ExpressiveGameState::fire_teslas(&mut self.player, &mut self.player_buildings, &mut self.unoccupied_player_cells, &mut self.opponent, &mut self.opponent_buildings, &mut self.unoccupied_opponent_cells, &settings);

        ExpressiveGameState::add_missiles(&mut self.player_buildings, &mut self.player_missiles);
        ExpressiveGameState::add_missiles(&mut self.opponent_buildings, &mut self.opponent_missiles);

        ExpressiveGameState::move_missiles(&mut self.player_missiles, |p| p.wrapping_move_right(),
                                           &mut self.opponent_buildings, &mut self.opponent,
                                           &mut self.unoccupied_opponent_cells,
                                           &settings);
        ExpressiveGameState::move_missiles(&mut self.opponent_missiles, |p| p.wrapping_move_left(),
                                           &mut self.player_buildings, &mut self.player,
                                           &mut self.unoccupied_player_cells,
                                           &settings);

        ExpressiveGameState::add_energy(&mut self.player);
        ExpressiveGameState::add_energy(&mut self.opponent);
        
        ExpressiveGameState::update_status(self);

        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.count_player_teslas() >= 2 }
    fn opponent_has_max_teslas(&self) -> bool { self.count_opponent_teslas() >= 2 }

    fn unoccupied_player_cell_count(&self) -> usize { self.unoccupied_player_cells.len() }
    fn unoccupied_opponent_cell_count(&self) -> usize { self.unoccupied_opponent_cells.len() }
    fn location_of_unoccupied_player_cell(&self, i: usize) -> Point  { self.unoccupied_player_cells[i] }
    fn location_of_unoccupied_opponent_cell(&self, i: usize) -> Point { self.unoccupied_opponent_cells[i] }
}

impl ExpressiveGameState {
    pub fn new(
        player: Player, opponent: Player,
        player_unconstructed_buildings: Vec<UnconstructedBuilding>, player_buildings: Vec<Building>,
        opponent_unconstructed_buildings: Vec<UnconstructedBuilding>, opponent_buildings: Vec<Building>,
        player_missiles: Vec<Missile>, opponent_missiles: Vec<Missile>,
        settings: &GameSettings) -> ExpressiveGameState {
        
        let unoccupied_player_cells = ExpressiveGameState::unoccupied_cells(
            &player_buildings, &player_unconstructed_buildings, Point::new(0, 0), Point::new(settings.size.x/2, settings.size.y)
        );
        let unoccupied_opponent_cells = ExpressiveGameState::unoccupied_cells(
            &opponent_buildings, &opponent_unconstructed_buildings, Point::new(settings.size.x/2, 0), Point::new(settings.size.x, settings.size.y)
        );
        ExpressiveGameState {
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

    fn perform_construct_command(unconstructed_buildings: &mut Vec<UnconstructedBuilding>, buildings: &mut Vec<Building>, player: &mut Player, unoccupied_cells: &mut Vec<Point>, settings: &GameSettings, command: Command, size: &Point) {
        if let Command::Build(p, b) = command {
            let blueprint = settings.building_settings(b);

            // This is used internally. I should not be making
            // invalid moves!
            debug_assert!(!buildings.iter().any(|b| b.pos == p));
            debug_assert!(p.x < size.x && p.y < size.y);
            debug_assert!(player.energy >= blueprint.price);
            debug_assert!(b != BuildingType::Tesla ||
                          (unconstructed_buildings.iter().filter(|b| b.weapon_damage == 20).count() +
                          buildings.iter().filter(|b| b.weapon_damage == 20).count() < 2));

            player.energy -= blueprint.price;
            unconstructed_buildings.push(UnconstructedBuilding::new(p, blueprint));
            
            let to_remove_index = unoccupied_cells.iter().position(|&pos| pos == p).unwrap();
            unoccupied_cells.swap_remove(to_remove_index);
        }
    }
    fn perform_deconstruct_command(unconstructed_buildings: &mut Vec<UnconstructedBuilding>, buildings: &mut Vec<Building>, player: &mut Player, unoccupied_cells: &mut Vec<Point>, command: Command) {
        if let Command::Deconstruct(p) = command {
            let to_remove_index = buildings.iter().position(|ref b| b.pos == p);
            let unconstructed_to_remove_index = unconstructed_buildings.iter().position(|ref b| b.pos == p);
            debug_assert!(to_remove_index.is_some() || unconstructed_to_remove_index.is_some());
            
            if let Some(i) = to_remove_index {
                player.energy_generated -= buildings[i].energy_generated_per_turn;
                buildings.swap_remove(i);
            }
            if let Some(i) = unconstructed_to_remove_index {
                unconstructed_buildings.swap_remove(i);
            }
            
            player.energy += 5;
            
            unoccupied_cells.push(p);
        }
    }

    fn update_construction(unconstructed_buildings: &mut Vec<UnconstructedBuilding>, buildings: &mut Vec<Building>, player: &mut Player) {
        let mut buildings_len = unconstructed_buildings.len();
        for i in (0..buildings_len).rev() {
            if unconstructed_buildings[i].is_constructed() {
                player.energy_generated += unconstructed_buildings[i].energy_generated_per_turn;
                buildings.push(unconstructed_buildings[i].to_building());
                buildings_len -= 1;
                unconstructed_buildings.swap(i, buildings_len);
            } else {
                unconstructed_buildings[i].construction_time_left -= 1
            }
        }
        unconstructed_buildings.truncate(buildings_len);
    }

    fn fire_teslas(player: &mut Player, player_buildings: &mut Vec<Building>, player_unoccupied_cells: &mut Vec<Point>, opponent: &mut Player, opponent_buildings: &mut Vec<Building>, opponent_unoccupied_cells: &mut Vec<Point>,settings: &GameSettings) {
        for tesla in player_buildings.iter_mut().filter(|b| b.weapon_damage == 20) {
            if tesla.weapon_cooldown_time_left > 0 {
                tesla.weapon_cooldown_time_left -= 1;
            } else if player.energy >= 100 {
                player.energy -= 100;
                tesla.weapon_cooldown_time_left = tesla.weapon_cooldown_period;

                if tesla.pos.x + 1 >= settings.size.x/2 {
                    opponent.health = opponent.health.saturating_sub(settings.tesla.weapon_damage);
                }
                'player_col_loop: for x in tesla.pos.x+1..tesla.pos.x+(settings.size.x/2)+2 {
                    for &y in [tesla.pos.y.saturating_sub(1), tesla.pos.y, tesla.pos.y.saturating_add(1)].iter() {
                        let target_point = Point::new(x, y);
                        for b in 0..opponent_buildings.len() {
                            if opponent_buildings[b].pos == target_point && opponent_buildings[b].health > 0 {
                                opponent_buildings[b].health = opponent_buildings[b].health.saturating_sub(settings.tesla.weapon_damage);
                                continue 'player_col_loop;
                            }
                        }
                    }
                }
            }
        }

        for tesla in opponent_buildings.iter_mut().filter(|b| b.weapon_damage == 20) {
            if tesla.weapon_cooldown_time_left > 0 {
                tesla.weapon_cooldown_time_left -= 1;
            } else if opponent.energy >= 100 {
                opponent.energy -= 100;
                tesla.weapon_cooldown_time_left = tesla.weapon_cooldown_period;
                
                if tesla.pos.x <= settings.size.x/2 {
                    player.health = player.health.saturating_sub(settings.tesla.weapon_damage);
                }
                'opponent_col_loop: for x in tesla.pos.x.saturating_sub((settings.size.x/2)+1)..tesla.pos.x {
                    for &y in [tesla.pos.y.saturating_sub(1), tesla.pos.y, tesla.pos.y.saturating_add(1)].iter() {
                        let target_point = Point::new(x, y);
                        for b in 0..player_buildings.len() {
                            if player_buildings[b].pos == target_point && player_buildings[b].health > 0 {
                                player_buildings[b].health = player_buildings[b].health.saturating_sub(settings.tesla.weapon_damage);
                                continue 'opponent_col_loop;
                            }
                        }
                    }
                }
            }
        }
        
        for building in player_buildings.iter().filter(|b| b.health == 0) {
            player_unoccupied_cells.push(building.pos);
            player.energy_generated -= building.energy_generated_per_turn;
        }
        player_buildings.retain(|b| b.health > 0);

        for building in opponent_buildings.iter().filter(|b| b.health == 0) {
            opponent_unoccupied_cells.push(building.pos);
            opponent.energy_generated -= building.energy_generated_per_turn;
        }
        opponent_buildings.retain(|b| b.health > 0);
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
        let mut missiles_len = missiles.len();
        'speed_loop: for _ in 0..settings.attack.weapon_speed {
            'missile_loop: for m in (0..missiles.len()).rev() {
                wrapping_move_fn(&mut missiles[m].pos);
                if missiles[m].pos.x >= settings.size.x {
                    opponent.health = opponent.health.saturating_sub(missiles[m].damage);

                    missiles_len -= 1;
                    missiles.swap(m, missiles_len);
                                        
                    continue 'missile_loop;
                }
                else {
                    for b in 0..opponent_buildings.len() {
                        if opponent_buildings[b].pos == missiles[m].pos {
                            opponent_buildings[b].health = opponent_buildings[b].health.saturating_sub(missiles[m].damage);

                            missiles_len -= 1;
                            missiles.swap(m, missiles_len);

                            if opponent_buildings[b].health == 0 {
                                unoccupied_cells.push(opponent_buildings[b].pos);
                                opponent.energy_generated -= opponent_buildings[b].energy_generated_per_turn;
                                opponent_buildings.swap_remove(b);
                            }
                            //after game engine bug fix, this should go back to missile_loop
                            continue 'missile_loop;
                        }
                    }
                }
            }
            missiles.truncate(missiles_len);
        }
    }

    fn add_energy(player: &mut Player) {
        player.energy += player.energy_generated;
    }

    fn update_status(state: &mut ExpressiveGameState) {
        let player_dead = state.player.health == 0;
        let opponent_dead = state.opponent.health == 0;
        state.status = match (player_dead, opponent_dead) {
            (true, true) => GameStatus::Draw,
            (false, true) => GameStatus::PlayerWon,
            (true, false) => GameStatus::OpponentWon,
            (false, false) => GameStatus::Continue,
        };
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

    pub fn count_player_teslas(&self) -> usize {
        self.player_unconstructed_buildings.iter().filter(|b| b.weapon_damage == 20).count() +
            self.player_buildings.iter().filter(|b| b.weapon_damage == 20).count()
    }

    pub fn count_opponent_teslas(&self) -> usize {
        self.opponent_unconstructed_buildings.iter().filter(|b| b.weapon_damage == 20).count() +
            self.opponent_buildings.iter().filter(|b| b.weapon_damage == 20).count()
    }
}

impl GameStatus {
    fn is_complete(&self) -> bool {
        *self != GameStatus::Continue
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
        self.weapon_damage > 0 && self.weapon_damage < 20
    }
}

