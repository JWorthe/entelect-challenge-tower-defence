use engine;
use engine::command::*;


pub fn choose_move(settings: &engine::settings::GameSettings, state: &engine::GameState) -> Command {
    if state.player.can_afford_defence_buildings(settings) {
        for y in 0..settings.size.y {
            if is_under_attack(state, y) {
                let p_options = state.unoccupied_player_cells_in_row(settings, y);
                if let Some(&p) = p_options.first() {
                    return Command::Build(p, BuildingType::Defense);
                }
            }
        }
    }

    if state.player.can_afford_all_buildings(settings) {
        let options = state.unoccupied_player_cells(settings);
        let option = options.first();
        let buildings = [BuildingType::Attack, BuildingType::Defense, BuildingType::Energy];
        let building = buildings.first();
        match (option, building) {
            (Some(&p), Some(&building)) => Command::Build(p, building),
            _ => Command::Nothing
        }
    }
    else {
        Command::Nothing
    }
}

fn is_under_attack(state: &engine::GameState, y: u8) -> bool {
    let attack = state.opponent_buildings.iter()
        .any(|b| b.pos.y == y && b.weapon_damage > 0);
    let defences = state.player_buildings.iter()
        .any(|b| b.pos.y == y && b.health > 5);
    attack && !defences
}
