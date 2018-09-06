use engine::command::*;
use engine::status::GameStatus;
use engine::bitwise_engine::{Player, BitwiseGameState};
use engine::constants::*;
use engine::geometry::*;

use rand::{Rng, XorShiftRng, SeedableRng};
use time::{Duration, PreciseTime};

use strategy::monte_carlo;

use arrayvec::ArrayVec;

#[derive(Debug)]
struct NodeStats {
    wins: f32,
    losses: f32,
    attempts: f32,
    explored: Vec<(Command, NodeStats)>,
    unexplored: Vec<Command>
}

impl NodeStats {
    fn create_node(player: &Player) -> NodeStats {
        let unoccupied_cells_count = player.unoccupied_cell_count();
        let unoccupied_cells = (0..unoccupied_cells_count)
            .map(|i| player.location_of_unoccupied_cell(i));

        let mut all_buildings: ArrayVec<[BuildingType; NUMBER_OF_BUILDING_TYPES]> = ArrayVec::new();
        if DEFENCE_PRICE <= player.energy {
            all_buildings.push(BuildingType::Defence);
        }
        if MISSILE_PRICE <= player.energy {
            all_buildings.push(BuildingType::Attack);
        }
        if ENERGY_PRICE <= player.energy {
            all_buildings.push(BuildingType::Energy);
        }
        if TESLA_PRICE <= player.energy && !player.has_max_teslas() {
            all_buildings.push(BuildingType::Tesla);
        }
        
        let building_command_count = unoccupied_cells.len()*all_buildings.len();

        let mut commands = Vec::with_capacity(building_command_count + 2);

        commands.push(Command::Nothing);
        if IRON_CURTAIN_PRICE <= player.energy && player.can_build_iron_curtain() {
            commands.push(Command::IronCurtain);
        }

        for position in unoccupied_cells {
            for &building in &all_buildings {
                commands.push(Command::Build(position, building));
            }
        }
        
        NodeStats {
            wins: 0.,
            losses: 0.,
            attempts: 0.,
            explored: Vec::with_capacity(commands.len()),
            unexplored: commands
        }
    }
    
    fn node_with_highest_ucb<'a>(&'a mut self) -> &'a mut (Command, NodeStats) {
        debug_assert!(self.unexplored.is_empty());
        debug_assert!(self.explored.len() > 0);
        let total_attempts = self.explored.iter().map(|(_, n)| n.attempts).sum::<f32>();

        let mut max_position = 0;
        let mut max_value = self.explored[0].1.ucb(total_attempts);
        for i in 1..self.explored.len() {
            let value = self.explored[i].1.ucb(total_attempts);
            if value > max_value {
                max_position = i;
                max_value = value;
            }
        }
        &mut self.explored[max_position]
    }

    fn ucb(&self, n: f32) -> f32 {
        self.wins / self.attempts + (2.0 * n / self.attempts).sqrt()
    }

    fn add_node<'a>(&'a mut self, player: &Player, command: Command) -> &'a mut (Command, NodeStats) {
        let node = NodeStats::create_node(player);
        self.explored.push((command, node));
        self.unexplored.retain(|c| *c != command);
        self.explored.last_mut().unwrap()
    }

    fn add_victory(&mut self) {
        self.attempts += 1.;
        self.wins += 1.;
    }
    fn add_defeat(&mut self) {
        self.attempts += 1.;
        self.losses += 1.;
    }
    fn add_draw(&mut self) {
        self.attempts += 1.;
    }

    fn count_explored(&self) -> usize {
        1 + self.explored.iter().map(|(_, n)| n.count_explored()).sum::<usize>()
    }
}

pub fn choose_move(state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Command {
    let mut rng = XorShiftRng::from_seed(INIT_SEED);
    
    let mut root = NodeStats::create_node(&state.player);

    while start_time.to(PreciseTime::now()) < max_time {
        tree_search(&state, &mut root, &mut rng);
    }

    #[cfg(feature = "benchmarking")]
    {
        println!("Explored nodes: {}", root.count_explored());
    }

    let (command, _) = root.node_with_highest_ucb();
    command.clone()
}

fn tree_search<R: Rng>(state: &BitwiseGameState, stats: &mut NodeStats, rng: &mut R) -> GameStatus {
    // root is opponent move
    // node being added is player move
    
    if state.round >= MAX_MOVES {
        return GameStatus::Draw
    }
    
    if stats.unexplored.is_empty() {
        let result = {
            let (next_command, next_tree) = stats.node_with_highest_ucb();
            tree_search_opponent(state, next_tree, next_command.clone(), rng)
        };
        match result {
            GameStatus::PlayerWon => {stats.add_defeat()},
            GameStatus::OpponentWon => {stats.add_victory()},
            _ => {stats.add_draw()}
        };
        result
    } else {
        let next_command = rng.choose(&stats.unexplored).expect("Partially explored had no options").clone();
        let result = {
            let (_, next_stats) = stats.add_node(&state.opponent, next_command);

            let opponent_random = monte_carlo::random_move(&state.opponent, &state.player, rng);
            let mut next_state = state.clone();
            next_state.simulate(next_command, opponent_random);

            let result = simulate_to_endstate(next_state, rng);
            match result {
                GameStatus::PlayerWon => {next_stats.add_victory()},
                GameStatus::OpponentWon => {next_stats.add_defeat()},
                _ => {next_stats.add_draw()}
            };
            
            result
        };

        match result {
            GameStatus::PlayerWon => {stats.add_defeat()},
            GameStatus::OpponentWon => {stats.add_victory()},
            _ => {stats.add_draw()}
        };
        result
    }
}

fn tree_search_opponent<R: Rng>(state: &BitwiseGameState, stats: &mut NodeStats, player_command: Command, rng: &mut R) -> GameStatus {
    // root is player move
    // node being added is opponent move

    if stats.unexplored.is_empty() {
        let result = {
            let (next_command, next_tree) = stats.node_with_highest_ucb();
            let mut next_state = state.clone();
            next_state.simulate(player_command, next_command.clone());
            tree_search(&next_state, next_tree, rng)
        };
        match result {
            GameStatus::PlayerWon => {stats.add_victory()},
            GameStatus::OpponentWon => {stats.add_defeat()},
            _ => {stats.add_draw()}
        };
        result
    } else {
        let next_command = rng.choose(&stats.unexplored).expect("Partially explored had no options").clone();
        let mut next_state = state.clone();
        next_state.simulate(player_command, next_command);

        let result = {
            let (_, next_stats) = stats.add_node(&next_state.player, next_command);

            let result = simulate_to_endstate(next_state, rng);
            match result {
                GameStatus::PlayerWon => {next_stats.add_defeat()},
                GameStatus::OpponentWon => {next_stats.add_victory()},
                _ => {next_stats.add_draw()}
            };
            
            result
        };
        
        match result {
            GameStatus::PlayerWon => {stats.add_victory()},
            GameStatus::OpponentWon => {stats.add_defeat()},
            _ => {stats.add_draw()}
        };
        result
    }
}


fn simulate_to_endstate<R: Rng>(mut state: BitwiseGameState, rng: &mut R) -> GameStatus  {
    let mut status = GameStatus::Continue;
    
    while status == GameStatus::Continue && state.round < MAX_MOVES {
        let player_command = monte_carlo::random_move(&state.player, &state.opponent, rng);
        let opponent_command = monte_carlo::random_move(&state.opponent, &state.player, rng);
        status = state.simulate(player_command, opponent_command);
    }
    status
}

