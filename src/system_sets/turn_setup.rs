// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    human_player: Res<HumanPlayer>,
    mut state: ResMut<State<GameState>>,
) {
    let mut possible_moves = std::collections::BTreeSet::new(); // so we disregard duplicates
    for (entity, marble) in marbles.iter() {
        for (value, which_die) in dice_data.get_dice_values() { // TODO: need to check if we've already used one or both dice in the previous move
            // exit base / enter board - only one possible move for this marble
            if marble.index == BOARD.len() {
                if value == 1 {
                    possible_moves.insert((entity, START_INDEX, which_die));
                }
                continue;
            }

            // exit center space - only one possible move for this marble
            if marble.index == CENTER_INDEX {
                if value == 1 {
                    possible_moves.insert((entity, CENTER_EXIT_INDEX, which_die));
                }
                continue;
            }

            // basic move
            let next_index = marble.index + value as usize;
            if next_index <= LAST_HOME_INDEX { // the very last home position
                possible_moves.insert((entity, next_index, which_die));
            }

            // enter center space
            if CENTER_ENTRANCE_INDEXES.contains(&(next_index - 1)) {
                possible_moves.insert((entity, CENTER_INDEX, which_die));
            }
        }
    }

    // remove possible moves that violate "self-hop" rule per marble
    for (marble, other_marble) in marbles.iter().zip(marbles.iter()) {
        if marble.0 == other_marble.0 || marble.1.index > other_marble.1.index {
            continue;
        }
        // remove possible moves where either:
        // - marble_a lands on marble_b
        // - marble_b is between marble_a's current position and the destination
        possible_moves = possible_moves.into_iter().filter(|(_, next_index, _)| {
            *next_index != other_marble.1.index && !(marble.1.index..*next_index).contains(&other_marble.1.index)
        }).collect();
    }

    current_player_data.possible_moves = possible_moves;

    if current_player_data.possible_moves.is_empty() {
        state.set(GameState::NextPlayer).unwrap();
    } else if human_player.color == current_player_data.player {
        state.set(GameState::HumanIdle).unwrap();
    } else {
        state.set(GameState::ComputerTurn).unwrap();
    }

    println!("calc_possible_moves");
}
