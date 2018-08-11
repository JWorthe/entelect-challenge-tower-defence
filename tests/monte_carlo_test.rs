extern crate zombot;
extern crate time;
use time::{PreciseTime, Duration};

use zombot::*;

const STATE_PATH: &str = "tests/state0.json";

// there are assertions in the game engine, run when it's in debug mode
#[test]
fn it_does_a_normal_turn_successfully() {
    let start_time = PreciseTime::now();
    let state = match input::json::read_bitwise_state_from_file(STATE_PATH) {
        Ok(ok) => ok,
        Err(error) => panic!("Error while parsing JSON file: {}", error)
    };
    let max_time = Duration::milliseconds(200);
    strategy::monte_carlo::choose_move(&state, start_time, max_time);

    assert!(start_time.to(PreciseTime::now()) < max_time + Duration::milliseconds(50))
}
