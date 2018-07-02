extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;

const STATE_PATH: &str = "tests/state0.json";

use std::process;

fn main() {
//    expressive();
    bitwise();
}

fn expressive() {
    println!("Running expressive engine");
    let start_time = PreciseTime::now();
    let (settings, state) = match input::json::read_expressive_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let max_time = Duration::milliseconds(1950);
    strategy::monte_carlo::choose_move(&settings, &state, &start_time, max_time);
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
    let max_time = Duration::milliseconds(1950);
    strategy::monte_carlo::choose_move(&settings, &state, &start_time, max_time);
}
