pub const FULL_MAP_WIDTH: u8 = 16;
pub const SINGLE_MAP_WIDTH: u8 = FULL_MAP_WIDTH/2;
pub const MAP_HEIGHT: u8 = 8;

pub const MISSILE_COOLDOWN: usize = 3;
pub const MISSILE_COOLDOWN_STATES: usize = MISSILE_COOLDOWN+1;
pub const MISSILE_SPEED: usize = 2;
pub const MISSILE_MAX_SINGLE_CELL: usize = SINGLE_MAP_WIDTH as usize / MISSILE_SPEED;
pub const MISSILE_DAMAGE: u8 = 5;

pub const DEFENCE_HEALTH: usize = 4; // '20' health is 4 hits

pub const TESLA_MAX: usize = 2;
pub const TESLA_COOLDOWN: u8 = 10;
pub const TESLA_FIRING_ENERGY: u16 = 100;
pub const TESLA_DAMAGE: u8 = 20;

pub const ENERGY_GENERATED_BASE: u16 = 5;
pub const ENERGY_GENERATED_TOWER: u16 = 3;

pub const DECONSTRUCT_ENERGY: u16 = 5;
