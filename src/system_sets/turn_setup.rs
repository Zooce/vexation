// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut selection_data: ResMut<SelectionData>,
    human_player: Res<HumanPlayer>,
    mut state: ResMut<State<GameState>>,
) {
    selection_data.marble = None; // TODO: do this in its own system ?
    let mut possible_moves = std::collections::BTreeSet::new(); // so we disregard duplicates
    for (entity, marble) in marbles.iter() {
        // exit base
        if marble.index == BOARD.len() {
            match (dice_data.die_1_side, dice_data.die_2_side) {
                (Some(1), Some(1)) => {
                    possible_moves.insert((entity, vec![START_INDEX], WhichDie::One));
                    possible_moves.insert((entity, vec![START_INDEX], WhichDie::Two));
                    possible_moves.insert((entity, vec![START_INDEX, START_INDEX + 1], WhichDie::Both));
                }
                (Some(1), Some(d2)) => {
                    possible_moves.insert((entity, vec![START_INDEX], WhichDie::One));
                    possible_moves.insert((entity, (START_INDEX..=START_INDEX + d2 as usize).collect(), WhichDie::Both));

                    // enter center - can only land on center using an exact roll with both dice
                    if CENTER_ENTRANCE_INDEXES.contains(&(START_INDEX + d2 as usize - 1)) {
                        let mut path: Vec<_> = (START_INDEX..=START_INDEX + d2 as usize - 1).collect();
                        path.push(CENTER_INDEX);
                        possible_moves.insert((entity, path, WhichDie::Both));
                    }
                }
                (Some(d1), Some(1)) => {
                    possible_moves.insert((entity, vec![START_INDEX], WhichDie::Two));
                    possible_moves.insert((entity, (START_INDEX..=START_INDEX + d1 as usize).collect(), WhichDie::Both));

                    // enter center - can only land on center using an exact roll with both dice
                    if CENTER_ENTRANCE_INDEXES.contains(&(START_INDEX + d1 as usize - 1)) {
                        let mut path: Vec<_> = (START_INDEX..=START_INDEX + d1 as usize - 1).collect();
                        path.push(CENTER_INDEX);
                        possible_moves.insert((entity, path, WhichDie::Both));
                    }
                }
                (Some(1), None) => { possible_moves.insert((entity, vec![START_INDEX], WhichDie::One)); }
                (None, Some(1)) => { possible_moves.insert((entity, vec![START_INDEX], WhichDie::Two)); }
                _ => {} // no exit
            }
            continue;
        }

        // exit center
        if marble.index == CENTER_INDEX {
            match (dice_data.die_1_side, dice_data.die_2_side) {
                (Some(1), Some(1)) => {
                    possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::One));
                    possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::Two));
                    possible_moves.insert((entity, vec![CENTER_EXIT_INDEX, CENTER_EXIT_INDEX + 1], WhichDie::Both));
                }
                (Some(1), Some(d2)) => {
                    possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::One));
                    possible_moves.insert((entity, (CENTER_EXIT_INDEX..=CENTER_EXIT_INDEX + d2 as usize).collect(), WhichDie::Both));
                }
                (Some(d1), Some(1)) => {
                    possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::Two));
                    possible_moves.insert((entity, (CENTER_EXIT_INDEX..=CENTER_EXIT_INDEX + d1 as usize).collect(), WhichDie::Both));
                }
                (Some(1), None) => { possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::One)); }
                (None, Some(1)) => { possible_moves.insert((entity, vec![CENTER_EXIT_INDEX], WhichDie::Two)); }
                _ => {} // no exit
            }
            continue;
        }

        // basic moves
        let mut basic_moves = std::collections::BTreeSet::new();
        match (dice_data.die_1_side, dice_data.die_2_side) {
            (Some(d1), Some(d2)) => {
                basic_moves.insert((entity, (marble.index + 1..=marble.index + d1 as usize).collect(), WhichDie::One));
                basic_moves.insert((entity, (marble.index + 1..=marble.index + d2 as usize).collect(), WhichDie::Two));
                basic_moves.insert((entity, (marble.index + 1..=marble.index + (d1 + d2) as usize).collect(), WhichDie::Both));

                // enter center - can only land on center using an exact roll with both dice
                if CENTER_ENTRANCE_INDEXES.contains(&(marble.index + (d1 + d2) as usize - 1)) {
                    let mut path: Vec<_> = (marble.index + 1..=marble.index + (d1 + d2) as usize - 1).collect();
                    path.push(CENTER_INDEX);
                    basic_moves.insert((entity, path, WhichDie::Both));
                }
            }
            (Some(d1), None) => {
                basic_moves.insert((entity, (marble.index + 1..=marble.index + (d1 as usize)).collect(), WhichDie::One));
            }
            (None, Some(d2)) => {
                basic_moves.insert((entity, (marble.index + 1..=marble.index + (d2 as usize)).collect(), WhichDie::Two));
            }
            _ => unreachable!(),
        }

        // can't move beyond the end of the home row
        possible_moves.append(&mut basic_moves.into_iter().filter(|(_, path, _)| {
            let src = *path.first().unwrap();
            let dst = *path.last().unwrap();
            dst <= LAST_HOME_INDEX // must be a valid board space
                || (dst == CENTER_INDEX // center row is okay as long as...
                    // ...we're coming from either the base or not the home row
                    && (src < FIRST_HOME_INDEX || src == START_INDEX))
        }).collect());
    }

    // filter out moves that violate the self-hop rules
    // - marbles of the same color cannot capture each other
    // - marbles of the same color cannot jump over each other
    current_player_data.possible_moves = possible_moves.into_iter()
        .filter_map(|(entity, path, which)| {
            let dst = *path.last().unwrap();
            if dst > LAST_HOME_INDEX && dst != CENTER_INDEX {
                return None; // can't go past the last home index, unless we're going into the center
            }
            match marbles.iter()
                // no need to compare the same marbles
                .filter(|(e, _)| *e != entity)
                // look for a same color marble along the path of this move
                .find(|(_, other_marble)| path.iter().find(|i| other_marble.index == **i).is_some())
            {
                Some(_) => None, // we found a marble along the path of this move, so it's no good
                None => Some((entity, dst, which))
            }
        })
        .collect();

    if current_player_data.possible_moves.is_empty() {
        state.set(GameState::NextPlayer).unwrap();
    } else if human_player.color == current_player_data.player {
        state.set(GameState::HumanIdle).unwrap();
    } else {
        state.set(GameState::ComputerTurn).unwrap();
    }
}
