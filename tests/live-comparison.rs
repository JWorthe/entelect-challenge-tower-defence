extern crate zombot;

use zombot::input::json;
use zombot::engine::command::{Command, BuildingType};
use zombot::engine::geometry::Point;
use zombot::engine::settings::GameSettings;

use std::fs::File;
use std::io::prelude::*;

#[test]
fn it_successfully_simulates_replay() {
    test_from_replay("tests/after_113", 32);
}

fn test_from_replay(replay_folder: &str, length: usize) {
    let (settings, mut state) = json::read_state_from_file(&format!("{}/Round 000/state.json", replay_folder)).unwrap();
    
    for i in 0..length {
        let player = read_player_command(&format!("{}/Round {:03}/PlayerCommand.txt", replay_folder, i));
        let opponent = read_opponent_command(&format!("{}/Round {:03}/OpponentCommand.txt", replay_folder, i), &settings);
        let (_, mut expected_state) = json::read_state_from_file(&format!("{}/Round {:03}/state.json", replay_folder, i+1)).unwrap();
        
        state.simulate_mut(&settings, player, opponent);
        state.sort();
        expected_state.sort();
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
        Command::Build(
            Point::new(components.next().unwrap().trim().parse().unwrap(),
                       components.next().unwrap().trim().parse().unwrap()
            ),
            BuildingType::from_u8(components.next().unwrap().trim().parse().unwrap()).unwrap()
        )
    }
}

fn read_opponent_command(filename: &str, settings: &GameSettings) -> Command {
    match read_player_command(filename) {
        Command::Nothing => Command::Nothing,
        Command::Build(p, b) => Command::Build(Point::new(
            settings.size.x - p.x - 1,
            p.y
        ), b)
    }
    
}
