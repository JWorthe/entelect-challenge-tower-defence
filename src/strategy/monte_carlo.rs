use engine::command::*;
use engine::status::GameStatus;
use engine::bitwise_engine::{Player, BitwiseGameState};
use engine::constants::*;

use rand::{Rng, XorShiftRng, SeedableRng};

const MAX_MOVES: u16 = 400;

use time::{Duration, PreciseTime};

#[cfg(not(feature = "single-threaded"))]
use rayon::prelude::*;

//TODO Rethink / adjust these?
#[cfg(feature = "energy-cutoff")] pub const ENERGY_PRODUCTION_CUTOFF: u16 = 100;
#[cfg(feature = "energy-cutoff")] pub const ENERGY_STORAGE_CUTOFF: u16 = 100;

pub fn choose_move(state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Command {
    let mut command_scores = CommandScore::init_command_scores(state);
    let command = simulate_options_to_timeout(&mut command_scores, state, start_time, max_time);
    
    match command {
        Some(command) => command.command,
        _ => Command::Nothing
    }
}

#[cfg(not(feature = "discard-poor-performers"))]
fn simulate_options_to_timeout(command_scores: &'a mut Vec<CommandScore>, settings: &GameSettings, state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Option<&'a CommandScore> {
    loop {
        simulate_all_options_once(command_scores, settings, state);
        if start_time.to(PreciseTime::now()) > max_time {
            break;
        }
    }

    #[cfg(feature = "benchmarking")]
    {
        let total_iterations: u32 = command_scores.iter().map(|c| c.attempts).sum();
        println!("Iterations: {}", total_iterations);
    }

    command_scores.iter().max_by_key(|&c| c.win_ratio())
}

#[cfg(feature = "discard-poor-performers")]
fn simulate_options_to_timeout<'a>(command_scores: &'a mut Vec<CommandScore>, state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Option<&'a CommandScore> {
    use std::cmp;
    let min_options = cmp::min(command_scores.len(), 5);
    
    let maxes = [max_time / 4, max_time / 2, max_time * 3 / 4, max_time];
    for (i, &max) in maxes.iter().enumerate() {
        let new_length = cmp::max(min_options, command_scores.len() / (2usize.pow(i as u32)));
        let active_scores = &mut command_scores[0..new_length];
        loop {
            simulate_all_options_once(active_scores, state);
            if start_time.to(PreciseTime::now()) > max {
                break;
            }
        }
        active_scores.sort_unstable_by_key(|c| -c.win_ratio());
    }

    #[cfg(feature = "benchmarking")]
    {
        let total_iterations: u32 = command_scores.iter().map(|c| c.attempts).sum();
        println!("Iterations: {}", total_iterations);
    }

    command_scores.first()
}

#[cfg(feature = "single-threaded")]
fn simulate_all_options_once(command_scores: &mut[CommandScore], state: &BitwiseGameState) {
    command_scores.iter_mut()
        .for_each(|score| {
            let mut rng = XorShiftRng::from_seed(score.next_seed);
            simulate_to_endstate(score, state, &mut rng);
        });
}

#[cfg(not(feature = "single-threaded"))]
fn simulate_all_options_once(command_scores: &mut[CommandScore], state: &BitwiseGameState) {
    command_scores.par_iter_mut()
        .for_each(|score| {
            let mut rng = XorShiftRng::from_seed(score.next_seed);
            simulate_to_endstate(score, state, &mut rng);
        });
}

fn simulate_to_endstate<R: Rng>(command_score: &mut CommandScore, state: &BitwiseGameState, rng: &mut R) {
    let mut state_mut = state.clone();
    
    let opponent_first = random_move(&state_mut.opponent, rng);
    let mut status = state_mut.simulate(command_score.command, opponent_first);
    
    for _ in 0..MAX_MOVES {
        if status != GameStatus::Continue {
            break;
        }

        let player_command = random_move(&state_mut.player, rng);
        let opponent_command = random_move(&state_mut.opponent, rng);
        status = state_mut.simulate(player_command, opponent_command);
    }

    let next_seed = [rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32()];
    match status {
        GameStatus::PlayerWon => command_score.add_victory(next_seed),
        GameStatus::OpponentWon => command_score.add_defeat(next_seed),
        GameStatus::Continue => command_score.add_stalemate(next_seed),
        GameStatus::Draw => command_score.add_draw(next_seed)
    }
}

// TODO: Given enough energy, most opponents won't do nothing
fn random_move<R: Rng>(player: &Player, rng: &mut R) -> Command {
    let all_buildings = sensible_buildings(player);
    let nothing_count = 1;
    let iron_curtain_count = if player.can_build_iron_curtain() { 1 } else { 0 };
    let free_positions_count = player.unoccupied_cell_count();
        
    let building_choice_index = rng.gen_range(0, all_buildings.len() + nothing_count + iron_curtain_count);
    
    if building_choice_index == all_buildings.len() {
        Command::Nothing
    } else if iron_curtain_count > 0 && building_choice_index == all_buildings.len() + 1 {
        Command::IronCurtain
    } else if free_positions_count > 0 {
        let position_choice = rng.gen_range(0, free_positions_count);
        Command::Build(
            player.location_of_unoccupied_cell(position_choice),
            all_buildings[building_choice_index]
        )
    } else {
        Command::Nothing
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

    //TODO: Devalue nothing so that it doesn't stand and do nothing when it can do things
    fn init_command_scores(state: &BitwiseGameState) -> Vec<CommandScore> {
        let all_buildings = sensible_buildings(&state.player);

        let unoccupied_cells = (0..state.player.unoccupied_cell_count()).map(|i| state.player.location_of_unoccupied_cell(i));

        let building_command_count = unoccupied_cells.len()*all_buildings.len();
        
        let mut commands = Vec::with_capacity(building_command_count + 2);
        commands.push(CommandScore::new(Command::Nothing));
        if state.player.can_build_iron_curtain() {
            commands.push(CommandScore::new(Command::IronCurtain));
        }

        for position in unoccupied_cells {
            for &building in &all_buildings {
                commands.push(CommandScore::new(Command::Build(position, building)));
            }
        }

        commands
    }
}

#[cfg(not(feature = "energy-cutoff"))]
fn sensible_buildings(player: &Player) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);

    if DEFENCE_PRICE <= player.energy {
        result.push(BuildingType::Defence);
    }
    if MISSILE_PRICE <= player.energy {
        result.push(BuildingType::Attack);
    }
    if ENERGY_PRICE <= player.energy {
        result.push(BuildingType::Energy);
    }
    if TESLA_PRICE <= player.energy && !player.has_max_teslas() {
        result.push(BuildingType::Tesla);
    }

    result
}


//TODO: Heuristic that avoids building the initial energy towers all in the same row? Max energy in a row?
#[cfg(feature = "energy-cutoff")]
fn sensible_buildings(player: &Player) -> Vec<BuildingType> {
    let mut result = Vec::with_capacity(4);
    let needs_energy = player.energy_generated() <= ENERGY_PRODUCTION_CUTOFF ||
        player.energy <= ENERGY_STORAGE_CUTOFF;

    if DEFENCE_PRICE <= player.energy {
        result.push(BuildingType::Defence);
    }
    if MISSILE_PRICE <= player.energy {
        result.push(BuildingType::Attack);
    }
    if ENERGY_PRICE <= player.energy && needs_energy {
        result.push(BuildingType::Energy);
    }
    if TESLA_PRICE <= player.energy && !player.has_max_teslas() {
        result.push(BuildingType::Tesla);
    }
    
    result
}

