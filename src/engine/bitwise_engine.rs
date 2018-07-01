use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings};
use engine::{GameStatus, Player, GameState};

const MAP_WIDTH: usize = 16;

const MISSILE_COOLDOWN: usize = 3;

const DEFENCE_HEALTH: usize = 4; // '20' health is 4 hits

const MAX_TESLAS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitwiseGameState {
    pub status: GameStatus,
    pub player: Player,
    pub opponent: Player,
    pub player_buildings: PlayerBuildings,
    pub opponent_buildings: PlayerBuildings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerBuildings {
    pub unconstructed: Vec<UnconstructedBuilding>,
    pub buildings: [u64; DEFENCE_HEALTH],
    pub occupied: u64,
    
    pub energy_towers: u64,
    pub missile_towers: [u64; MISSILE_COOLDOWN+1],
    
    pub missiles: [(u64, u64); MAP_WIDTH/4],
    pub tesla_cooldowns: [TeslaCooldown; MAX_TESLAS]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnconstructedBuilding {
    pub pos: Point,
    pub construction_time_left: u8,
    pub building_type: BuildingType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeslaCooldown {
    pub active: bool,
    pub pos: Point,
    pub cooldown: u8
}


const EMPTY: [Point; 0] = [];

impl GameState for BitwiseGameState {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus {
        BitwiseGameState::perform_command(settings, &mut self.player, &mut self.player_buildings, player_command);      BitwiseGameState::perform_command(settings, &mut self.opponent, &mut self.opponent_buildings, opponent_command);

        BitwiseGameState::update_construction(settings, &mut self.player_buildings);
        BitwiseGameState::update_construction(settings, &mut self.opponent_buildings);
        
        //TODO: Fire the TESLAS!

        BitwiseGameState::add_left_missiles(&mut self.player_buildings);
        BitwiseGameState::add_right_missiles(&mut self.opponent_buildings);
        //TODO: Move and collide missiles

        BitwiseGameState::add_energy(settings, &mut self.player, &mut self.player_buildings);
        BitwiseGameState::add_energy(settings, &mut self.opponent, &mut self.opponent_buildings);

        self.update_status();
        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.player_buildings.count_teslas() >= MAX_TESLAS }
    fn opponent_has_max_teslas(&self) -> bool { self.opponent_buildings.count_teslas() >= MAX_TESLAS }
    fn unoccupied_player_cells(&self) -> &[Point] { &EMPTY } //TODO
    fn unoccupied_opponent_cells(&self) -> &[Point] { &EMPTY } //TODO
}

impl BitwiseGameState {
    pub fn new(
        player: Player, opponent: Player,
        player_buildings: PlayerBuildings, opponent_buildings: PlayerBuildings
    ) -> BitwiseGameState {
        BitwiseGameState {
            status: GameStatus::Continue,
            player, opponent,
            player_buildings, opponent_buildings
        }
    }

    fn perform_command(settings: &GameSettings, player: &mut Player, player_buildings: &mut PlayerBuildings, command: Command) {
        match command {
            Command::Nothing => {},
            Command::Build(p, b) => {
                let blueprint = settings.building_settings(b);
                let bitfield = p.to_either_bitfield(settings.size.x);

                // This is used internally. I should not be making
                // invalid moves!
                debug_assert!(player_buildings.buildings[0] & bitfield == 0);
                debug_assert!(p.x < settings.size.x && p.y < settings.size.y);
                debug_assert!(player.energy >= blueprint.price);
                debug_assert!(b != BuildingType::Tesla ||
                              player_buildings.count_teslas() < MAX_TESLAS);

                player.energy -= blueprint.price;
                player_buildings.unconstructed.push(UnconstructedBuilding {
                    pos: p,
                    construction_time_left: blueprint.construction_time,
                    building_type: b
                });
                player_buildings.occupied |= bitfield;
            },
            Command::Deconstruct(p) => {
                let unconstructed_to_remove_index = player_buildings.unconstructed.iter().position(|ref b| b.pos == p);
                let deconstruct_mask = !(p.to_either_bitfield(settings.size.x) & player_buildings.buildings[0]);
                
                debug_assert!(deconstruct_mask != 0 || unconstructed_to_remove_index.is_some());
                
                if let Some(i) = unconstructed_to_remove_index {
                    player_buildings.unconstructed.swap_remove(i);
                }
                
                player.energy += 5;
                
                for tier in 0..player_buildings.buildings.len() {
                    player_buildings.buildings[tier] &= deconstruct_mask;
                }
                player_buildings.energy_towers &= deconstruct_mask;
                for tier in 0..player_buildings.missile_towers.len() {
                    player_buildings.missile_towers[tier] &= deconstruct_mask;
                }
                for tesla in 0..player_buildings.tesla_cooldowns.len() {
                    if player_buildings.tesla_cooldowns[tesla].pos == p {
                        player_buildings.tesla_cooldowns[tesla].active = false;
                    }
                }
                player_buildings.occupied &= deconstruct_mask;
            }
        }
    }

    fn update_construction(settings: &GameSettings, player_buildings: &mut PlayerBuildings) {
        let mut buildings_len = player_buildings.unconstructed.len();
        for i in (0..buildings_len).rev() {
            if player_buildings.unconstructed[i].construction_time_left == 0 {
                let building_type = player_buildings.unconstructed[i].building_type;
                let blueprint = settings.building_settings(building_type);
                let pos = player_buildings.unconstructed[i].pos;
                let bitfield = pos.to_either_bitfield(settings.size.x);
                
                for health_tier in 0..4 {
                    if blueprint.health > health_tier*5 {
                        player_buildings.buildings[health_tier as usize] |= bitfield;
                    }
                }
                if building_type == BuildingType::Energy {
                    player_buildings.energy_towers |= bitfield;
                }
                if building_type == BuildingType::Attack {
                    player_buildings.missile_towers[0] |= bitfield;
                }
                if building_type == BuildingType::Tesla {
                    let ref mut tesla_cooldown = if player_buildings.tesla_cooldowns[0].active {
                        player_buildings.tesla_cooldowns[1]
                    } else {
                        player_buildings.tesla_cooldowns[0]
                    };
                    tesla_cooldown.active = true;
                    tesla_cooldown.pos = pos;
                    tesla_cooldown.cooldown = 0;
                }
                
                buildings_len -= 1;
                player_buildings.unconstructed.swap(i, buildings_len);
            } else {
                player_buildings.unconstructed[i].construction_time_left -= 1
            }
        }
        player_buildings.unconstructed.truncate(buildings_len);
    }


    fn add_left_missiles(player_buildings: &mut PlayerBuildings) {
        let mut missiles = player_buildings.missile_towers[0];
        for mut tier in player_buildings.missiles.iter_mut() {
            let setting = !tier.0 & missiles;
            tier.0 |= setting;
            missiles &= !setting;
        }

        BitwiseGameState::rotate_missile_towers(player_buildings);
    }

    fn add_right_missiles(player_buildings: &mut PlayerBuildings) {
        let mut missiles = player_buildings.missile_towers[0];
        for mut tier in player_buildings.missiles.iter_mut() {
            let setting = !tier.1 & missiles;
            tier.1 |= setting;
            missiles &= !setting;
        }

        BitwiseGameState::rotate_missile_towers(player_buildings);
    }

    fn rotate_missile_towers(player_buildings: &mut PlayerBuildings) {
        let zero = player_buildings.missile_towers[0];
        for i in 1..player_buildings.missile_towers.len() {
            player_buildings.missile_towers[i-1] = player_buildings.missile_towers[i];
        }
        let end = player_buildings.missile_towers.len()-1;
        player_buildings.missile_towers[end] = zero;
    }


    fn move_and_collide_missiles_left(settings: &GameSettings, player_buildings: &mut PlayerBuildings, opponent: &mut Player) {
        for _ in 0..settings.attack.weapon_speed {
            for i in 0..player_buildings.missiles.len() {
                //TODO this isn't so simple...
                //collide some with the player, others jump the boundary
                player_buildings.missiles[i].0 = player_buildings.missiles[i].0 << 1;
                //TODO Collide with buildings
            }
        }
    }
    
    
    fn add_energy(settings: &GameSettings, player: &mut Player, player_buildings: &mut PlayerBuildings) {
        player.energy_generated = player_buildings.energy_towers.count_ones() as u16 * settings.energy.energy_generated_per_turn;
        player.energy += player.energy_generated;
    }

    fn update_status(&mut self) {
        let player_dead = self.player.health == 0;
        let opponent_dead = self.opponent.health == 0;
        self.status = match (player_dead, opponent_dead) {
            (true, true) => GameStatus::Draw,
            (false, true) => GameStatus::PlayerWon,
            (true, false) => GameStatus::OpponentWon,
            (false, false) => GameStatus::Continue,
        };
    }

}

impl PlayerBuildings {
    pub fn count_teslas(&self) -> usize {
        self.tesla_cooldowns.iter().filter(|t| t.active).count()
    }

    pub fn empty() -> PlayerBuildings {
        PlayerBuildings {
            unconstructed: Vec::with_capacity(4),
            buildings: [0; 4],
            occupied: 0,
            energy_towers: 0,
            missile_towers: [0; 4],
            missiles: [(0,0); 4],
            tesla_cooldowns: [TeslaCooldown::empty(); 2]
        }
    }
}

impl TeslaCooldown {
    pub fn empty() -> TeslaCooldown {
        TeslaCooldown {
            active: false,
            pos: Point::new(0,0),
            cooldown: 0
        }
    }
}
