use engine::command::*;
use engine::status::GameStatus;
use engine::bitwise_engine::{Player, BitwiseGameState};
use engine::constants::*;
use engine::geometry::*;

use rand::{Rng, XorShiftRng, SeedableRng};
use time::{Duration, PreciseTime};


enum SearchTree {
    Leaf(NodeStats),
    FullyExploredNode(FullyExploredStats),
    PartiallyExploredNode(PartiallyExploredStats)
}

struct NodeStats {
    wins: u32,
    attempts: u32
}

struct FullyExploredStats {
    wins: u32,
    attempts: u32,
    explored: Vec<(Command, SearchTree)>
}

struct PartiallyExploredStats {
    wins: u32,
    attempts: u32,
    explored: Vec<(Command, SearchTree)>,
    unexplored: Vec<Command>
}

impl SearchTree {
    fn create_node(state: &Player) -> SearchTree {
        SearchTree::PartiallyExploredNode(PartiallyExploredStats {
            wins: 0,
            attempts: 0,
            explored: Vec::new(),
            unexplored: Vec::new() //TODO
        })
    }
}

impl FullyExploredStats {
    fn node_with_highest_ucb<'a>(&'a mut self) -> &'a mut (Command, SearchTree) {
        //TODO
        &mut self.explored[0]
    }
}

impl PartiallyExploredStats {
    fn add_node<'a>(&'a mut self, state: &Player, command: Command) -> &'a mut (Command, SearchTree) {
        //TODO: Insert
        let node = SearchTree::create_node(state);
        self.explored.push((command, node));
        self.explored.last_mut().unwrap()
    }
}

use self::SearchTree::*;
pub fn choose_move(state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Command {
    
    
    // create root node as partially explored node
    // creating a new node needs to populate all (valid) unexplored moves

    let mut root = SearchTree::create_node(&state.player);

    loop {
        // TODO: Break out!
        tree_search(&state, &mut root);
    }
    
    Command::Nothing    
}

// TODO: Max depth

fn tree_search(state: &BitwiseGameState, tree: &mut SearchTree) -> bool {
    match tree {
        Leaf(stats) => {
            // ???
            false
        },
        FullyExploredNode(ref mut stats) => {
            let (next_command, next_tree) = stats.node_with_highest_ucb();
            tree_search_opponent(state, next_tree, next_command.clone())
            // TODO: Back-propagation?
        },
        PartiallyExploredNode(ref mut stats) => {
            let next_command = stats.unexplored[0].clone(); // TODO: Random
            let next_tree = stats.add_node(&state.opponent, next_command);

            // TODO: simulate to end
            // TODO: Back-propagate
            false
        }
    }
}

fn tree_search_opponent(state: &BitwiseGameState, tree: &mut SearchTree, player_command: Command) -> bool {
    match tree {
        Leaf(stats) => {
            // ???
            false
        },
        FullyExploredNode(ref mut stats) => {
            let (next_command, next_tree) = stats.node_with_highest_ucb();
            let mut next_state = state.clone();
            next_state.simulate(player_command, next_command.clone());
            tree_search(&next_state, next_tree)
            // TODO: Back-propagation?
        },
        PartiallyExploredNode(ref mut stats) => {
            let next_command = stats.unexplored[0].clone(); // TODO: Random
            
            let mut next_state = state.clone();
            next_state.simulate(player_command, next_command.clone());
            
            let next_tree = stats.add_node(&next_state.player, next_command);

            // TODO: simulate to end
            // TODO: Back-propagate
            false
        }
    }
}
