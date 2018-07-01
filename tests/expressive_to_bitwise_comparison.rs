extern crate zombot;

#[macro_use] extern crate proptest;
extern crate rand;

use zombot::input;
use zombot::engine::command::{Command, BuildingType};
use zombot::engine::geometry::Point;
use zombot::engine::settings::GameSettings;
use zombot::engine::{GameState, GameStatus, Player};

use zombot::engine::expressive_engine;
use zombot::engine::bitwise_engine;

use proptest::prelude::*;

use rand::{Rng, XorShiftRng, SeedableRng};


const STATE_PATH: &str = "tests/state0.json";

proptest! {
    #[test]
    fn follows_the_same_random_game_tree(seed in any::<[u32;4]>()) {
        let mut rng = XorShiftRng::from_seed(seed);
        
        let (settings, mut expressive_state) = input::json::read_expressive_state_from_file(STATE_PATH).expect("Failed to load expressive state");
        let mut bitwise_state = input::json::read_bitwise_state_from_file(STATE_PATH).expect("Failed to load bitwise state");

        let mut expected_status = GameStatus::Continue;
        while expected_status == GameStatus::Continue {
            let player_command = random_player_move(&settings, &expressive_state, &mut rng);
            let opponent_command = random_opponent_move(&settings, &expressive_state, &mut rng);

            expected_status = expressive_state.simulate(&settings, player_command, opponent_command);
            let actual_status = bitwise_state.simulate(&settings, player_command, opponent_command);

            assert_eq!(&expected_status, &actual_status);
            assert_eq!(build_bitwise_from_expressive(&expressive_state), bitwise_state.clone());
        }
    }
}



fn random_player_move<R: Rng, GS: GameState>(settings: &GameSettings, state: &GS, rng: &mut R) -> Command {
    let all_buildings = sensible_buildings(settings, &state.player(), state.player_has_max_teslas());
    random_move(&state.unoccupied_player_cells(), &all_buildings, rng)
}

fn random_opponent_move<R: Rng, GS: GameState>(settings: &GameSettings, state: &GS, rng: &mut R) -> Command {
    let all_buildings = sensible_buildings(settings, &state.opponent(), state.opponent_has_max_teslas());
    random_move(&state.unoccupied_opponent_cells(), &all_buildings, rng)
}

fn random_move<R: Rng>(free_positions: &[Point], all_buildings: &[BuildingType], rng: &mut R) -> Command {
    
    let building_command_count = free_positions.len()*all_buildings.len();
    let nothing_count = 1;

    let number_of_commands = building_command_count + nothing_count;
    
    let choice_index = rng.gen_range(0, number_of_commands);

    if choice_index == number_of_commands - 1 {
        Command::Nothing
    } else {
        Command::Build(
            free_positions[choice_index/all_buildings.len()],
            all_buildings[choice_index%all_buildings.len()]
        )
    }
}

fn sensible_buildings(settings: &GameSettings, player: &Player, has_max_teslas: bool) -> Vec<BuildingType> {
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

fn build_bitwise_from_expressive(expressive: &expressive_engine::ExpressiveGameState) -> bitwise_engine::BitwiseGameState {
    //TODO
    bitwise_engine::BitwiseGameState {
        status: expressive.status,
        player: expressive.player.clone(),
        opponent: expressive.opponent.clone(),
        player_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: Vec::new(),
            buildings: [0,0,0,0],
            energy_towers: 0,
            missile_towers: [0,0,0],
            missiles: [(0,0),(0,0),(0,0),(0,0)],
            tesla_cooldowns: [bitwise_engine::TeslaCooldown {
                active: false,
                pos: Point::new(0,0),
                cooldown: 0
            }, bitwise_engine::TeslaCooldown {
                active: false,
                pos: Point::new(0,0),
                cooldown: 0
            }],
            unoccupied: Vec::new()
        },
        opponent_buildings: bitwise_engine::PlayerBuildings {
            unconstructed: Vec::new(),
            buildings: [0,0,0,0],
            energy_towers: 0,
            missile_towers: [0,0,0],
            missiles: [(0,0),(0,0),(0,0),(0,0)],
            tesla_cooldowns: [bitwise_engine::TeslaCooldown {
                active: false,
                pos: Point::new(0,0),
                cooldown: 0
            }, bitwise_engine::TeslaCooldown {
                active: false,
                pos: Point::new(0,0),
                cooldown: 0
            }],
            unoccupied: Vec::new()
        }
    }
}
