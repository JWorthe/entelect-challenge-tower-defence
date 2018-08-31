use engine::command::*;
use engine::status::GameStatus;
use engine::bitwise_engine::{Player, BitwiseGameState};
use engine::constants::*;
use engine::geometry::*;

use std::fmt;

use rand::{Rng, XorShiftRng, SeedableRng};

use arrayvec::ArrayVec;

const MAX_MOVES: u16 = 400;
const INIT_SEED: [u8;16] = [0x7b, 0x6a, 0xe1, 0xf4, 0x41, 0x3c, 0xe9, 0x0f, 0x67, 0x81, 0x67, 0x99, 0x77, 0x0a, 0x6b, 0xda];

use time::{Duration, PreciseTime};

#[cfg(not(feature = "single-threaded"))]
use rayon::prelude::*;

//TODO Rethink / adjust these?
#[cfg(feature = "energy-cutoff")] pub const ENERGY_PRODUCTION_CUTOFF: u16 = 50;
#[cfg(feature = "energy-cutoff")] pub const ENERGY_STORAGE_CUTOFF: u16 = 100;

pub fn choose_move(state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Command {
    let mut command_scores = CommandScore::init_command_scores(state);

    let command = {
        let best_command_score = simulate_options_to_timeout(&mut command_scores, state, start_time, max_time);
        match best_command_score {
            Some(best) if !best.starts_with_nothing => best.command,
            _ => Command::Nothing
        }
    };

    #[cfg(feature = "benchmarking")]
    {
        let total_iterations: u32 = command_scores.iter().map(|c| c.attempts).sum();
        println!("Iterations: {}", total_iterations);
    }
    #[cfg(feature = "debug-decisions")]
    {
        debug_print_choices("ENERGY", &command_scores, |score| match score.command {
            Command::Build(p, BuildingType::Energy) => Some((p, score.win_ratio())),
            _ => None
        });
        debug_print_choices("ATTACK", &command_scores, |score| match score.command {
            Command::Build(p, BuildingType::Attack) => Some((p, score.win_ratio())),
            _ => None
        });
        debug_print_choices("DEFENCE", &command_scores, |score| match score.command {
            Command::Build(p, BuildingType::Defence) => Some((p, score.win_ratio())),
            _ => None
        });
        debug_print_choices("TESLA", &command_scores, |score| match score.command {
            Command::Build(p, BuildingType::Tesla) => Some((p, score.win_ratio())),
            _ => None
        });
        
        println!("NOTHING");
        println!("{}", command_scores.iter().find(|c| c.command == Command::Nothing).map(|s| s.win_ratio()).unwrap_or(0));
        println!("");

        println!("IRON CURTAIN");
        println!("{}", command_scores.iter().find(|c| c.command == Command::IronCurtain).map(|s| s.win_ratio()).unwrap_or(0));
        println!("");
    }

    command
}

#[cfg(feature = "debug-decisions")]
fn debug_print_choices<F: FnMut(&CommandScore) -> Option<(Point, i32)>>(label: &str, command_scores: &[CommandScore], extractor: F) {
    println!("#+NAME: {}", label);
    println!("#+PLOT: type:3d with:pm3d");
    let relevant_moves: Vec<(Point, i32)>  = command_scores.iter()
        .filter_map(extractor)
        .collect();
    for y in 0..MAP_HEIGHT {
        for x in 0..SINGLE_MAP_WIDTH {
            let point = Point::new(x, y);
            let score = relevant_moves.iter().find(|(p, _)| *p == point);
            print!(" | {}", score.map(|(_,s)| s).cloned().unwrap_or(0));
        }
        println!(" |");
    }
    println!("");
}

#[cfg(not(feature = "discard-poor-performers"))]
fn simulate_options_to_timeout(command_scores: &'a mut Vec<CommandScore>, settings: &GameSettings, state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Option<&'a CommandScore> {
    loop {
        simulate_all_options_once(command_scores, settings, state);
        if start_time.to(PreciseTime::now()) > max_time {
            break;
        }
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
    
    let mut status = GameStatus::Continue; //state_mut.simulate(command_score.command, opponent_first);
    let mut first_move_made = false;
    
    for _ in 0..MAX_MOVES {
        if status != GameStatus::Continue {
            break;
        }

        let player_command = if first_move_made {
            random_move(&state_mut.player, rng)
        } else {
            let do_nothing = command_score.command.cant_build_yet(state_mut.player.energy);
            first_move_made = !do_nothing;
            if do_nothing { Command::Nothing } else { command_score.command }
        };
        let opponent_command = random_move(&state_mut.opponent, rng);
        status = state_mut.simulate(player_command, opponent_command);
    }

    let mut next_seed: [u8;16] = [0; 16];
    rng.fill_bytes(&mut next_seed);
    match status {
        GameStatus::PlayerWon => command_score.add_victory(next_seed),
        GameStatus::OpponentWon => command_score.add_defeat(next_seed),
        GameStatus::Continue => command_score.add_stalemate(next_seed),
        GameStatus::Draw => command_score.add_draw(next_seed)
    }
}

// TODO
// 1. Have a (static) array of all moves. Even invalid ones. ALL
// 2. Create a new CDF array, same size.
// 3. Loop moves
// 3.1. Compute PDF for move. Invalid moves are 0.
// 3.2. Add to last CDF value and stick in array
// 4. Generate random number uniformly, 0 to CDF max
// 5. Binary search to find random number in CDF array. Min index where CDF[index] > random
// 6. Lookup move in static array
#[cfg(feature = "heuristic-random")]
fn random_move<R: Rng>(player: &Player, rng: &mut R) -> Command {
    Command::Nothing
}

#[cfg(not(feature = "heuristic-random"))]
fn random_move<R: Rng>(player: &Player, rng: &mut R) -> Command {
    let free_positions_count = player.unoccupied_cell_count();

    let open_building_spot = free_positions_count > 0;

    let all_buildings = sensible_buildings(player, open_building_spot);

    let iron_curtain_count = if player.can_build_iron_curtain() && player.energy >= IRON_CURTAIN_PRICE { 1 } else { 0 };
    let nothing_count = 1;

    let building_choice_index = rng.gen_range(0, all_buildings.len() + nothing_count + iron_curtain_count);

    if building_choice_index < all_buildings.len() {
        let position_choice = rng.gen_range(0, free_positions_count);
        Command::Build(
            player.location_of_unoccupied_cell(position_choice),
            all_buildings[building_choice_index]
        )
    }
    else if building_choice_index == all_buildings.len() {
        Command::Nothing        
    } else {
        Command::IronCurtain
    }
}

#[derive(Debug)]
struct CommandScore {
    command: Command,
    starts_with_nothing: bool,
    victories: u32,
    defeats: u32,
    draws: u32,
    stalemates: u32,
    attempts: u32,
    next_seed: [u8; 16]
}

impl CommandScore {
    fn new(command: Command, starts_with_nothing: bool) -> CommandScore {
        CommandScore {
            command, starts_with_nothing,
            victories: 0,
            defeats: 0,
            draws: 0,
            stalemates: 0,
            attempts: 0,
            next_seed: INIT_SEED
        }
    }

    fn add_victory(&mut self, next_seed: [u8; 16]) {
        self.victories += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_defeat(&mut self, next_seed: [u8; 16]) {
        self.defeats += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_draw(&mut self, next_seed: [u8; 16]) {
        self.draws += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn add_stalemate(&mut self, next_seed: [u8; 16]) {
        self.stalemates += 1;
        self.attempts += 1;
        self.next_seed = next_seed;
    }

    fn win_ratio(&self) -> i32 {
        (self.victories as i32 - self.defeats as i32) * 10000 / (self.attempts as i32)
    }

    fn init_command_scores(state: &BitwiseGameState) -> Vec<CommandScore> {
        let unoccupied_cells_count = state.player.unoccupied_cell_count();
        let unoccupied_cells = (0..unoccupied_cells_count)
            .map(|i| state.player.location_of_unoccupied_cell(i));
        let energy_generated = state.player.energy_generated();

        let mut all_buildings: ArrayVec<[BuildingType; 4]> = ArrayVec::new();
        if DEFENCE_PRICE <= state.player.energy {
            all_buildings.push(BuildingType::Defence);
        }
        if MISSILE_PRICE <= state.player.energy {
            all_buildings.push(BuildingType::Attack);
        }
        if ENERGY_PRICE <= state.player.energy {
            all_buildings.push(BuildingType::Energy);
        }
        if !state.player.has_max_teslas() && (TESLA_PRICE.saturating_sub(state.player.energy) / energy_generated < 4) {
            all_buildings.push(BuildingType::Tesla);
        }
        
        let building_command_count = unoccupied_cells.len()*all_buildings.len();
        
        let mut commands = Vec::with_capacity(building_command_count + 1);
        if state.player.can_build_iron_curtain() && IRON_CURTAIN_PRICE.saturating_sub(state.player.energy) / energy_generated < 4 {
            commands.push(CommandScore::new(Command::IronCurtain, state.player.energy < IRON_CURTAIN_PRICE));
        }

        for position in unoccupied_cells {
            for &building in &all_buildings {
                commands.push(CommandScore::new(Command::Build(position, building), building.cant_build_yet(state.player.energy)));
            }
        }

        commands
    }
}

impl fmt::Display for CommandScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.command, self.win_ratio())
    }
}

#[cfg(not(feature = "energy-cutoff"))]
fn sensible_buildings(player: &Player, open_building_spot: bool) -> ArrayVec<[BuildingType;4]> {
    let mut result = ArrayVec::new();
    if !open_building_spot {
        return result;
    }

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

#[cfg(feature = "energy-cutoff")]
fn sensible_buildings(player: &Player, open_building_spot: bool) -> ArrayVec<[BuildingType;4]> {
    let mut result = ArrayVec::new();
    if !open_building_spot {
        return result;
    }

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

