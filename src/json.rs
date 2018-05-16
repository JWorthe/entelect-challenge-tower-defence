use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;

use ::engine;


pub fn read_state_from_file(filename: &str) -> Result<(engine::settings::GameSettings, engine::GameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;

    let engine_settings = state.to_engine_settings();
    let engine_state = state.to_engine(&engine_settings);
    Ok((engine_settings, engine_state))
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
    //round: u32,
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
    attack: BuildingBlueprint
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
        engine::settings::GameSettings {
            size: engine::geometry::Point::new(self.game_details.map_width, self.game_details.map_height),
            energy_income: self.game_details.round_income_energy,
            energy: self.game_details.buildings_stats.energy.to_engine(),
            defence: self.game_details.buildings_stats.defense.to_engine(),
            attack: self.game_details.buildings_stats.attack.to_engine(),
        }
    }
    
    fn to_engine(&self, settings: &engine::settings::GameSettings) -> engine::GameState {
        engine::GameState::new(
            self.player().to_engine(),
            self.opponent().to_engine(),
            self.unconstructed_buildings_to_engine('A'),
            self.buildings_to_engine('A'),
            self.unconstructed_buildings_to_engine('B'),
            self.buildings_to_engine('B'),
            self.missiles_to_engine('A'),
            self.missiles_to_engine('B'),
            settings
        )
    }

    fn player(&self) -> &Player {
        self.players.iter()
            .filter(|p| p.player_type == 'A')
            .next()
            .expect("Player character did not appear in state.json")
    }

    fn opponent(&self) -> &Player {
        self.players.iter()
            .filter(|p| p.player_type == 'B')
            .next()
            .expect("Opponent character did not appear in state.json")
    }

    fn unconstructed_buildings_to_engine(&self, player_type: char) -> Vec<engine::UnconstructedBuilding> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left > 0)
                                .map(|b| b.to_engine_unconstructed())
                      )
            )
            .collect()
    }
    
    fn buildings_to_engine(&self, player_type: char) -> Vec<engine::Building> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left <= 0)
                                .map(|b| b.to_engine())
                      )
            )
            .collect()
    }

    fn missiles_to_engine(&self, player_type: char) -> Vec<engine::Missile> {
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
            construction_time: self.construction_time-2,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
        }
    }
}

impl Player {
    fn to_engine(&self) -> engine::Player {
        engine::Player {
            energy: self.energy,
            health: self.health,
        }
    }
}

impl BuildingState {
    fn to_engine(&self) -> engine::Building {
        engine::Building {
            pos: engine::geometry::Point::new(self.x, self.y),
            health: self.health,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_time_left: self.weapon_cooldown_time_left,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
        }
    }

    fn to_engine_unconstructed(&self) -> engine::UnconstructedBuilding {
        engine::UnconstructedBuilding {
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
    fn to_engine(&self) -> engine::Missile {
        engine::Missile {
            pos: engine::geometry::Point::new(self.x, self.y),
            damage: self.damage,
            speed: self.speed,
        }
    }
}
