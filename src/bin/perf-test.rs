extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;
use zombot::engine::constants::*;

const STATE_PATH: &str = "tests/state0.json";

use std::process;

fn main() {
    bitwise();
}

fn bitwise() {
    println!("Running bitwise engine");
    let start_time = PreciseTime::now();
    let (settings, state) = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let max_time = Duration::milliseconds(MAX_TIME_MILLIS);
    strategy::monte_carlo::choose_move(&settings, &state, &start_time, max_time);
}
