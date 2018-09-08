extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;
use zombot::engine::constants::*;

const STATE_PATH: &str = "tests/state0.json";

use std::process;

fn main() {
    println!("Running bitwise engine");
    let start_time = PreciseTime::now();
    let state = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let max_time = Duration::milliseconds(MAX_TIME_MILLIS);

    #[cfg(feature = "full-monte-carlo-tree")] strategy::monte_carlo_tree::choose_move(&state, start_time, max_time);
    #[cfg(not(feature = "full-monte-carlo-tree"))] strategy::monte_carlo::choose_move(&state, start_time, max_time);
}
