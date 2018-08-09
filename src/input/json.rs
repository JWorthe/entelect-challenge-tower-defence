use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::error::Error;

use engine;
use engine::command;
use engine::bitwise_engine;
use engine::constants::*;

pub fn read_bitwise_state_from_file(filename: &str) -> Result<bitwise_engine::BitwiseGameState, Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let state: State = serde_json::from_str(content.as_ref())?;

    let engine_state = state.to_bitwise_engine();
    Ok(engine_state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    players: Vec<Player>,
    game_map: Vec<Vec<GameCell>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
    player_type: char,
    energy: u16,
    health: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameCell {
    x: u8,
    y: u8,
    buildings: Vec<BuildingState>,
    missiles: Vec<MissileState>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildingState {
    health: u8,
    construction_time_left: i16,
    weapon_cooldown_time_left: u8,
    building_type: String,
    x: u8,
    y: u8,
    player_type: char
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MissileState {
    player_type: char
}


impl State {
    fn to_bitwise_engine(&self) -> bitwise_engine::BitwiseGameState {
        let player = self.player().to_bitwise_engine();
        let opponent = self.opponent().to_bitwise_engine();
        let mut player_buildings = bitwise_engine::PlayerBuildings::empty();
        let mut opponent_buildings = bitwise_engine::PlayerBuildings::empty();
        for row in &self.game_map {
            for cell in row {
                let point = engine::geometry::Point::new(cell.x, cell.y);
                for building in &cell.buildings {
                    let building_type = building.convert_building_type();
                    
                    let (mut bitwise_buildings, bitfield) = if building.player_type == 'A' {
                        (&mut player_buildings, point.to_left_bitfield())
                    } else {
                        (&mut opponent_buildings, point.to_right_bitfield())
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
}

impl Player {
    fn to_bitwise_engine(&self) -> engine::bitwise_engine::Player {
        engine::bitwise_engine::Player {
            energy: self.energy,
            health: self.health
        }
    }
}

impl BuildingState {
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
