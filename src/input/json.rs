use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;

use engine;
use engine::command;
use engine::expressive_engine;
use engine::bitwise_engine;
use engine::constants::*;

pub fn read_expressive_state_from_file(filename: &str) -> Result<(engine::settings::GameSettings, expressive_engine::ExpressiveGameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;

    let engine_settings = state.to_engine_settings();
    let engine_state = state.to_expressive_engine(&engine_settings);
    Ok((engine_settings, engine_state))
}

pub fn read_bitwise_state_from_file(filename: &str) -> Result<(engine::settings::GameSettings, bitwise_engine::BitwiseGameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;

    let engine_settings = state.to_engine_settings();
    let engine_state = state.to_bitwise_engine();
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
    x: u8,
    y: u8,
    buildings: Vec<BuildingState>,
    missiles: Vec<MissileState>,
    //cell_owner: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildingState {
    health: u8,
    construction_time_left: i16,
    //price: u16,
    weapon_damage: u8,
    weapon_speed: u8,
    weapon_cooldown_time_left: u8,
    weapon_cooldown_period: u8,
    //destroy_multiplier: u32,
    //construction_score: u32,
    energy_generated_per_turn: u16,
    building_type: String,
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
    
    fn to_expressive_engine(&self, settings: &engine::settings::GameSettings) -> expressive_engine::ExpressiveGameState {
        let player_buildings = self.buildings_to_expressive_engine('A');
        let opponent_buildings = self.buildings_to_expressive_engine('B');
        expressive_engine::ExpressiveGameState::new(
            self.player().to_engine(settings, &player_buildings),
            self.opponent().to_engine(settings, &opponent_buildings),
            self.unconstructed_buildings_to_expressive_engine('A'),
            player_buildings,
            self.unconstructed_buildings_to_expressive_engine('B'),
            opponent_buildings,
            self.missiles_to_expressive_engine('A'),
            self.missiles_to_expressive_engine('B'),
            settings
        )
    }

    fn to_bitwise_engine(&self) -> bitwise_engine::BitwiseGameState {
        let mut player = self.player().to_bitwise_engine();
        let mut opponent = self.opponent().to_bitwise_engine();
        let mut player_buildings = bitwise_engine::PlayerBuildings::empty();
        let mut opponent_buildings = bitwise_engine::PlayerBuildings::empty();
        for row in &self.game_map {
            for cell in row {
                let point = engine::geometry::Point::new(cell.x, cell.y);
                for building in &cell.buildings {
                    let building_type = building.convert_building_type();
                    
                    let (mut engine_player, mut bitwise_buildings, bitfield) = if building.player_type == 'A' {
                        (&mut player, &mut player_buildings, point.to_left_bitfield())
                    } else {
                        (&mut opponent, &mut opponent_buildings, point.to_right_bitfield())
                    };

                    bitwise_buildings.occupied |= bitfield;
                    if building.construction_time_left >= 0 {
                        bitwise_buildings.unconstructed.push(building.to_bitwise_engine_unconstructed());
                    } else {
                        for health_tier in 0..DEFENCE_HEALTH {
                            if building.health > health_tier as u8 * MISSILE_DAMAGE {
                                bitwise_buildings.buildings[health_tier] |= bitfield;
                            }
                        }
                        if building_type == command::BuildingType::Energy {
                            bitwise_buildings.energy_towers |= bitfield;
                            engine_player.energy_generated += building.energy_generated_per_turn;
                        }
                        else if building_type == command::BuildingType::Attack {
                            for cooldown_tier in 0..MISSILE_COOLDOWN + 1 {
                                if building.weapon_cooldown_time_left == cooldown_tier as u8 {
                                    bitwise_buildings.missile_towers[cooldown_tier] |= bitfield;
                                }
                            }
                        }
                        else if building_type == command::BuildingType::Tesla {
                            let ref mut tesla_cooldown = if bitwise_buildings.tesla_cooldowns[0].active {
                                &mut bitwise_buildings.tesla_cooldowns[1]
                            } else {
                                &mut bitwise_buildings.tesla_cooldowns[0]
                            };
                            tesla_cooldown.active = true;
                            tesla_cooldown.pos = point;
                            tesla_cooldown.cooldown = building.weapon_cooldown_time_left;
                            tesla_cooldown.age = building.construction_time_left.abs() as u16;
                        }
                    }
                }
                for missile in &cell.missiles {
                    let bitfields = point.to_bitfield();
                    let (mut bitwise_buildings, mut left, mut right) = if missile.player_type == 'A' {
                        (&mut player_buildings, bitfields.0, bitfields.1)
                    } else {
                        (&mut opponent_buildings, bitfields.1, bitfields.0)
                    };

                    for mut tier in bitwise_buildings.missiles.iter_mut() {
                        let setting = (!tier.0 & left, !tier.1 & right);
                        tier.0 |= setting.0;
                        tier.1 |= setting.1;
                        left &= !setting.0;
                        right &= !setting.1;
                    }
                }
            }
        }
            
        bitwise_engine::BitwiseGameState::new(
            player, opponent,
            player_buildings, opponent_buildings
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

    fn unconstructed_buildings_to_expressive_engine(&self, player_type: char) -> Vec<expressive_engine::UnconstructedBuilding> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left >= 0)
                                .map(|b| b.to_expressive_engine_unconstructed())
                      )
            )
            .collect()
    }
   
    fn buildings_to_expressive_engine(&self, player_type: char) -> Vec<expressive_engine::Building> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.buildings.iter()
                                .filter(|b| b.player_type == player_type && b.construction_time_left < 0)
                                .map(|b| b.to_expressive_engine())
                      )
            )
            .collect()
    }

    fn missiles_to_expressive_engine(&self, player_type: char) -> Vec<expressive_engine::Missile> {
        self.game_map.iter()
            .flat_map(|row| row.iter()
                      .flat_map(|cell| cell.missiles.iter()
                                .filter(|b| b.player_type == player_type)
                                .map(|b| b.to_expressive_engine())
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
    fn to_bitwise_engine(&self) -> engine::Player {
        engine::Player {
            energy: self.energy,
            health: self.health,
            energy_generated: ENERGY_GENERATED_BASE
        }
    }
}

impl BuildingState {
    fn to_expressive_engine(&self) -> expressive_engine::Building {
        expressive_engine::Building {
            pos: engine::geometry::Point::new(self.x, self.y),
            health: self.health,
            weapon_damage: self.weapon_damage,
            weapon_speed: self.weapon_speed,
            weapon_cooldown_time_left: self.weapon_cooldown_time_left,
            weapon_cooldown_period: self.weapon_cooldown_period,
            energy_generated_per_turn: self.energy_generated_per_turn,
            age: self.construction_time_left.abs() as u16
        }
    }

    fn to_expressive_engine_unconstructed(&self) -> expressive_engine::UnconstructedBuilding {
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

    fn to_bitwise_engine_unconstructed(&self) -> bitwise_engine::UnconstructedBuilding {
        bitwise_engine::UnconstructedBuilding {
            pos: engine::geometry::Point::new(self.x, self.y),
            construction_time_left: self.construction_time_left as u8, // > 0 check already happened
            building_type: self.convert_building_type()
        }
    }

    fn convert_building_type(&self) -> command::BuildingType {
        match self.building_type.as_ref() {
            "ATTACK" => command::BuildingType::Attack,
            "ENERGY" => command::BuildingType::Energy,
            "TESLA" => command::BuildingType::Tesla,
            _ => command::BuildingType::Defence,
        }
    }
}

impl MissileState {
    fn to_expressive_engine(&self) -> expressive_engine::Missile {
        expressive_engine::Missile {
            pos: engine::geometry::Point::new(self.x, self.y),
            damage: self.damage,
            speed: self.speed,
        }
    }
}
