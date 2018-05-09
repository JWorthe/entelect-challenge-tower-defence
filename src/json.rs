use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;
use std::cmp;

use ::engine;


pub fn read_state_from_file(filename: &str) -> Result<(engine::settings::GameSettings, engine::GameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;
    Ok((state.to_engine_settings(), state.to_engine()))
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
    building_prices: BuildingPrices
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct BuildingPrices {
    energy: u16,
    defense: u16,
    attack: u16
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
    player_type: char,
    energy: u16,
    health: u16,
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
    health: u16,
    construction_time_left: i8,
    //price: u16,
    weapon_damage: u16,
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
    damage: u16,
    speed: u8,
    x: u8,
    y: u8,
    player_type: char
}


impl State {
    fn to_engine_settings(&self) -> engine::settings::GameSettings {
        engine::settings::GameSettings {
            size: engine::geometry::Point::new(self.game_details.map_width, self.game_details.map_height),
            energy_income: 5,
            energy_price: self.game_details.building_prices.energy,
            defence_price: self.game_details.building_prices.defense,
            attack_price: self.game_details.building_prices.attack,
        }
    }
    
    fn to_engine(&self) -> engine::GameState {
        engine::GameState {
            status: engine::GameStatus::Continue,
            player: self.player().to_engine(),
            opponent: self.opponent().to_engine(),
            player_buildings: self.buildings_to_engine('A'),
            opponent_buildings: self.buildings_to_engine('B'),
            player_missiles: self.missiles_to_engine('A'),
            opponent_missiles: self.missiles_to_engine('B'),
        }
    }

    fn player(&self) -> &Player {
        self.players.iter()
            .filter(|p| p.player_type == 'A')
            .next()
            .expect("Player character did not appear in state.json")
    }

    fn opponent(&self) -> &Player {
        self.players.iter()
            .filter(|p| p.player_type != 'B')
            .next()
            .expect("Opponent character did not appear in state.json")
    }

    fn buildings_to_engine(&self, player_type: char) -> Vec<engine::Building> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type)
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
            construction_time_left: cmp::max(0, self.construction_time_left) as u8,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_time_left: self.weapon_cooldown_time_left,
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
