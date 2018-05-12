use engine::settings::GameSettings;
use engine::command::*;
use engine::{GameState, GameStatus};

use rand::{thread_rng, Rng};

const MAX_MOVES: u16 = 400;

// TODO Round start time here
pub fn choose_move(settings: &GameSettings, state: &GameState) -> Command {
    println!("Using MONTE_CARLO strategy");
    
    let mut rng = thread_rng();
    let mut command_scores = CommandScore::init_command_scores(settings, state);

    // TODO Repeat this until time is out
    for _ in 0..1000 {
        for mut score in &mut command_scores {
            if simulate_to_endstate(settings, state, score.command, &mut rng) {
                score.add_victory();
            } else {
                score.add_defeat();
            }
        }
    }

    let command = command_scores.iter().max_by_key(|&c| c.win_ratio());
    
    match command {
        Some(ref command) => command.command,
        _ => Command::Nothing
    }
}

fn simulate_to_endstate<R: Rng>(settings: &GameSettings, state: &GameState, command: Command, rng: &mut R) -> bool {
    let opponent_first = random_opponent_move(settings, state, rng);
    let mut state_mut = state.simulate(settings, command, opponent_first);
    
    for _ in 0..MAX_MOVES {
        if state_mut.status != GameStatus::Continue {
            break;
        }

        let player_command = random_player_move(settings, state, rng);
        let opponent_command = random_opponent_move(settings, state, rng);
        state_mut.simulate_mut(settings, player_command, opponent_command);
        
    }
    
    state_mut.status == GameStatus::PlayerWon
}

fn random_player_move<R: Rng>(settings: &GameSettings, state: &GameState, rng: &mut R) -> Command {
    let all_commands = enumerate_player_commands(settings, state);
    rng.choose(&all_commands).cloned().unwrap_or(Command::Nothing)
}
fn random_opponent_move<R: Rng>(settings: &GameSettings, state: &GameState, rng: &mut R) -> Command {
    let all_commands = enumerate_opponent_commands(settings, state);
    rng.choose(&all_commands).cloned().unwrap_or(Command::Nothing)
}


struct CommandScore {
    command: Command,
    victories: u32,
    attempts: u32
}

impl CommandScore {
    fn new(command: Command) -> CommandScore {
        CommandScore {
            command: command,
            victories: 0,
            attempts: 0
        }
    }

    fn add_victory(&mut self) {
        self.victories += 1;
        self.attempts += 1;
    }

    fn add_defeat(&mut self) {
        self.attempts += 1;
    }

    fn win_ratio(&self) -> u32 {
        self.victories * 1000 / self.attempts
    }
    
    fn init_command_scores(settings: &GameSettings, state: &GameState) -> Vec<CommandScore> {
        enumerate_player_commands(settings, state).iter()
            .map(|&c| CommandScore::new(c))
            .collect()
    }
}

fn enumerate_player_commands(settings: &GameSettings, state: &GameState) -> Vec<Command> {
    let all_positions = state.unoccupied_player_cells(settings);
    let all_buildings = state.player_affordable_buildings(settings);
    
    let build_commands = all_positions.iter()
        .flat_map(|&pos| all_buildings.iter()
                  .map(|&building| Command::Build(pos, building)).collect::<Vec<_>>()
        );
    let other_commands = vec!(Command::Nothing);

    build_commands.chain(other_commands)
        .collect()
}

fn enumerate_opponent_commands(settings: &GameSettings, state: &GameState) -> Vec<Command> {
    let all_positions = state.unoccupied_opponent_cells(settings);
    let all_buildings = state.opponent_affordable_buildings(settings);
    
    let build_commands = all_positions.iter()
        .flat_map(|&pos| all_buildings.iter()
                  .map(|&building| Command::Build(pos, building)).collect::<Vec<_>>()
        );
    let other_commands = vec!(Command::Nothing);

    build_commands.chain(other_commands)
        .collect()
}
