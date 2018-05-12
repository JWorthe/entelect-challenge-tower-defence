extern crate zombot;
use zombot::*;
use zombot::engine::command::Command;

use std::error::Error;

const STATE_PATH: &str = "state.json";

const COMMAND_PATH: &str = "command.txt";

use std::fs::File;
use std::io::prelude::*;
use std::process;

fn choose_move(settings: &engine::settings::GameSettings, state: &engine::GameState) -> Command {
    strategy::sample::choose_move(settings, state)
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
            eprintln!("Error while parsing JSON file: {}", error);
            process::exit(1);
        }
    };
    let command = choose_move(&settings, &state);

    match write_command(COMMAND_PATH, command) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Error while writing command file: {}", error);
            process::exit(1);
        }
    }
}
