use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::constants::*;
use engine::status::GameStatus;

use arrayvec::ArrayVec;

const LEFT_COL_MASK: u64 = 0x0101_0101_0101_0101;
const RIGHT_COL_MASK: u64 = 0x8080_8080_8080_8080;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitwiseGameState {
    pub status: GameStatus,
    pub player: Player,
    pub opponent: Player,
    pub round: u16
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub energy: u16,
    pub health: u8,
    pub unconstructed: ArrayVec<[UnconstructedBuilding; MAX_CONCURRENT_CONSTRUCTION]>,
    pub buildings: [u64; DEFENCE_HEALTH],
    pub occupied: u64,
    
    pub energy_towers: u64,

    pub missile_towers: [u64; MISSILE_COOLDOWN_STATES],
    pub firing_tower: usize,
    
    pub missiles: [(u64, u64); MISSILE_MAX_SINGLE_CELL],
    pub tesla_cooldowns: ArrayVec<[TeslaCooldown; TESLA_MAX]>,

    pub iron_curtain_available: bool,
    pub iron_curtain_remaining: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnconstructedBuilding {
    pub pos: Point,
    pub construction_time_left: u8,
    pub building_type: BuildingType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeslaCooldown {
    pub pos: Point,
    pub cooldown: u8,
    pub age: u16
}


impl BitwiseGameState {
    pub fn simulate(&mut self, player_command: Command, opponent_command: Command) -> GameStatus {
        self.player.perform_command(player_command);
        self.opponent.perform_command(opponent_command);

        self.player.update_construction();
        self.opponent.update_construction();

        self.player.add_missiles();
        self.opponent.add_missiles();

        BitwiseGameState::fire_teslas(&mut self.player, &mut self.opponent);

        BitwiseGameState::move_and_collide_missiles(&mut self.player, &mut self.opponent.missiles);
        BitwiseGameState::move_and_collide_missiles(&mut self.opponent, &mut self.player.missiles);

        BitwiseGameState::add_energy(&mut self.player);
        BitwiseGameState::add_energy(&mut self.opponent);

        BitwiseGameState::update_iron_curtain(&mut self.player, self.round);
        BitwiseGameState::update_iron_curtain(&mut self.opponent, self.round);

        self.round += 1;

        self.update_status();
        self.status
    }
}

fn find_bit_index_from_rank(occupied: u64, i: u64) -> u8 {
    // Adapted from https://graphics.stanford.edu/~seander/bithacks.html#SelectPosFromMSBRank
    let v = !occupied;
    
    let mut r = u64::from(v.count_ones()) - i;

    let a: u64 =  v - ((v >> 1) & (!0u64/3));
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

    64 - s as u8
}

impl BitwiseGameState {
    pub fn new(
        player: Player, opponent: Player,
        round: u16
    ) -> BitwiseGameState {
        BitwiseGameState {
            status: GameStatus::Continue,
            player, opponent,
            round
        }
    }

    /**
     * This is to make things more comparable when writing tests, not
     * for actual use in the engine.
     */
    #[cfg(debug_assertions)]
    pub fn sort(&mut self) {
        for i in 0..MISSILE_MAX_SINGLE_CELL {
            for j in i+1..MISSILE_MAX_SINGLE_CELL {
                let move_down1 = !self.player.missiles[i].0 & self.player.missiles[j].0;
                self.player.missiles[i].0 |= move_down1;
                self.player.missiles[j].0 &= !move_down1;

                let move_down2 = !self.player.missiles[i].1 & self.player.missiles[j].1;
                self.player.missiles[i].1 |= move_down2;
                self.player.missiles[j].1 &= !move_down2;

                let move_down3 = !self.opponent.missiles[i].0 & self.opponent.missiles[j].0;
                self.opponent.missiles[i].0 |= move_down3;
                self.opponent.missiles[j].0 &= !move_down3;

                let move_down4 = !self.opponent.missiles[i].1 & self.opponent.missiles[j].1;
                self.opponent.missiles[i].1 |= move_down4;
                self.opponent.missiles[j].1 &= !move_down4;
            }
        }

        self.player.unconstructed.sort_by_key(|b| b.pos);
        self.opponent.unconstructed.sort_by_key(|b| b.pos);

        self.player.tesla_cooldowns.sort_by_key(|b| b.pos);
        self.opponent.tesla_cooldowns.sort_by_key(|b| b.pos);


        while self.player.firing_tower > 0 {
            self.player.firing_tower -= 1;
            let zero = self.player.missile_towers[0];
            for i in 1..self.player.missile_towers.len() {
                self.player.missile_towers[i-1] = self.player.missile_towers[i];
            }
            let end = self.player.missile_towers.len()-1;
            self.player.missile_towers[end] = zero;
        }
        while self.opponent.firing_tower > 0 {
            self.opponent.firing_tower -= 1;
            let zero = self.opponent.missile_towers[0];
            for i in 1..self.opponent.missile_towers.len() {
                self.opponent.missile_towers[i-1] = self.opponent.missile_towers[i];
            }
            let end = self.opponent.missile_towers.len()-1;
            self.opponent.missile_towers[end] = zero;
        }
    }

    #[cfg(debug_assertions)]
    pub fn sorted(&self) -> BitwiseGameState {
        let mut res = self.clone();
        res.sort();
        res
    }

    fn update_iron_curtain(player: &mut Player, round: u16) {
        if round != 0 && round % IRON_CURTAIN_UNLOCK_INTERVAL == 0 {
            player.iron_curtain_available = true;
        }
        player.iron_curtain_remaining = player.iron_curtain_remaining.saturating_sub(1);
    }
    
    fn fire_teslas(player: &mut Player, opponent: &mut Player) {
        BitwiseGameState::fire_single_players_teslas_without_cleanup(player, opponent);
        BitwiseGameState::fire_single_players_teslas_without_cleanup(opponent, player);

        BitwiseGameState::update_tesla_activity(player);
        BitwiseGameState::update_tesla_activity(opponent);
    }

    fn fire_single_players_teslas_without_cleanup(player: &mut Player, opponent: &mut Player) {
        player.tesla_cooldowns.sort_unstable_by(|a, b| b.age.cmp(&a.age));
        for tesla in player.tesla_cooldowns.iter_mut() {
            tesla.age += 1;
            if tesla.cooldown > 0 {
                tesla.cooldown -= 1;
            } else if player.energy >= TESLA_FIRING_ENERGY && opponent.iron_curtain_remaining > 0 {
                player.energy -= TESLA_FIRING_ENERGY;
                tesla.cooldown = TESLA_COOLDOWN;
            } else if player.energy >= TESLA_FIRING_ENERGY {
                player.energy -= TESLA_FIRING_ENERGY;
                tesla.cooldown = TESLA_COOLDOWN;

                if tesla.pos.to_either_bitfield() & RIGHT_COL_MASK != 0 {
                    opponent.health = opponent.health.saturating_sub(TESLA_DAMAGE);
                }

                let x = tesla.pos.x();
                let y = tesla.pos.y();
                let missed_cells = (u32::from(SINGLE_MAP_WIDTH - x)).saturating_sub(2);
                
                let top_row = y.saturating_sub(1);
                let top_row_mask = 255u64 << (top_row * SINGLE_MAP_WIDTH);
                let mut destroy_mask = top_row_mask.wrapping_shl(missed_cells) & top_row_mask;

                let mut hits = 0;
                for _ in 0..(if y == 0 || y == MAP_HEIGHT-1 { 2 } else { 3 }) {
                    hits |= destroy_mask & opponent.buildings[0];
                    destroy_mask &= !hits;
                    destroy_mask <<= SINGLE_MAP_WIDTH;
                }
                BitwiseGameState::destroy_buildings(opponent, hits);
            }
        }
    }

    fn move_and_collide_missiles(opponent: &mut Player, player_missiles: &mut [(u64, u64); MISSILE_MAX_SINGLE_CELL]) {
        let mut destroyed = 0;
        let mut damaging = 0;
        for _ in 0..MISSILE_SPEED {
            for missile in player_missiles.iter_mut() {
                let swapping_sides = if opponent.iron_curtain_remaining > 0 { 0 } else { missile.0 & RIGHT_COL_MASK };
                let about_to_hit_opponent = missile.1 & LEFT_COL_MASK;

                missile.0 = (missile.0 & !RIGHT_COL_MASK) << 1;
                missile.1 = ((missile.1 & !LEFT_COL_MASK) >> 1) | swapping_sides;

                damaging = (damaging << 1) | about_to_hit_opponent;

                let mut hits = 0;
                for health_tier in (0..DEFENCE_HEALTH).rev() {
                    hits = opponent.buildings[health_tier] & missile.1;
                    missile.1 &= !hits;
                    opponent.buildings[health_tier] &= !hits;
                }
                destroyed |= hits;
            }
        }
        let damage = damaging.count_ones() as u8 * MISSILE_DAMAGE;
        opponent.health = opponent.health.saturating_sub(damage);

        BitwiseGameState::destroy_buildings(opponent, destroyed);
        BitwiseGameState::update_tesla_activity(opponent);
    }

    fn destroy_buildings(buildings: &mut Player, hit_mask: u64) {
        let deconstruct_mask = !hit_mask;
        
        buildings.energy_towers &= deconstruct_mask;
        for tier in &mut buildings.missile_towers {
            *tier &= deconstruct_mask;
        }
        for tier in &mut buildings.buildings {
            *tier &= deconstruct_mask;
        }
        buildings.occupied &= deconstruct_mask;
    }

    fn update_tesla_activity(buildings: &mut Player) {
        let occupied = buildings.occupied;
        buildings.tesla_cooldowns.retain(|t| (t.pos.to_either_bitfield() & occupied) != 0);
    }
    
    
    fn add_energy(player: &mut Player) {
        player.energy += player.energy_generated();
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

impl Player {
    pub fn count_teslas(&self) -> usize {
        self.tesla_cooldowns.len()
            + self.unconstructed.iter().filter(|t| t.building_type == BuildingType::Tesla).count()
    }

    pub fn empty() -> Player {
        Player {
            health: 0,
            energy: 0,
            unconstructed: ArrayVec::new(),
            buildings: [0; DEFENCE_HEALTH],
            occupied: 0,
            energy_towers: 0,
            missile_towers: [0; MISSILE_COOLDOWN_STATES],
            firing_tower: 0,
            missiles: [(0,0); MISSILE_MAX_SINGLE_CELL],
            tesla_cooldowns: ArrayVec::new(),
            iron_curtain_available: false,
            iron_curtain_remaining: 0,
        }
    }

    pub fn energy_generated(&self) -> u16 {
        ENERGY_GENERATED_BASE + self.energy_towers.count_ones() as u16 * ENERGY_GENERATED_TOWER
    }

    pub fn has_max_teslas(&self) -> bool {
        self.count_teslas() >= TESLA_MAX
    }

    pub fn can_build_iron_curtain(&self) -> bool {
        self.iron_curtain_available && self.iron_curtain_remaining == 0 && self.energy >= IRON_CURTAIN_PRICE
    }

    pub fn unoccupied_cell_count(&self) -> usize { self.occupied.count_zeros() as usize }
    pub fn location_of_unoccupied_cell(&self, i: usize) -> Point  {
        let bit = find_bit_index_from_rank(self.occupied, i as u64);
        let point = Point { index: bit };
        debug_assert!(point.to_either_bitfield() & self.occupied == 0);
        point
    }


    fn perform_command(&mut self, command: Command) {
        match command {
            Command::Nothing => {},
            Command::Build(p, b) => {
                let bitfield = p.to_either_bitfield();

                let price = match b {
                    BuildingType::Attack => MISSILE_PRICE,
                    BuildingType::Defence => DEFENCE_PRICE,
                    BuildingType::Energy => ENERGY_PRICE,
                    BuildingType::Tesla => TESLA_PRICE,
                };
                let construction_time = match b {
                    BuildingType::Attack => MISSILE_CONSTRUCTION_TIME,
                    BuildingType::Defence => DEFENCE_CONSTRUCTION_TIME,
                    BuildingType::Energy => ENERGY_CONSTRUCTION_TIME,
                    BuildingType::Tesla => TESLA_CONSTRUCTION_TIME,
                };

                // This is used internally. I should not be making
                // invalid moves!
                debug_assert!(self.buildings[0] & bitfield == 0);
                debug_assert!(p.x() < FULL_MAP_WIDTH && p.y() < MAP_HEIGHT);
                debug_assert!(self.energy >= price);
                debug_assert!(b != BuildingType::Tesla ||
                              self.count_teslas() < TESLA_MAX);

                self.energy -= price;
                self.unconstructed.push(UnconstructedBuilding {
                    pos: p,
                    construction_time_left: construction_time,
                    building_type: b
                });
                self.occupied |= bitfield;
            },
            Command::Deconstruct(p) => {
                let unconstructed_to_remove_index = self.unconstructed.iter().position(|ref b| b.pos == p);
                let deconstruct_mask = !(p.to_either_bitfield() & self.buildings[0]);

                debug_assert!(deconstruct_mask != 0 || unconstructed_to_remove_index.is_some());

                if let Some(i) = unconstructed_to_remove_index {
                    self.unconstructed.swap_remove(i);
                }

                self.energy += DECONSTRUCT_ENERGY;

                for tier in 0..self.buildings.len() {
                    self.buildings[tier] &= deconstruct_mask;
                }
                self.energy_towers &= deconstruct_mask;
                for tier in 0..self.missile_towers.len() {
                    self.missile_towers[tier] &= deconstruct_mask;
                }
                self.tesla_cooldowns.retain(|t| t.pos != p);
                self.occupied &= deconstruct_mask;
            },
            Command::IronCurtain => {
                debug_assert!(self.iron_curtain_available);
                debug_assert!(self.energy >= IRON_CURTAIN_PRICE);

                self.energy -= IRON_CURTAIN_PRICE;
                self.iron_curtain_available = false;
                self.iron_curtain_remaining = IRON_CURTAIN_DURATION;
            }
        }
    }

    fn update_construction(&mut self) {
        let mut buildings_len = self.unconstructed.len();
        for i in (0..buildings_len).rev() {
            if self.unconstructed[i].construction_time_left == 0 {
                let building_type = self.unconstructed[i].building_type;
                let health = if building_type == BuildingType::Defence { DEFENCE_HEALTH } else { 1 };

                let pos = self.unconstructed[i].pos;
                let bitfield = pos.to_either_bitfield();

                for health_tier in 0..health {
                    self.buildings[health_tier] |= bitfield;
                }
                if building_type == BuildingType::Energy {
                    self.energy_towers |= bitfield;
                }
                if building_type == BuildingType::Attack {
                    self.missile_towers[self.firing_tower] |= bitfield;
                }
                if building_type == BuildingType::Tesla {
                    self.tesla_cooldowns.push(TeslaCooldown {
                        pos,
                        cooldown: 0,
                        age: 0
                    });
                }

                buildings_len -= 1;
                self.unconstructed.swap(i, buildings_len);
            } else {
                self.unconstructed[i].construction_time_left -= 1
            }
        }
        self.unconstructed.truncate(buildings_len);
    }

    fn add_missiles(&mut self) {
        let mut missiles = self.missile_towers[self.firing_tower];
        for mut tier in &mut self.missiles {
            let setting = !tier.0 & missiles;
            tier.0 |= setting;
            missiles &= !setting;
        }
        self.firing_tower = (self.firing_tower + 1) % MISSILE_COOLDOWN_STATES;
    }
}
