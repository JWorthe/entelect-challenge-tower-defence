use std::fmt;
use super::constants::*;
use super::geometry::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Nothing,
    Build(Point, BuildingType),
    IronCurtain
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Command::Nothing => write!(f, ""),
            Command::Build(p, b) => write!(f, "{},{},{}", p.x(), p.y(), b as u8),
            Command::IronCurtain => write!(f, "0,0,5")
        }
    }
}

impl Command {
    pub fn cant_build_yet(&self, energy: u16) -> bool {
        use self::Command::*;

        match self {
            Nothing => false,
            Build(_, b) => b.cant_build_yet(energy),
            IronCurtain => energy < IRON_CURTAIN_PRICE
        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Defence = 0,
    Attack = 1,
    Energy = 2,
    Tesla = 4,
}

impl BuildingType {
    pub fn all() -> [BuildingType; NUMBER_OF_BUILDING_TYPES] {
        use self::BuildingType::*;
        [Defence, Attack, Energy, Tesla]
    }

    pub fn from_u8(id: u8) -> Option<BuildingType> {
        use std::mem;
        if id <= 4 && id != 3 { Some(unsafe { mem::transmute(id) }) } else { None }
    }

    pub fn cant_build_yet(&self, energy: u16) -> bool {
        use self::BuildingType::*;

        let required = match self {
            Defence => DEFENCE_PRICE,
            Attack => MISSILE_PRICE,
            Energy => ENERGY_PRICE,
            Tesla => TESLA_PRICE
        };
        energy < required
    }
}
