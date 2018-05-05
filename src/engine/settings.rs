use super::geometry::Point;

#[derive(Debug)]
pub struct GameSettings {
    pub size: Point,
    pub energy_income: u16,
    pub energy_price: u16,
    pub defence_price: u16,
    pub attack_price: u16
}
