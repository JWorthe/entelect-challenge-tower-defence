extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;
use zombot::engine::constants::*;
use zombot::engine::command::Command;

use std::error::Error;

const STATE_PATH: &str = "state.json";

const COMMAND_PATH: &str = "command.txt";

use std::fs::File;
use std::io::prelude::*;
use std::process;

fn write_command(filename: &str, command: Command) -> Result<(), Box<Error> > {
    let mut file = File::create(filename)?;
    write!(file, "{}", command)?;
    Ok(())
}

fn main() {
    let start_time = PreciseTime::now();
    let max_time = Duration::milliseconds(MAX_TIME_MILLIS);
    
    let state = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };

    // TODO: Opening playbook?
    let command = strategy::monte_carlo::choose_move(&state, start_time, max_time);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            println!("Error while writing command file: {}", error);
            process::exit(1);
        }
    }

    println!("Elapsed time: {}", start_time.to(PreciseTime::now()));
}
