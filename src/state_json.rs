use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;

pub fn read_state_from_file(filename: &str) -> Result<State, Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state = serde_json::from_str(content.as_ref())?;
    Ok(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub game_details: GameDetails,
    pub players: Vec<Player>,
    pub game_map: Vec<Vec<GameCell>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    pub round: u32,
    pub map_width: u32,
    pub map_height: u32,
    pub building_prices: BuildingPrices
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct BuildingPrices {
    pub energy: u32,
    pub defense: u32,
    pub attack: u32
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub player_type: char,
    pub energy: u32,
    pub health: u32,
    pub hits_taken: u32,
    pub score: u32
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameCell {
    pub x: u32,
    pub y: u32,
    pub buildings: Vec<BuildingState>,
    pub missiles: Vec<MissileState>,
    pub cell_owner: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingState {
    pub health: u32,
    pub construction_time_left: i32,
    pub price: u32,
    pub weapon_damage: u32,
    pub weapon_speed: u32,
    pub weapon_cooldown_time_left: u32,
    pub weapon_cooldown_period: u32,
    pub destroy_multiplier: u32,
    pub construction_score: u32,
    pub energy_generated_per_turn: u32,
    pub building_type: String,
    pub x: u32,
    pub y: u32,
    pub player_type: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissileState {
    pub damage: u32,
    pub speed: u32,
    pub x: u32,
    pub y: u32,
    pub player_type: char
}
