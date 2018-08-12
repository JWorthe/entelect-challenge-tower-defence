extern crate zombot;

use zombot::input::json;
use zombot::engine::command::{Command, BuildingType};
use zombot::engine::geometry::Point;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[test]
fn it_successfully_simulates_replay() {
    test_from_replay(&Path::new("tests/v300_normal_towers"));
}

fn test_from_replay(replay_folder: &Path) {
    let length = replay_folder.read_dir().unwrap().count()-1;
        
    let  mut state = json::read_bitwise_state_from_file(&format!("{}/Round 000/state.json", replay_folder.display())).unwrap();
    
    for i in 0..length {
        let player = read_player_command(&format!("{}/Round {:03}/PlayerCommand.txt", replay_folder.display(), i));
        let opponent = read_opponent_command(&format!("{}/Round {:03}/OpponentCommand.txt", replay_folder.display(), i));
        let mut expected_state = json::read_bitwise_state_from_file(&format!("{}/Round {:03}/state.json", replay_folder.display(), i+1)).unwrap();
        
        state.simulate(player, opponent);
        state.sort();
        expected_state.sort();

        println!("State {}: {:?}", i+1, state);
        assert_eq!(state, expected_state, "\nFailed on state {}\n", i+1);
    }
}

fn read_player_command(filename: &str) -> Command {
    let mut file = File::open(filename).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    if content.trim() == "No Command" {
        Command::Nothing
    }
    else {
        let mut components = content.split(',');
        let point = Point::new(components.next().unwrap().trim().parse().unwrap(),
                               components.next().unwrap().trim().parse().unwrap());
        let action_type = components.next().unwrap().trim().parse().unwrap();
        if action_type == 3 {
            Command::Deconstruct(point)
        } else {
            Command::Build(point, BuildingType::from_u8(action_type).unwrap())
        }
    }
}

fn read_opponent_command(filename: &str) -> Command {
    read_player_command(filename)
}
