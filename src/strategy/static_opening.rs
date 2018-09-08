use engine::geometry::*;
use engine::command::*;
use engine::bitwise_engine::*;

pub const STATIC_OPENING_LENGTH: u16 = 12;

pub fn choose_move(state: &BitwiseGameState) -> Command {
    match state.round {
        0 => Command::Build(Point::new(0,0), BuildingType::Energy),
        3 => Command::Build(Point::new(0,1), BuildingType::Energy),
        5 => Command::Build(Point::new(0,2), BuildingType::Energy),
        7 => Command::Build(Point::new(0,3), BuildingType::Energy),
        9 => Command::Build(Point::new(0,4), BuildingType::Energy),
        10 => Command::Build(Point::new(0,5), BuildingType::Energy),
        11 => Command::Build(Point::new(0,6), BuildingType::Energy),
        12 => Command::Build(Point::new(0,7), BuildingType::Energy),
        13 => Command::Build(Point::new(1,0), BuildingType::Energy),
        14 => Command::Build(Point::new(1,7), BuildingType::Energy),
        _ => Command::Nothing
    }
}
