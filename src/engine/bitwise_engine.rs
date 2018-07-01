use engine::command::{Command, BuildingType};
use engine::geometry::Point;
use engine::settings::{GameSettings};
use engine::{GameStatus, Player, GameState};

const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 8;

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
    fn simulate(&mut self, _settings: &GameSettings, _player_command: Command, _opponent_command: Command) -> GameStatus {
        //TODO
        self.status
    }


    fn player(&self) -> &Player { &self.player }
    fn opponent(&self) -> &Player { &self.opponent }
    fn player_has_max_teslas(&self) -> bool { self.player_buildings.count_teslas() >= MAX_TESLAS }
    fn opponent_has_max_teslas(&self) -> bool { self.opponent_buildings.count_teslas() >= MAX_TESLAS }
    fn unoccupied_player_cells(&self) -> &[Point] { &EMPTY }
    fn unoccupied_opponent_cells(&self) -> &[Point] { &EMPTY }
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
}

impl PlayerBuildings {
    pub fn count_teslas(&self) -> usize {
        self.tesla_cooldowns.iter().filter(|t| t.active).count()
    }

    pub fn empty() -> PlayerBuildings {
        PlayerBuildings {
            unconstructed: Vec::with_capacity(4),
            buildings: [0; 4],
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
