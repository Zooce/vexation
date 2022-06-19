// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use std::collections::BTreeSet;

pub fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut selection_data: ResMut<SelectionData>,
) {
    selection_data.marble = None; // TODO: do this in its own system ?
    let mut possible_moves = BTreeSet::new(); // so we disregard duplicates
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
        let mut basic_moves = basic((dice_data.die_1_side, dice_data.die_2_side), entity, marble);
        possible_moves.append(&mut basic_moves);
    }

    // filter out moves that violate the self-hop rules
    // - marbles of the same color cannot capture each other
    // - marbles of the same color cannot jump over each other
    current_player_data.possible_moves = possible_moves.into_iter()
        .filter_map(|(entity, path, which)| {
            match marbles.iter()
                // no need to compare the same marbles
                .filter(|(e, _)| *e != entity)
                // look for a same color marble along the path of this move
                .find(|(_, other_marble)| path.iter().find(|i| other_marble.index == **i).is_some())
            {
                Some(_) => None, // we found a marble along the path of this move, so it's no good
                None => Some((entity, *path.last().unwrap(), which))
            }
        })
        .collect();
}

pub fn buffer_timer(
    time: Res<Time>,
    mut timer: ResMut<BufferTimer>,
    current_player_data: ResMut<CurrentPlayerData>,
    mut state: ResMut<State<GameState>>,
    human_player: Res<HumanPlayer>,
    dice_data: Res<DiceData>,
) {
    if current_player_data.possible_moves.is_empty() {
        let used_a_die = dice_data.die_1_side.is_none() || dice_data.die_2_side.is_none();
        if used_a_die || timer.0.tick(time.delta()).just_finished() {
            timer.0.reset();
            if dice_data.doubles {
                state.set(GameState::DiceRoll).unwrap();
            } else {
                state.set(GameState::NextPlayer).unwrap();
            }
        }
    } else if human_player.color == current_player_data.player {
        state.set(GameState::HumanIdle).unwrap();
    } else {
        state.set(GameState::ComputerTurn).unwrap();
    }
}

fn basic(
    dice: (Option<u8>, Option<u8>),
    entity: Entity,
    marble: &Marble
) -> BTreeSet<(Entity, Vec<usize>, WhichDie)>
{
    let mut basic_moves = BTreeSet::new();
    match dice {
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
    basic_moves.into_iter().filter(|(_, path, _)| {
        let dst = *path.last().unwrap();
        dst <= LAST_HOME_INDEX // must be a valid board space
            || (dst == CENTER_INDEX // center row is okay as long as...
                // ...the marble was not at the end of the home row (this means the path will only be [CENTER_INDEX]) AND...
                && marble.index != LAST_HOME_INDEX
                // ...the path doesn't go through the home row
                && path.iter().find(|i| **i >= FIRST_HOME_INDEX && **i <= LAST_HOME_INDEX).is_none())
    }).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_moves() {
        let dice = (Some(5), Some(5));
        let marble = Marble{ index: 43, origin: Vec3::ZERO };
        let res = basic(dice, Entity::from_raw(12), &marble);
        let mut iter = res.iter();
        assert_eq!(2, res.len());
        assert_eq!(vec![44, 45, 46, 47, 48], iter.next().unwrap().1);
        assert_eq!(vec![44, 45, 46, 47, 48], iter.next().unwrap().1);

        let dice = (Some(4), Some(1));
        let marble = Marble{ index: 52, origin: Vec3::ZERO };
        let res = basic(dice, Entity::from_raw(13), &marble);
        assert_eq!(0, res.len());
    }
}
