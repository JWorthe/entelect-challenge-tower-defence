#[macro_use]
extern crate criterion;
use criterion::Criterion;

#[macro_use]
extern crate lazy_static;

extern crate zombot;
use zombot::engine::{GameState, Player, GameStatus, Building, Missile};
use zombot::engine::settings::{GameSettings, BuildingSettings};
use zombot::engine::geometry::Point;
use zombot::engine::command::{Command, BuildingType};

extern crate rand;
use rand::{thread_rng, Rng};

fn create_example_state(settings: &GameSettings,
    player_buildings: usize, opponent_buildings: usize,
    player_missiles: usize, opponent_missiles: usize
) -> GameState {
    GameState {
        status: GameStatus::Continue,
        player: Player {
            energy: 30,
            health: 100
        },
        opponent: Player {
            energy: 30,
            health: 100
        },
        player_buildings: (0..player_buildings).map(|_| create_player_building(settings)).collect(),
        opponent_buildings: (0..opponent_buildings).map(|_| create_player_building(settings)).collect(),
        player_missiles: (0..player_missiles).map(|_| create_missile(settings)).collect(),
        opponent_missiles: (0..opponent_missiles).map(|_| create_missile(settings)).collect()
    }
}

fn create_example_settings() -> GameSettings {
    GameSettings {
        size: Point::new(10,10),
        energy_income: 5,
        energy: BuildingSettings {
            price: 20,
            health: 5,
            construction_time: 1,
            weapon_damage: 0,
            weapon_speed: 0,
            weapon_cooldown_period: 0,
            energy_generated_per_turn: 3
        },
        defence: BuildingSettings {
            price: 20,
            health: 5,
            construction_time: 1,
            weapon_damage: 0,
            weapon_speed: 0,
            weapon_cooldown_period: 0,
            energy_generated_per_turn: 3
        },
        attack: BuildingSettings {
            price: 20,
            health: 5,
            construction_time: 1,
            weapon_damage: 0,
            weapon_speed: 0,
            weapon_cooldown_period: 0,
            energy_generated_per_turn: 3
        }
    }
}

fn create_player_building(settings: &GameSettings) -> Building {
    let all_positions = (0..settings.size.y)
        .flat_map(|y| (0..settings.size.x/2).map(|x| Point::new(x, y)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let all_buildings = BuildingType::all();

    let mut rng = thread_rng();
    let position = rng.choose(&all_positions).unwrap();
    let building = rng.choose(&all_buildings).unwrap();
    let blueprint = settings.building_settings(*building);

    Building::new(*position, blueprint)
}

fn create_opponent_building(settings: &GameSettings) -> Building {
    let all_positions = (0..settings.size.y)
        .flat_map(|y| (settings.size.x/2..settings.size.x).map(|x| Point::new(x, y)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let all_buildings = BuildingType::all();

    let mut rng = thread_rng();
    let position = rng.choose(&all_positions).unwrap();
    let building = rng.choose(&all_buildings).unwrap();
    let blueprint = settings.building_settings(*building);

    Building::new(*position, blueprint)
}

fn create_missile(settings: &GameSettings) -> Missile {
    let all_positions = (0..settings.size.y)
        .flat_map(|y| (settings.size.x/2..settings.size.x).map(|x| Point::new(x, y)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut rng = thread_rng();
    let position = rng.choose(&all_positions).unwrap();

    Missile {
        pos: *position,
        damage: 5,
        speed: 1
    }
}


fn full_simulation_benchmark(c: &mut Criterion) {
    let settings = create_example_settings();
    let state = create_example_state(&settings, 5, 5, 5, 5);
    
    let player_command = Command::Build(Point::new(0,0),BuildingType::Defence);
    let opponent_command = Command::Build(Point::new(4,4),BuildingType::Energy);
    c.bench_function("full simulation", move |b| b.iter(|| state.simulate(&settings, player_command, opponent_command)));
}

fn full_simulation_benchmark_against_number_of_buildings(c: &mut Criterion) {
    let settings = create_example_settings();

    lazy_static! {
        static ref STATES: Vec<GameState> = {
            let settings = create_example_settings();
            (0..10)
                .map(|i| create_example_state(&settings, i*2, 0, 0, 0))
                .collect::<Vec<_>>()
        };
    }

    let player_command = Command::Build(Point::new(0,0),BuildingType::Defence);
    let opponent_command = Command::Build(Point::new(4,4),BuildingType::Energy);
    
    c.bench_function_over_inputs("player buildings variable", move |b, &state_index| b.iter(|| STATES[state_index].simulate(&settings, player_command, opponent_command)), (0..STATES.len()));
}

fn full_simulation_benchmark_against_number_of_missiles(c: &mut Criterion) {
    let settings = create_example_settings();

    lazy_static! {
        static ref STATES: Vec<GameState> = {
            let settings = create_example_settings();
            (0..10)
                .map(|i| create_example_state(&settings, 2, 5, i*2, i*2))
                .collect::<Vec<_>>()
        };
    }

    let player_command = Command::Build(Point::new(0,0),BuildingType::Defence);
    let opponent_command = Command::Build(Point::new(4,4),BuildingType::Energy);
    
    c.bench_function_over_inputs("player missiles variable", move |b, &state_index| b.iter(|| STATES[state_index].simulate(&settings, player_command, opponent_command)), (0..STATES.len()));
}

criterion_group!(benches,
                 full_simulation_benchmark,
                 full_simulation_benchmark_against_number_of_buildings,
                 full_simulation_benchmark_against_number_of_missiles);
criterion_main!(benches);
