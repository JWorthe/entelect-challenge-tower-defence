use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;

use engine;
use engine::expressive_engine;
use engine::bitwise_engine;

pub fn read_expressive_state_from_file(filename: &str) -> Result<(engine::settings::GameSettings, expressive_engine::ExpressiveGameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;

    let engine_settings = state.to_engine_settings();
    let engine_state = state.to_engine(&engine_settings);
    Ok((engine_settings, engine_state))
}

pub fn read_bitwise_state_from_file(filename: &str) -> Result<bitwise_engine::BitwiseGameState, Box<Error>> {
    //TODO
    Ok(bitwise_engine::BitwiseGameState {
        status: engine::GameStatus::Continue,
        player: engine::Player {
            energy: 0, health: 0, energy_generated: 0
        },
        opponent: engine::Player {
            energy: 0, health: 0, energy_generated: 0
        },
        player_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: Vec::new(),
            buildings: [0,0,0,0],
            energy_towers: 0,
            missile_towers: [0,0,0],
            missiles: [(0,0),(0,0),(0,0),(0,0)],
            tesla_cooldowns: [bitwise_engine::TeslaCooldown {
                active: false,
                pos: engine::geometry::Point::new(0,0),
                cooldown: 0
            }, bitwise_engine::TeslaCooldown {
                active: false,
                pos: engine::geometry::Point::new(0,0),
                cooldown: 0
            }],
            unoccupied: Vec::new()
        },
        opponent_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: Vec::new(),
            buildings: [0,0,0,0],
            energy_towers: 0,
            missile_towers: [0,0,0],
            missiles: [(0,0),(0,0),(0,0),(0,0)],
            tesla_cooldowns: [bitwise_engine::TeslaCooldown {
                active: false,
                pos: engine::geometry::Point::new(0,0),
                cooldown: 0
            }, bitwise_engine::TeslaCooldown {
                active: false,
                pos: engine::geometry::Point::new(0,0),
                cooldown: 0
            }],
            unoccupied: Vec::new()
        }
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    game_details: GameDetails,
    players: Vec<Player>,
    game_map: Vec<Vec<GameCell>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameDetails {
    //round: u16,
    //max_rounds: u16,
    map_width: u8,
    map_height: u8,
    round_income_energy: u16,
    buildings_stats: BuildingStats
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct BuildingStats {
    energy: BuildingBlueprint,
    defense: BuildingBlueprint,
    attack: BuildingBlueprint,
    tesla: BuildingBlueprint,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildingBlueprint {
    price: u16,
    health: u8,
    construction_time: u8,
    weapon_damage: u8,
    weapon_speed: u8,
    weapon_cooldown_period: u8,
    energy_generated_per_turn: u16,
//    destroy_multiplier: u16,
//    construction_score: u16
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
    player_type: char,
    energy: u16,
    health: u8,
    //hits_taken: u32,
    //score: u32
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameCell {
    //x: u8,
    //y: u8,
    buildings: Vec<BuildingState>,
    missiles: Vec<MissileState>,
    //cell_owner: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildingState {
    health: u8,
    construction_time_left: i8,
    //price: u16,
    weapon_damage: u8,
    weapon_speed: u8,
    weapon_cooldown_time_left: u8,
    weapon_cooldown_period: u8,
    //destroy_multiplier: u32,
    //construction_score: u32,
    energy_generated_per_turn: u16,
    //building_type: String,
    x: u8,
    y: u8,
    player_type: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MissileState {
    damage: u8,
    speed: u8,
    x: u8,
    y: u8,
    player_type: char
}


impl State {
    fn to_engine_settings(&self) -> engine::settings::GameSettings {
        engine::settings::GameSettings::new(
            engine::geometry::Point::new(self.game_details.map_width, self.game_details.map_height),
            self.game_details.round_income_energy,
            self.game_details.buildings_stats.energy.to_engine(),
            self.game_details.buildings_stats.defense.to_engine(),
            self.game_details.buildings_stats.attack.to_engine(),
            self.game_details.buildings_stats.tesla.to_engine(),
        )
    }
    
    fn to_engine(&self, settings: &engine::settings::GameSettings) -> expressive_engine::ExpressiveGameState {
        let player_buildings = self.buildings_to_engine('A');
        let opponent_buildings = self.buildings_to_engine('B');
        expressive_engine::ExpressiveGameState::new(
            self.player().to_engine(settings, &player_buildings),
            self.opponent().to_engine(settings, &opponent_buildings),
            self.unconstructed_buildings_to_engine('A'),
            player_buildings,
            self.unconstructed_buildings_to_engine('B'),
            opponent_buildings,
            self.missiles_to_engine('A'),
            self.missiles_to_engine('B'),
            settings
        )
    }

    fn player(&self) -> &Player {
        self.players.iter()
            .find(|p| p.player_type == 'A')
            .expect("Player character did not appear in state.json")
    }

    fn opponent(&self) -> &Player {
        self.players.iter()
            .find(|p| p.player_type == 'B')
            .expect("Opponent character did not appear in state.json")
    }

    fn unconstructed_buildings_to_engine(&self, player_type: char) -> Vec<expressive_engine::UnconstructedBuilding> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left >= 0)
                                .map(|b| b.to_engine_unconstructed())
                      )
            )
            .collect()
    }
    
    fn buildings_to_engine(&self, player_type: char) -> Vec<expressive_engine::Building> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left < 0)
                                .map(|b| b.to_engine())
                      )
            )
            .collect()
    }

    fn missiles_to_engine(&self, player_type: char) -> Vec<expressive_engine::Missile> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.missiles.iter()
                                .filter(|b| b.player_type == player_type)
                                .map(|b| b.to_engine())
                      )
            )
            .collect()
    }
}

impl BuildingBlueprint {
    fn to_engine(&self) -> engine::settings::BuildingSettings {
        engine::settings::BuildingSettings {
            price: self.price,
            health: self.health,
            construction_time: self.construction_time-1,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
        }
    }
}

impl Player {
    fn to_engine(&self, settings: &engine::settings::GameSettings, buildings: &[expressive_engine::Building]) -> engine::Player {
        engine::Player {
            energy: self.energy,
            health: self.health,
            energy_generated: settings.energy_income + buildings.iter().map(|b| b.energy_generated_per_turn).sum::<u16>()
        }
    }
}

impl BuildingState {
    fn to_engine(&self) -> expressive_engine::Building {
        expressive_engine::Building {
            pos: engine::geometry::Point::new(self.x, self.y),
            health: self.health,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_time_left: self.weapon_cooldown_time_left,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
        }
    }

    fn to_engine_unconstructed(&self) -> expressive_engine::UnconstructedBuilding {
        expressive_engine::UnconstructedBuilding {
            pos: engine::geometry::Point::new(self.x, self.y),
            health: self.health,
            construction_time_left: self.construction_time_left as u8, // > 0 check already happened
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
        }
    }
}

impl MissileState {
    fn to_engine(&self) -> expressive_engine::Missile {
        expressive_engine::Missile {
            pos: engine::geometry::Point::new(self.x, self.y),
            damage: self.damage,
            speed: self.speed,
        }
    }
}
