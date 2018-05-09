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

mod json;
mod engine;
use engine::command::Command;

fn choose_move(settings: &engine::settings::GameSettings, state: &engine::GameState) -> Command {
    state.simulate(&settings, Command::Nothing, Command::Nothing);
    Command::Nothing
}


fn write_command(filename: &str, command: Command) -> Result<(), Box<Error> > {
    let mut file = File::create(filename)?;
    write!(file, "{}", command)?;

    Ok(())
}


fn main() {
    let (settings, state) = match json::read_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => {
            eprintln!("Failed to read the {} file. {}", STATE_PATH, error);
            process::exit(1);
        }
    };
    let command = choose_move(&settings, &state);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Failed to write the {} file. {}", COMMAND_PATH, error);
            process::exit(1);
        }
    }
}
