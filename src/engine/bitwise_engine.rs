use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings};
use engine::{GameStatus, Player, GameState};

const FULL_MAP_WIDTH: u8 = 16;
const SINGLE_MAP_WIDTH: u8 = FULL_MAP_WIDTH/2;
const MAX_CONCURRENT_MISSILES: usize = SINGLE_MAP_WIDTH as usize / 2;

const MISSILE_COOLDOWN: usize = 3;

const DEFENCE_HEALTH: usize = 4; // '20' health is 4 hits

const MAX_TESLAS: usize = 2;

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
    
    pub missiles: [(u64, u64); MAX_CONCURRENT_MISSILES],
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


impl GameState for BitwiseGameState {
    fn simulate(&mut self, settings: &GameSettings, player_command: Command, opponent_command: Command) -> GameStatus {
        BitwiseGameState::perform_command(settings, &mut self.player, &mut self.player_buildings, player_command);      BitwiseGameState::perform_command(settings, &mut self.opponent, &mut self.opponent_buildings, opponent_command);

        BitwiseGameState::update_construction(settings, &mut self.player_buildings);
        BitwiseGameState::update_construction(settings, &mut self.opponent_buildings);
        
        //TODO: Fire the TESLAS!

        BitwiseGameState::add_left_missiles(&mut self.player_buildings);
        BitwiseGameState::add_right_missiles(&mut self.opponent_buildings);

        BitwiseGameState::move_left_and_collide_missiles(settings, &mut self.player, &mut self.player_buildings, &mut self.opponent_buildings.missiles);
        BitwiseGameState::move_right_and_collide_missiles(settings, &mut self.opponent, &mut self.opponent_buildings, &mut self.player_buildings.missiles);

        BitwiseGameState::add_energy(settings, &mut self.player, &mut self.player_buildings);
        BitwiseGameState::add_energy(settings, &mut self.opponent, &mut self.opponent_buildings);

        self.update_status();
        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.player_buildings.count_teslas() >= MAX_TESLAS }
    fn opponent_has_max_teslas(&self) -> bool { self.opponent_buildings.count_teslas() >= MAX_TESLAS }

    fn unoccupied_player_cell_count(&self) -> usize { self.player_buildings.occupied.count_zeros() as usize }
    fn unoccupied_opponent_cell_count(&self) -> usize { self.opponent_buildings.occupied.count_zeros() as usize }
    fn location_of_unoccupied_player_cell(&self, i: usize) -> Point  {
        let bit = find_bit_index_from_rank(self.player_buildings.occupied, i as u64);
        let point = Point::new(bit%SINGLE_MAP_WIDTH, bit/SINGLE_MAP_WIDTH);
        debug_assert!(point.to_either_bitfield(SINGLE_MAP_WIDTH) & self.player_buildings.occupied == 0);
        point
    }
    fn location_of_unoccupied_opponent_cell(&self, i: usize) -> Point {
        let bit = find_bit_index_from_rank(self.opponent_buildings.occupied, i as u64);
        let point = Point::new(bit%SINGLE_MAP_WIDTH+SINGLE_MAP_WIDTH, bit/SINGLE_MAP_WIDTH);
        debug_assert!(point.to_either_bitfield(SINGLE_MAP_WIDTH) & self.opponent_buildings.occupied == 0);
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
    pub fn sort(&mut self) {
        for i in 0..MAX_CONCURRENT_MISSILES {
            for j in i+1..MAX_CONCURRENT_MISSILES {
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
    }

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
                let bitfield = p.to_either_bitfield(SINGLE_MAP_WIDTH);

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
                let deconstruct_mask = !(p.to_either_bitfield(SINGLE_MAP_WIDTH) & player_buildings.buildings[0]);
                
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
                let bitfield = pos.to_either_bitfield(SINGLE_MAP_WIDTH);
                
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
/*
    fn fire_teslas(settings: &GameSettings, player: &mut Player, player_buildings: &mut PlayerBuildings, opponent: &mut Player, opponent_buildings: &mut PlayerBuildings) {
        for tesla in player_buildings.tesla_cooldowns.iter_mut().filter(|t| t.active) {
            if tesla.cooldown > 0 {
                tesla.cooldown -= 1;
            } else if player.energy >= 100 {
                player.energy -= 100;
                tesla.cooldown = settings.tesla.weapon_cooldown_period;

                if tesla.pos.x + 1 >= settings.size.x/2 {
                    opponent.health = opponent.health.saturating_sub(settings.tesla.weapon_damage);
                }
                // TODO destroy some buildings?
                
            }
        }

        // TODO Only clean up the tesla cooldowns after this has been called in both directions
    }
*/

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


    fn move_left_and_collide_missiles(settings: &GameSettings, opponent: &mut Player, opponent_buildings: &mut PlayerBuildings, player_missiles: &mut [(u64, u64); MAX_CONCURRENT_MISSILES]) {
        for _ in 0..settings.attack.weapon_speed {
            for i in 0..player_missiles.len() {
                let about_to_hit_opponent = player_missiles[i].0 & LEFT_COL_MASK;
                let damage = about_to_hit_opponent.count_ones() as u8 * settings.attack.weapon_damage;
                opponent.health = opponent.health.saturating_sub(damage);
                player_missiles[i].0 = (player_missiles[i].0 & !LEFT_COL_MASK) >> 1;

                let swapping_sides = player_missiles[i].1 & LEFT_COL_MASK;
                player_missiles[i].0 |= swapping_sides << 7;
                player_missiles[i].1 = (player_missiles[i].1 & !LEFT_COL_MASK) >> 1;


                let mut hits = 0;
                for health_tier in (0..DEFENCE_HEALTH).rev() {
                    hits = opponent_buildings.buildings[health_tier] & player_missiles[i].0;
                    player_missiles[i].0 &= !hits;
                    opponent_buildings.buildings[health_tier] &= !hits;
                }

                let deconstruct_mask = !hits;
                opponent_buildings.energy_towers &= deconstruct_mask;
                for tier in 0..opponent_buildings.missile_towers.len() {
                    opponent_buildings.missile_towers[tier] &= deconstruct_mask;
                }
                for tesla in 0..opponent_buildings.tesla_cooldowns.len() {
                    if opponent_buildings.tesla_cooldowns[tesla].pos.to_either_bitfield(SINGLE_MAP_WIDTH) & deconstruct_mask == 0 {
                        opponent_buildings.tesla_cooldowns[tesla].active = false;
                    }
                }
                opponent_buildings.occupied &= deconstruct_mask;
            }
        }
    }

    fn move_right_and_collide_missiles(settings: &GameSettings, opponent: &mut Player, opponent_buildings: &mut PlayerBuildings, player_missiles: &mut [(u64, u64); MAX_CONCURRENT_MISSILES]) {
        for _ in 0..settings.attack.weapon_speed {
            for i in 0..player_missiles.len() {
                let about_to_hit_opponent = player_missiles[i].1 & RIGHT_COL_MASK;
                let damage = about_to_hit_opponent.count_ones() as u8 * settings.attack.weapon_damage;
                opponent.health = opponent.health.saturating_sub(damage);
                player_missiles[i].1 = (player_missiles[i].1 & !RIGHT_COL_MASK) << 1;

                let swapping_sides = player_missiles[i].0 & RIGHT_COL_MASK;
                player_missiles[i].1 |= swapping_sides >> 7;
                player_missiles[i].0 = (player_missiles[i].0 & !RIGHT_COL_MASK) << 1;

                
                let mut hits = 0;
                for health_tier in (0..DEFENCE_HEALTH).rev() {
                    hits = opponent_buildings.buildings[health_tier] & player_missiles[i].1;
                    player_missiles[i].1 &= !hits;
                    opponent_buildings.buildings[health_tier] &= !hits;
                }

                let deconstruct_mask = !hits;
                opponent_buildings.energy_towers &= deconstruct_mask;
                for tier in 0..opponent_buildings.missile_towers.len() {
                    opponent_buildings.missile_towers[tier] &= deconstruct_mask;
                }
                for tesla in 0..opponent_buildings.tesla_cooldowns.len() {
                    if opponent_buildings.tesla_cooldowns[tesla].pos.to_either_bitfield(SINGLE_MAP_WIDTH) & deconstruct_mask == 0 {
                        opponent_buildings.tesla_cooldowns[tesla].active = false;
                    }
                }
                opponent_buildings.occupied &= deconstruct_mask;
            }
        }
    }
    
    
    fn add_energy(settings: &GameSettings, player: &mut Player, player_buildings: &mut PlayerBuildings) {
        player.energy_generated = settings.energy_income + player_buildings.energy_towers.count_ones() as u16 * settings.energy.energy_generated_per_turn;
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
