extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;
use zombot::engine::command::Command;

use std::error::Error;

const STATE_PATH: &str = "state.json";

const COMMAND_PATH: &str = "command.txt";

use std::fs::File;
use std::io::prelude::*;
use std::process;

fn choose_move<GS:engine::GameState>(settings: &engine::settings::GameSettings, state: &GS, start_time: &PreciseTime) -> Command {
    #[cfg(not(feature = "reduced-time"))]
    #[cfg(not(feature = "extended-time"))]
    let max_time = Duration::milliseconds(1950);
    
    #[cfg(feature = "reduced-time")]
    let max_time = Duration::milliseconds(950);

    #[cfg(feature = "extended-time")]
    let max_time = Duration::milliseconds(19950);
    
    strategy::monte_carlo::choose_move(settings, state, start_time, max_time)
}


fn write_command(filename: &str, command: Command) -> Result<(), Box<Error> > {
    let mut file = File::create(filename)?;
    write!(file, "{}", command)?;
    Ok(())
}


fn main() {
    let start_time = PreciseTime::now();
    
    let (settings, state) = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            println!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let command = choose_move(&settings, &state, &start_time);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            println!("Error while writing command file: {}", error);
            process::exit(1);
        }
    }

    println!("Elapsed time: {}", start_time.to(PreciseTime::now()));
}
