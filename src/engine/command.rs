use std::fmt;
use super::geometry::Point;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Nothing,
    Build(Point, BuildingType),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Command::Nothing => write!(f, ""),
            &Command::Build(p, b) => write!(f, "{},{},{}", p.x, p.y, b as u8),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BuildingType {
    Defence = 0,
    Attack = 1,
    Energy = 2,
}
