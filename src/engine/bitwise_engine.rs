use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings};
use engine::constants::*;
use engine::{GameStatus, Player, GameState};

const LEFT_COL_MASK: u64 = 0x0101010101010101;
const RIGHT_COL_MASK: u64 = 0x8080808080808080;

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
    
    pub missiles: [(u64, u64); MISSILE_MAX_SINGLE_CELL],
    pub tesla_cooldowns: [TeslaCooldown; TESLA_MAX]
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


impl GameState for BitwiseGameState {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus {
        BitwiseGameState::perform_command(settings, &mut self.player, &mut self.player_buildings, player_command);      BitwiseGameState::perform_command(settings, &mut self.opponent, &mut self.opponent_buildings, opponent_command);

        BitwiseGameState::update_construction(settings, &mut self.player_buildings);
        BitwiseGameState::update_construction(settings, &mut self.opponent_buildings);
        
        BitwiseGameState::fire_teslas(&mut self.player, &mut self.player_buildings, &mut self.opponent, &mut self.opponent_buildings);

        BitwiseGameState::add_left_missiles(&mut self.player_buildings);
        BitwiseGameState::add_right_missiles(&mut self.opponent_buildings);

        BitwiseGameState::move_left_and_collide_missiles(&mut self.player, &mut self.player_buildings, &mut self.opponent_buildings.missiles);
        BitwiseGameState::move_right_and_collide_missiles(&mut self.opponent, &mut self.opponent_buildings, &mut self.player_buildings.missiles);

        BitwiseGameState::add_energy(&mut self.player, &mut self.player_buildings);
        BitwiseGameState::add_energy(&mut self.opponent, &mut self.opponent_buildings);

        self.update_status();
        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.player_buildings.count_teslas() >= TESLA_MAX }
    fn opponent_has_max_teslas(&self) -> bool { self.opponent_buildings.count_teslas() >= TESLA_MAX }

    fn unoccupied_player_cell_count(&self) -> usize { self.player_buildings.occupied.count_zeros() as usize }
    fn unoccupied_opponent_cell_count(&self) -> usize { self.opponent_buildings.occupied.count_zeros() as usize }
    fn location_of_unoccupied_player_cell(&self, i: usize) -> Point  {
        let bit = find_bit_index_from_rank(self.player_buildings.occupied, i as u64);
        let point = Point::new(bit%SINGLE_MAP_WIDTH, bit/SINGLE_MAP_WIDTH);
        debug_assert!(point.to_either_bitfield() & self.player_buildings.occupied == 0);
        point
    }
    fn location_of_unoccupied_opponent_cell(&self, i: usize) -> Point {
        let bit = find_bit_index_from_rank(self.opponent_buildings.occupied, i as u64);
        let point = Point::new(bit%SINGLE_MAP_WIDTH+SINGLE_MAP_WIDTH, bit/SINGLE_MAP_WIDTH);
        debug_assert!(point.to_either_bitfield() & self.opponent_buildings.occupied == 0);
        point
    }
}

fn find_bit_index_from_rank(occupied: u64, i: u64) -> u8 {
    // Adapted from https://graphics.stanford.edu/~seander/bithacks.html#SelectPosFromMSBRank
    let v = !occupied;
    
    let mut r = v.count_ones() as u64 - i as u64;

    let a: u64 =  v - ((v >> 1) & !0u64/3);
    let b: u64 = (a & (!0u64/5)) + ((a >> 2) & (!0u64/5));
    let c: u64 = (b + (b >> 4)) & (!0u64/0x11);
    let d: u64 = (c + (c >> 8)) & (!0u64/0x101);
    let mut t: u64 = (d >> 32) + (d >> 48);

    let mut s: u64 = 64;
    s -= (t.wrapping_sub(r) & 256) >> 3; r -= t & (t.wrapping_sub(r) >> 8);
    t  = (d >> (s - 16)) & 0xff;
    s -= (t.wrapping_sub(r) & 256) >> 4; r -= t & (t.wrapping_sub(r) >> 8);
    t  = (c >> (s - 8)) & 0xf;
    s -= (t.wrapping_sub(r) & 256) >> 5; r -= t & (t.wrapping_sub(r) >> 8);
    t  = (b >> (s - 4)) & 0x7;
    s -= (t.wrapping_sub(r) & 256) >> 6; r -= t & (t.wrapping_sub(r) >> 8);
    t  = (a >> (s - 2)) & 0x3;
    s -= (t.wrapping_sub(r) & 256) >> 7; r -= t & (t.wrapping_sub(r) >> 8);
    t  = (v >> (s - 1)) & 0x1;
    s -= (t.wrapping_sub(r) & 256) >> 8;
    s = 65 - s;

    let bit = 64 - s as u8;
    bit
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

    /**
     * Like with the expressive, this is to make things more
     * comparable when writing tests, not for actual use in the
     * engine.
     */
    #[cfg(debug_assertions)]
    pub fn sort(&mut self) {
        for i in 0..MISSILE_MAX_SINGLE_CELL {
            for j in i+1..MISSILE_MAX_SINGLE_CELL {
                let move_down1 = !self.player_buildings.missiles[i].0 & self.player_buildings.missiles[j].0;
                self.player_buildings.missiles[i].0 |= move_down1;
                self.player_buildings.missiles[j].0 &= !move_down1;

                let move_down2 = !self.player_buildings.missiles[i].1 & self.player_buildings.missiles[j].1;
                self.player_buildings.missiles[i].1 |= move_down2;
                self.player_buildings.missiles[j].1 &= !move_down2;

                let move_down3 = !self.opponent_buildings.missiles[i].0 & self.opponent_buildings.missiles[j].0;
                self.opponent_buildings.missiles[i].0 |= move_down3;
                self.opponent_buildings.missiles[j].0 &= !move_down3;

                let move_down4 = !self.opponent_buildings.missiles[i].1 & self.opponent_buildings.missiles[j].1;
                self.opponent_buildings.missiles[i].1 |= move_down4;
                self.opponent_buildings.missiles[j].1 &= !move_down4;
            }
        }

        self.player_buildings.unconstructed.sort_by_key(|b| b.pos);
        self.opponent_buildings.unconstructed.sort_by_key(|b| b.pos);

        for tesla in self.player_buildings.tesla_cooldowns.iter_mut() {
            if !tesla.active {
                tesla.pos = Point::new(0,0);
                tesla.cooldown = 0;
            }
        }
        for tesla in self.opponent_buildings.tesla_cooldowns.iter_mut() {
            if !tesla.active {
                tesla.pos = Point::new(0,0);
                tesla.cooldown = 0;
            }
        }

        self.player_buildings.tesla_cooldowns.sort_by_key(|b| (!b.active, b.pos));
        self.opponent_buildings.tesla_cooldowns.sort_by_key(|b| (!b.active, b.pos));
    }

    #[cfg(debug_assertions)]
    pub fn sorted(&self) -> BitwiseGameState {
        let mut res = self.clone();
        res.sort();
        res
    }

    fn perform_command(settings: &GameSettings, player: &mut Player, player_buildings: &mut PlayerBuildings, command: Command) {
        match command {
            Command::Nothing => {},
            Command::Build(p, b) => {
                let blueprint = settings.building_settings(b);
                let bitfield = p.to_either_bitfield();

                // This is used internally. I should not be making
                // invalid moves!
                debug_assert!(player_buildings.buildings[0] & bitfield == 0);
                debug_assert!(p.x < settings.size.x && p.y < settings.size.y);
                debug_assert!(player.energy >= blueprint.price);
                debug_assert!(b != BuildingType::Tesla ||
                              player_buildings.count_teslas() < TESLA_MAX);

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
                let deconstruct_mask = !(p.to_either_bitfield() & player_buildings.buildings[0]);
                
                debug_assert!(deconstruct_mask != 0 || unconstructed_to_remove_index.is_some());
                
                if let Some(i) = unconstructed_to_remove_index {
                    player_buildings.unconstructed.swap_remove(i);
                }
                
                player.energy += DECONSTRUCT_ENERGY;
                
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
                //TODO: Avoid blueprint here?
                let building_type = player_buildings.unconstructed[i].building_type;
                let blueprint = settings.building_settings(building_type);
                
                let pos = player_buildings.unconstructed[i].pos;
                let bitfield = pos.to_either_bitfield();
                
                for health_tier in 0..DEFENCE_HEALTH as u8 {
                    if blueprint.health > health_tier*MISSILE_DAMAGE {
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
                        &mut player_buildings.tesla_cooldowns[1]
                    } else {
                        &mut player_buildings.tesla_cooldowns[0]
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

    fn fire_teslas(player: &mut Player, player_buildings: &mut PlayerBuildings, opponent: &mut Player, opponent_buildings: &mut PlayerBuildings) {

        #[cfg(debug_assertions)]
        {
            player_buildings.tesla_cooldowns.sort_by_key(|b| (!b.active, b.pos));
            opponent_buildings.tesla_cooldowns.sort_by_key(|b| (!b.active, b.pos));
        }

        for tesla in player_buildings.tesla_cooldowns.iter_mut().filter(|t| t.active) {
            if tesla.cooldown > 0 {
                tesla.cooldown -= 1;
            } else if player.energy >= TESLA_FIRING_ENERGY {
                player.energy -= TESLA_FIRING_ENERGY;
                tesla.cooldown = TESLA_COOLDOWN;

                if tesla.pos.x + 1 >= SINGLE_MAP_WIDTH {
                    opponent.health = opponent.health.saturating_sub(TESLA_DAMAGE);
                }

                let missed_cells = ((SINGLE_MAP_WIDTH - tesla.pos.x) as u32).saturating_sub(2);
                
                let top_row = if tesla.pos.y == 0 { 0 } else { tesla.pos.y - 1 };
                let top_row_mask = 255u64 << (top_row * SINGLE_MAP_WIDTH);
                let mut destroy_mask = top_row_mask.wrapping_shr(missed_cells) & top_row_mask;

                for _ in 0..(if tesla.pos.y == 0 || tesla.pos.y == MAP_HEIGHT-1 { 2 } else { 3 }) {
                    let hits = destroy_mask & opponent_buildings.buildings[0];
                    destroy_mask &= !hits;
                    BitwiseGameState::destroy_buildings(opponent_buildings, hits);
                    destroy_mask = destroy_mask << SINGLE_MAP_WIDTH;
                }
            }
        }

        for tesla in opponent_buildings.tesla_cooldowns.iter_mut().filter(|t| t.active) {
            if tesla.cooldown > 0 {
                tesla.cooldown -= 1;
            } else if opponent.energy >= TESLA_FIRING_ENERGY {
                opponent.energy -= TESLA_FIRING_ENERGY;
                tesla.cooldown = TESLA_COOLDOWN;

                if tesla.pos.x <= SINGLE_MAP_WIDTH {
                    player.health = player.health.saturating_sub(TESLA_DAMAGE);
                }

                let missed_cells = ((tesla.pos.x - SINGLE_MAP_WIDTH) as u32).saturating_sub(1);
                
                let top_row = if tesla.pos.y == 0 { 0 } else { tesla.pos.y - 1 };
                let top_row_mask = 255u64 << (top_row * SINGLE_MAP_WIDTH);
                let mut destroy_mask = top_row_mask.wrapping_shl(missed_cells) & top_row_mask;

                for _ in 0..(if tesla.pos.y == 0 || tesla.pos.y == MAP_HEIGHT-1 { 2 } else { 3 }) {
                    let hits = destroy_mask & player_buildings.buildings[0];
                    destroy_mask &= !hits;
                    BitwiseGameState::destroy_buildings(player_buildings, hits);
                    destroy_mask = destroy_mask << SINGLE_MAP_WIDTH;
                }                
            }
        }

        BitwiseGameState::update_tesla_activity(player_buildings);
        BitwiseGameState::update_tesla_activity(opponent_buildings);
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

    //TODO: Add a pointer and stop rotating here
    fn rotate_missile_towers(player_buildings: &mut PlayerBuildings) {
        let zero = player_buildings.missile_towers[0];
        for i in 1..player_buildings.missile_towers.len() {
            player_buildings.missile_towers[i-1] = player_buildings.missile_towers[i];
        }
        let end = player_buildings.missile_towers.len()-1;
        player_buildings.missile_towers[end] = zero;
    }


    //TODO: Can I rearrange my bitfields to make these two functions one thing?
    fn move_left_and_collide_missiles(opponent: &mut Player, opponent_buildings: &mut PlayerBuildings, player_missiles: &mut [(u64, u64); MISSILE_MAX_SINGLE_CELL]) {
        for _ in 0..MISSILE_SPEED {
            for i in 0..MISSILE_MAX_SINGLE_CELL {
                let about_to_hit_opponent = player_missiles[i].0 & LEFT_COL_MASK;
                let damage = about_to_hit_opponent.count_ones() as u8 * MISSILE_DAMAGE;
                opponent.health = opponent.health.saturating_sub(damage);
                player_missiles[i].0 = (player_missiles[i].0 & !LEFT_COL_MASK) >> 1;

                let swapping_sides = player_missiles[i].1 & LEFT_COL_MASK;
                player_missiles[i].0 |= swapping_sides << (SINGLE_MAP_WIDTH-1);
                player_missiles[i].1 = (player_missiles[i].1 & !LEFT_COL_MASK) >> 1;


                let mut hits = 0;
                for health_tier in (0..DEFENCE_HEALTH).rev() {
                    hits = opponent_buildings.buildings[health_tier] & player_missiles[i].0;
                    player_missiles[i].0 &= !hits;
                    opponent_buildings.buildings[health_tier] &= !hits;
                }

                BitwiseGameState::destroy_buildings(opponent_buildings, hits);
                BitwiseGameState::update_tesla_activity(opponent_buildings);
            }
        }
    }

    fn move_right_and_collide_missiles(opponent: &mut Player, opponent_buildings: &mut PlayerBuildings, player_missiles: &mut [(u64, u64); MISSILE_MAX_SINGLE_CELL]) {
        for _ in 0..MISSILE_SPEED {
            for i in 0..MISSILE_MAX_SINGLE_CELL {
                let about_to_hit_opponent = player_missiles[i].1 & RIGHT_COL_MASK;
                let damage = about_to_hit_opponent.count_ones() as u8 * MISSILE_DAMAGE;
                opponent.health = opponent.health.saturating_sub(damage);
                player_missiles[i].1 = (player_missiles[i].1 & !RIGHT_COL_MASK) << 1;

                let swapping_sides = player_missiles[i].0 & RIGHT_COL_MASK;
                player_missiles[i].1 |= swapping_sides >> (SINGLE_MAP_WIDTH-1);
                player_missiles[i].0 = (player_missiles[i].0 & !RIGHT_COL_MASK) << 1;

                
                let mut hits = 0;
                for health_tier in (0..DEFENCE_HEALTH).rev() {
                    hits = opponent_buildings.buildings[health_tier] & player_missiles[i].1;
                    player_missiles[i].1 &= !hits;
                    opponent_buildings.buildings[health_tier] &= !hits;
                }

                BitwiseGameState::destroy_buildings(opponent_buildings, hits);
                BitwiseGameState::update_tesla_activity(opponent_buildings);
            }
        }
    }

    fn destroy_buildings(buildings: &mut PlayerBuildings, hit_mask: u64) {
        let deconstruct_mask = !hit_mask;
        
        buildings.energy_towers &= deconstruct_mask;
        for tier in buildings.missile_towers.iter_mut() {
            *tier &= deconstruct_mask;
        }
        for tier in buildings.buildings.iter_mut() {
            *tier &= deconstruct_mask;
        }
        buildings.occupied &= deconstruct_mask;
    }

    fn update_tesla_activity(buildings: &mut PlayerBuildings) {
        for tesla in buildings.tesla_cooldowns.iter_mut().filter(|t| t.active) {
            tesla.active = (tesla.pos.to_either_bitfield() & buildings.occupied) != 0;
        }
    }
    
    
    fn add_energy(player: &mut Player, player_buildings: &mut PlayerBuildings) {
        player.energy_generated = ENERGY_GENERATED_BASE + player_buildings.energy_towers.count_ones() as u16 * ENERGY_GENERATED_TOWER;
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
            + self.unconstructed.iter().filter(|t| t.building_type == BuildingType::Tesla).count()
    }

    pub fn empty() -> PlayerBuildings {
        PlayerBuildings {
            unconstructed: Vec::with_capacity(4),
            buildings: [0; DEFENCE_HEALTH],
            occupied: 0,
            energy_towers: 0,
            missile_towers: [0; MISSILE_COOLDOWN+1],
            missiles: [(0,0); MISSILE_MAX_SINGLE_CELL],
            tesla_cooldowns: [TeslaCooldown::empty(); TESLA_MAX]
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
