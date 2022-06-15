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
    let dice_values = dice_data.get_dice_values();
    for (entity, marble) in marbles.iter() {
        for (value, which_die) in dice_values.iter() {
            // exit base / enter board - only one possible move for this marble
            if marble.index == BOARD.len() {
                if *value == 1 {
                    possible_moves.insert((entity, START_INDEX, *which_die)); // path = (BOARD.len(), START_INDEX)
                }
                continue;
            }

            // exit center space - only one possible move for this marble
            if marble.index == CENTER_INDEX {
                if *value == 1 {
                    possible_moves.insert((entity, CENTER_EXIT_INDEX, *which_die)); // path = (CENTER_INDEX, CENTER_EXIT_INDEX)
                }
                continue;
            }

            // basic move
            let next_index = marble.index + *value as usize;
            if next_index <= LAST_HOME_INDEX {
                possible_moves.insert((entity, next_index, *which_die)); // path = (marble.index..=next_index)
            }

            // enter center space
            if CENTER_ENTRANCE_INDEXES.contains(&(next_index - 1)) {
                possible_moves.insert((entity, CENTER_INDEX, *which_die)); // path = (marble.index..next_index) + (CENTER_INDEX)
            }
        }
    }

    println!("TurnSetup - calc_possible_moves: unfiltered = {:?}", possible_moves);

    // FIXME: there's a bug where we can't properly filter out a move that jumps over a marble (same color) and into the center index - we need to know the actual path for this case

    // filter out moves that violate the self-hop rules
    // - marbles of the same color cannot capture each other
    // - marbles of the same color cannot jump over each other
    possible_moves = possible_moves.into_iter().filter(|(entity, dest_index, _)| {
        let (_, move_marble) = marbles.get(*entity).unwrap();
        marbles.iter()
            // no need to compare the same marbles
            .filter(|(e, _)| *e != *entity)
            // look for a reason why this is not a valid move
            .find(|(_, other_marble)| {
                // not valid if there's already a marble at the destination OR
                other_marble.index == *dest_index ||
                // not valid if there's a marble on the way to the destination
                (move_marble.index < BOARD.len()
                    && *dest_index != CENTER_INDEX
                    && (move_marble.index..*dest_index).contains(&other_marble.index))
            }).is_none()
    }).collect();

    current_player_data.possible_moves = possible_moves;

    if current_player_data.possible_moves.is_empty() {
        state.set(GameState::NextPlayer).unwrap();
    } else if human_player.color == current_player_data.player {
        state.set(GameState::HumanIdle).unwrap();
    } else {
        state.set(GameState::ComputerTurn).unwrap();
    }

    println!("TurnSetup - calc_possible_moves: filtered = {:?}", current_player_data.possible_moves);
}
