use super::geometry::Point;
use super::command::BuildingType;
use std::cmp;

#[derive(Debug)]
pub struct GameSettings {
    pub size: Point,
    pub energy_income: u16,
    pub max_building_price: u16,
    pub energy: BuildingSettings,
    pub defence: BuildingSettings,
    pub attack: BuildingSettings,
    pub tesla: BuildingSettings,
}

#[derive(Debug)]
pub struct BuildingSettings {
    pub price: u16,
    pub health: u8,
    pub construction_time: u8,
    pub weapon_damage: u8,
    pub weapon_speed: u8,
    pub weapon_cooldown_period: u8,
    pub energy_generated_per_turn: u16
}

impl GameSettings {
    pub fn new(size: Point, energy_income: u16, energy: BuildingSettings, defence: BuildingSettings, attack: BuildingSettings, tesla: BuildingSettings) -> GameSettings {
        let max_building_price = cmp::max(cmp::max(energy.price, defence.price), attack.price);
        GameSettings {
            size, energy_income, max_building_price,
            energy, defence, attack, tesla
        }
    }
    pub fn building_settings(&self, building: BuildingType) -> &BuildingSettings {
        match building {
            BuildingType::Defence => &self.defence,
            BuildingType::Attack => &self.attack,
            BuildingType::Energy => &self.energy,
            BuildingType::Tesla => &self.tesla,
        }
    }
    
}
