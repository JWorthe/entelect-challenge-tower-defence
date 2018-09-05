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
    fn node_with_highest_ucb<'a>(&'a self) -> &'a (Command, SearchTree) {
        //TODO
        &self.explored[0]
    }
}

impl PartiallyExploredStats {
    fn add_node(&mut self, command: Command) {
        //TODO
    }
}


pub fn choose_move(state: &BitwiseGameState, start_time: PreciseTime, max_time: Duration) -> Command {
    use self::SearchTree::*;
    
    // create root node as partially explored node
    // creating a new node needs to populate all (valid) unexplored moves

    let root = SearchTree::create_node(&state.player);

    loop {
        //tree_search(&state, &mut root);
    }
    
    Command::Nothing    
}

/*
fn tree_search(state: &BitwiseGameState, tree: &mut SearchTree) -> bool {
    match tree {
        Leaf(stats) => {
            // ???
            false
        },
        FullyExploredNode(stats) => {
            let (next_command, next_tree) = stats.node_with_highest_ucb();
            tree_search(state, &mut next_tree)
            // TODO: Swap players?
            // TODO: Back-propagation?
        },
        PartiallyExploredNode(stats) => {
            //   choose random command and add as partially explored node to the tree
            //   simulate to end with random commands
            //   back-propagate (remember to keep a stack of commands to that point node)
            //   convert to fully explored if applicable
        }
    }
    
}
*/
