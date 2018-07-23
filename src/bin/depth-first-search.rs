extern crate zombot;
extern crate time;
use time::PreciseTime;

use zombot::*;
use zombot::engine::*;
use zombot::engine::settings::*;
use zombot::engine::command::*;

const STATE_PATH: &str = "tests/state0.json";

use std::process;

fn main() {
    println!("Performing an exhaustive depth-first walk of the game states");
    let start_time = PreciseTime::now();
    let (settings, state) = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };

    walk_states(&settings, &state, 0);

    println!("Total running time: {}", start_time.to(PreciseTime::now()));
}

fn walk_states<GS: GameState>(settings: &GameSettings, state: &GS, depth: u32) {
    if depth >= 200 {
        return;
    }

    let player_buildings = valid_buildings(settings, &state.player(), state.player_has_max_teslas());
    let opponent_buildings = valid_buildings(settings, &state.opponent(), state.opponent_has_max_teslas());

    for &player_building in &player_buildings {
        for player_i in 0..state.unoccupied_player_cell_count() {
            for &opponent_building in &opponent_buildings {
                for opponent_i in 0..state.unoccupied_opponent_cell_count() {
                    let player_point = state.location_of_unoccupied_player_cell(player_i);
                    let player_move = Command::Build(player_point, player_building);
                    let opponent_point = state.location_of_unoccupied_player_cell(opponent_i);
                    let opponent_move = Command::Build(opponent_point, opponent_building);

                    let mut after_move = state.clone();
                    let status = after_move.simulate(settings, player_move, opponent_move);
                    if status == GameStatus::Continue {
                        walk_states(settings, &after_move, depth+1);
                    }
                }
            }
        }
    }
    for player_building in player_buildings {
        for player_i in 0..state.unoccupied_player_cell_count() {
            let player_point = state.location_of_unoccupied_player_cell(player_i);
            let player_move = Command::Build(player_point, player_building);
            let opponent_move = Command::Nothing;

            let mut after_move = state.clone();
            let status = after_move.simulate(settings, player_move, opponent_move);
            if status == GameStatus::Continue {
                walk_states(settings, &after_move, depth+1);
            }
        }
    }
    for opponent_building in opponent_buildings {
        for opponent_i in 0..state.unoccupied_opponent_cell_count() {
            let player_move = Command::Nothing;
            let opponent_point = state.location_of_unoccupied_player_cell(opponent_i);
            let opponent_move = Command::Build(opponent_point, opponent_building);

            let mut after_move = state.clone();
            let status = after_move.simulate(settings, player_move, opponent_move);
            if status == GameStatus::Continue {
                walk_states(settings, &after_move, depth+1);
            }
        }
    }
    let player_move = Command::Nothing;
    let opponent_move = Command::Nothing;
    let mut after_move = state.clone();
    let status = after_move.simulate(settings, player_move, opponent_move);
    if status == GameStatus::Continue {
        walk_states(settings, &after_move, depth+1);
    }
    if depth < 10 {
        print!(".");
    }
}

fn valid_buildings(settings: &GameSettings, player: &Player, has_max_teslas: bool) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);
    for b in BuildingType::all().iter() {
        let building_setting = settings.building_settings(*b);
        let affordable = building_setting.price <= player.energy;
        let is_tesla = *b == BuildingType::Tesla;
        if affordable && (!is_tesla || !has_max_teslas) {
            result.push(*b);
        }
    }
    result
}
