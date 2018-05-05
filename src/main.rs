extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::error::Error;

const STATE_PATH: &str = "state.json";

const COMMAND_PATH: &str = "command.txt";

use std::fs::File;
use std::io::prelude::*;
use std::process;

mod state_json;
mod engine;
use engine::command::Command;

fn choose_move(_state: &state_json::State) -> Option<Command> {
    None
}


fn write_command(filename: &str, command: Option<Command>) -> Result<(), Box<Error> > {
    let mut file = File::create(filename)?;
    if let Some(command) = command {
        write!(file, "{}", command)?;
    }

    Ok(())
}


fn main() {
    let state = match state_json::read_state_from_file(STATE_PATH) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("Failed to read the {} file. {}", STATE_PATH, error);
            process::exit(1);
        }
    };
    let command = choose_move(&state);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Failed to write the {} file. {}", COMMAND_PATH, error);
            process::exit(1);
        }
    }
}
