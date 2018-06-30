use engine::settings::GameSettings;
use engine::command::*;
use engine::geometry::*;
use engine::{GameState, GameStatus, Player};

use rand::{Rng, XorShiftRng, SeedableRng};

const MAX_MOVES: u16 = 400;

use time::{Duration, PreciseTime};

#[cfg(not(feature = "single-threaded"))]
use rayon::prelude::*;

#[cfg(feature = "energy-cutoff")] pub const ENERGY_PRODUCTION_CUTOFF: u16 = 30;
#[cfg(feature = "energy-cutoff")] pub const ENERGY_STORAGE_CUTOFF: u16 = 45;

pub fn choose_move<GS: GameState>(settings: &GameSettings, state: &GS, start_time: &PreciseTime, max_time: Duration) -> Command {
    let mut command_scores = CommandScore::init_command_scores(settings, state);
    
    loop {
        #[cfg(feature = "single-threaded")]
        {
            command_scores.iter_mut()
                .for_each(|score| {
                    let mut rng = XorShiftRng::from_seed(score.next_seed);
                    simulate_to_endstate(score, settings, state, &mut rng);
                });
        }
        #[cfg(not(feature = "single-threaded"))]
        {
            command_scores.par_iter_mut()
                .for_each(|score| {
                    let mut rng = XorShiftRng::from_seed(score.next_seed);
                    simulate_to_endstate(score, settings, state, &mut rng);
                });
        }
        if start_time.to(PreciseTime::now()) > max_time {
            break;
        }
    }

    let command = command_scores.iter().max_by_key(|&c| c.win_ratio());

    #[cfg(feature = "benchmarking")]
    {
        let total_iterations: u32 = command_scores.iter().map(|c| c.attempts).sum();
        println!("Iterations: {}", total_iterations);
    }
    
    match command {
        Some(command) => command.command,
        _ => Command::Nothing
    }
}

fn simulate_to_endstate<R: Rng, GS: GameState>(command_score: &mut CommandScore, settings: &GameSettings, state: &GS, rng: &mut R) {
    let mut state_mut = state.clone();
    
    let opponent_first = random_opponent_move(settings, &state_mut, rng);
    let mut status = state_mut.simulate(settings, command_score.command, opponent_first);
    
    for _ in 0..MAX_MOVES {
        if status != GameStatus::Continue {
            break;
        }

        let player_command = random_player_move(settings, &state_mut, rng);
        let opponent_command = random_opponent_move(settings, &state_mut, rng);
        status = state_mut.simulate(settings, player_command, opponent_command);
    }

    let next_seed = [rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32()];
    match status {
        GameStatus::PlayerWon => command_score.add_victory(next_seed),
        GameStatus::OpponentWon => command_score.add_defeat(next_seed),
        GameStatus::Continue => command_score.add_stalemate(next_seed),
        GameStatus::Draw => command_score.add_draw(next_seed)
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

#[derive(Debug)]
struct CommandScore {
    command: Command,
    victories: u32,
    defeats: u32,
    draws: u32,
    stalemates: u32,
    attempts: u32,
    next_seed: [u32; 4]
}

impl CommandScore {
    fn new(command: Command) -> CommandScore {
        CommandScore {
            command,
            victories: 0,
            defeats: 0,
            draws: 0,
            stalemates: 0,
            attempts: 0,
            next_seed: [0x7b6a_e1f4, 0x413c_e90f, 0x6781_6799, 0x770a_6bda]
        }
    }

    fn add_victory(&mut self, next_seed: [u32; 4]) {
        self.victories += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_defeat(&mut self, next_seed: [u32; 4]) {
        self.defeats += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_draw(&mut self, next_seed: [u32; 4]) {
        self.draws += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_stalemate(&mut self, next_seed: [u32; 4]) {
        self.stalemates += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn win_ratio(&self) -> i32 {
        (self.victories as i32 - self.defeats as i32) * 10000 / (self.attempts as i32)
    }
    
    fn init_command_scores<GS: GameState>(settings: &GameSettings, state: &GS) -> Vec<CommandScore> {
        let all_buildings = sensible_buildings(settings, &state.player(), state.player_has_max_teslas());

        let building_command_count = state.unoccupied_player_cells().len()*all_buildings.len();
        let nothing_count = 1;
        
        let mut commands = Vec::with_capacity(building_command_count + nothing_count);
        commands.push(CommandScore::new(Command::Nothing));

        for &position in state.unoccupied_player_cells() {
            for &building in &all_buildings {
                commands.push(CommandScore::new(Command::Build(position, building)));
            }
        }

        commands
    }
}

#[cfg(not(feature = "energy-cutoff"))]
fn sensible_buildings(settings: &GameSettings, player: &Player, has_max_teslas: bool) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);
    for b in BuildingType::all().iter() {
        let building_setting = settings.building_settings(*b);
        let affordable = building_setting.price <= player.energy;
        let is_tesla = b == BuildingType::Tesla;
        if affordable && (!is_tesla || !has_max_teslas) {
            result.push(*b);
        }
    }
    result
}


#[cfg(feature = "energy-cutoff")]
fn sensible_buildings(settings: &GameSettings, player: &Player, has_max_teslas: bool) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);
    let needs_energy = player.energy_generated <= ENERGY_PRODUCTION_CUTOFF ||
        player.energy <= ENERGY_STORAGE_CUTOFF;
    
    for b in BuildingType::all().iter() {
        let building_setting = settings.building_settings(*b);
        let affordable = building_setting.price <= player.energy;
        let energy_producing = building_setting.energy_generated_per_turn > 0;
        let is_tesla = *b == BuildingType::Tesla;
        if affordable && (!energy_producing || needs_energy) && (!is_tesla || !has_max_teslas) {
            result.push(*b);
        }
    }
    result
}

