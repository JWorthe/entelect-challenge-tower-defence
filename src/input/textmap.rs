use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

use ::engine::*;
use ::engine::settings::*;
use ::engine::geometry::*;


pub fn read_state_from_file(filename: &str) -> Result<(GameSettings, GameState), Box<Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    //TODO actually read the right file and parse it?

    let engine_settings = GameSettings {
        size: Point::new(8,4),
        energy_income: 5,
        energy: BuildingSettings {
            price: 20,
            health: 5,
            construction_time: 2-2,
            weapon_damage: 0,
            weapon_speed: 0,
            weapon_cooldown_period: 0,
            energy_generated_per_turn: 3
        },
        defence: BuildingSettings {
            price: 30,
            health: 20,
            construction_time: 4-2,
            weapon_damage: 0,
            weapon_speed: 0,
            weapon_cooldown_period: 0,
            energy_generated_per_turn: 0
        },
        attack: BuildingSettings {
            price: 30,
            health: 5,
            construction_time: 2-2,
            weapon_damage: 5,
            weapon_speed: 2,
            weapon_cooldown_period: 3,
            energy_generated_per_turn: 0
        }
    };
    let engine_state = GameState::new(
        Player {
            energy: 20,
            health: 100,
            energy_generated: 5
        },
        Player {
            energy: 20,
            health: 100,
            energy_generated: 5
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &engine_settings
    );
    
    Ok((engine_settings, engine_state))
}
