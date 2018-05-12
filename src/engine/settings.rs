use super::geometry::Point;
use super::command::BuildingType;

#[derive(Debug)]
pub struct GameSettings {
    pub size: Point,
    pub energy_income: u16,
    pub energy: BuildingSettings,
    pub defence: BuildingSettings,
    pub attack: BuildingSettings
}

#[derive(Debug)]
pub struct BuildingSettings {
    pub price: u16,
    pub health: u16,
    pub construction_time: u8,
    pub weapon_damage: u16,
    pub weapon_speed: u8,
    pub weapon_cooldown_period: u8,
    pub energy_generated_per_turn: u16
}

impl GameSettings {
    pub fn building_settings(&self, building: BuildingType) -> &BuildingSettings {
        match building {
            BuildingType::Defence => &self.defence,
            BuildingType::Attack => &self.attack,
            BuildingType::Energy => &self.energy
        }
    }
    
}
