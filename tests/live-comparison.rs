extern crate zombot;

use zombot::json;
use zombot::engine::command::{Command, BuildingType};
use zombot::engine::geometry::Point;

#[test]
fn it_successfully_simulates_moves() {
    let (settings, mut state) = json::read_state_from_file("tests/state0.json").expect("Failed to read state0.json");

    let all_commands = [
        (Command::Build(Point::new(3,2),BuildingType::Energy), Command::Nothing),
        (Command::Nothing, Command::Nothing),
        (Command::Nothing, Command::Build(Point::new(4,3),BuildingType::Energy)),
        (Command::Build(Point::new(3,1),BuildingType::Energy), Command::Nothing),
        (Command::Nothing, Command::Nothing),
        (Command::Build(Point::new(3,0),BuildingType::Energy),Command::Build(Point::new(6,0),BuildingType::Energy)),
        (Command::Nothing,Command::Nothing),
        (Command::Build(Point::new(3,3),BuildingType::Energy),Command::Build(Point::new(7,1),BuildingType::Attack)),
        (Command::Nothing,Command::Nothing),
        (Command::Build(Point::new(2,3),BuildingType::Attack),Command::Nothing),
        
        (Command::Build(Point::new(2,1),BuildingType::Energy),Command::Build(Point::new(5,3),BuildingType::Defence)),
        (Command::Nothing,Command::Nothing),
        (Command::Build(Point::new(1,0),BuildingType::Attack),Command::Nothing),
        (Command::Nothing,Command::Build(Point::new(5,0),BuildingType::Defence)),
        (Command::Build(Point::new(0,2),BuildingType::Attack),Command::Nothing),
        (Command::Build(Point::new(3,1),BuildingType::Energy),Command::Nothing),
        (Command::Nothing,Command::Nothing),
        (Command::Build(Point::new(0,1),BuildingType::Attack),Command::Build(Point::new(7,2),BuildingType::Defence)),
        (Command::Build(Point::new(2,1),BuildingType::Energy),Command::Nothing),
        (Command::Nothing,Command::Nothing),
        
        (Command::Build(Point::new(0,0),BuildingType::Attack),Command::Nothing),
        (Command::Build(Point::new(0,3),BuildingType::Attack),Command::Build(Point::new(4,1),BuildingType::Defence)),
        (Command::Nothing,Command::Nothing),
        (Command::Build(Point::new(1,3),BuildingType::Attack),Command::Nothing),
        (Command::Build(Point::new(3,1),BuildingType::Energy),Command::Nothing),
        (Command::Nothing,Command::Build(Point::new(6,1),BuildingType::Defence)),
        (Command::Build(Point::new(2,2),BuildingType::Energy),Command::Nothing),
        (Command::Build(Point::new(1,2),BuildingType::Energy),Command::Nothing),
        (Command::Build(Point::new(3,1),BuildingType::Energy),Command::Build(Point::new(7,0),BuildingType::Defence)),
        (Command::Build(Point::new(2,1),BuildingType::Energy),Command::Nothing)
    ];

    for (i, &(player, opponent)) in all_commands.iter().enumerate() {
        let file = format!("tests/state{}.json", i+1);
        state.simulate_mut(&settings, player, opponent);
        let (_, mut actual_state) = json::read_state_from_file(&file).unwrap();
        state.sort();
        actual_state.sort();
        assert_eq!(state, actual_state, "\nFailed on state {}\n", i+1);
    }
}
