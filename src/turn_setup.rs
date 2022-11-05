use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use std::collections::BTreeSet;

pub fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
) {
    let mut possible_moves = BTreeSet::new(); // so we disregard duplicates
    let dice = dice_data.sides();
    if dice == (None, None) {
        current_player_data.possible_moves = vec![]; // no moves
        return;
    }
    for (entity, marble) in &marbles {
        // exit base
        if marble.index == BOARD.len() {
            base_exit_rules(dice, entity, &mut possible_moves);
            continue;
        }

        // exit center
        if marble.index == CENTER_INDEX {
            center_exit_rules(dice, entity, &mut possible_moves);
            continue;
        }

        // basic moves
        basic_rules(dice, entity, marble, &mut possible_moves);
    }

    // filter out moves that violate the self-hop rules
    // - marbles of the same color cannot capture each other
    // - marbles of the same color cannot jump over each other
    // POWERUP: filter out moves that land on opponents who are currently "evading"
    current_player_data.possible_moves = possible_moves.into_iter()
        .filter_map(|(entity, path, which)| {
            match marbles.iter()
                // no need to compare the same marbles
                .filter(|(e, _)| *e != entity)
                // look for a same color marble along the path of this move
                // POWERUP: ignore this check if "self jump" power-up is currently enabled
                .find(|(_, other_marble)| path.iter().any(|i| other_marble.index == *i))
            {
                Some(_) => None, // we found a marble along the path of this move, so it's no good
                None => Some((entity, *path.last().unwrap(), which))
            }
        })
        .collect();
}

pub fn count_moves(
    mut game_data: ResMut<GameData>,
    current_player_data: Res<CurrentPlayerData>,
) {
    let count = current_player_data.possible_moves.len() as u8;
    game_data.players.get_mut(&current_player_data.player).unwrap().turn_move_count += count;
}

pub fn turn_setup_complete(
    mut state: ResMut<State<GameState>>,
    human_player: Res<HumanPlayer>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if human_player.color == current_player_data.player {
        state.set(GameState::HumanTurn).unwrap();
    } else {
        state.set(GameState::ComputerTurn).unwrap();
    }
}

/// Calculates the path from a starting index into the center index. This will
/// return `None` if the end index is not one index past a center entrance
/// index. If a path is returned it requires the use of both dice (i.e. a marble
/// can only land on the center space using an exact roll with both dice).
fn enter_center_path(start: usize, end: usize) -> Option<Vec<usize>> {
    if CENTER_ENTRANCE_INDEXES.contains(&(end - 1)) {
        let mut path: Vec<_> = (start..=end - 1).collect();
        path.push(CENTER_INDEX);
        Some(path)
    } else {
        None
    }
}

fn base_exit_rules(
    dice: (Option<u8>, Option<u8>),
    entity: Entity,
    possible_moves: &mut BTreeSet<(Entity, Vec<usize>, WhichDie)>,
) {
    if dice.0 == Some(1) || dice.0 == Some(6) {
        possible_moves.insert((entity, vec![START_INDEX], WhichDie::One)); // exit with die 1...
        if dice.1.is_some() {
            let dest = START_INDEX + dice.1.unwrap() as usize;
            possible_moves.insert((entity, (START_INDEX..=dest).collect(), WhichDie::Both)); // ...then move with die 2...or...
            if let Some(center_path) = enter_center_path(START_INDEX, dest) {
                possible_moves.insert((entity, center_path, WhichDie::Both)); // ...move to center with die 2
            }
        }
    }
    if dice.1 == Some(1) || dice.1 == Some(6) {
        possible_moves.insert((entity, vec![START_INDEX], WhichDie::Two)); // exit with die 2...
        if dice.0.is_some() {
            let dest = START_INDEX + dice.0.unwrap() as usize;
            possible_moves.insert((entity, (START_INDEX..=dest).collect(), WhichDie::Both)); // ...then move with die 1...or...
            if let Some(center_path) = enter_center_path(START_INDEX, dest) {
                possible_moves.insert((entity, center_path, WhichDie::Both)); //...move to center with die 1
            }
        }
    }
}

fn center_exit_rules(
    dice: (Option<u8>, Option<u8>),
    entity: Entity,
    possible_moves: &mut BTreeSet<(Entity, Vec<usize>, WhichDie)>,
) {
    match dice {
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
}

fn basic_rules(
    dice: (Option<u8>, Option<u8>),
    entity: Entity,
    marble: &Marble,
    possible_moves: &mut BTreeSet<(Entity, Vec<usize>, WhichDie)>,
) {
    let mut basic_moves = BTreeSet::new();
    match dice {
        (Some(d1), Some(d2)) => {
            basic_moves.insert((entity, (marble.index + 1..=marble.index + d1 as usize).collect(), WhichDie::One));
            basic_moves.insert((entity, (marble.index + 1..=marble.index + d2 as usize).collect(), WhichDie::Two));
            basic_moves.insert((entity, (marble.index + 1..=marble.index + (d1 + d2) as usize).collect(), WhichDie::Both));

            if let Some(center_path) = enter_center_path(marble.index, marble.index + (d1 + d2) as usize) {
                basic_moves.insert((entity, center_path, WhichDie::Both));
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

    // filter out moves that don't make sense
    basic_moves = basic_moves.into_iter().filter(|(_, path, _)| {
        let dest = *path.last().unwrap();
        dest <= LAST_HOME_INDEX // destination must be a valid board space
            || (dest == CENTER_INDEX // the center space is okay as long as...
                // ...the marble was not at the end of the home row (this means the path will only be [CENTER_INDEX]) AND...
                && marble.index != LAST_HOME_INDEX
                // ...the path doesn't go through the home row
                && !path.iter().any(|i| *i >= FIRST_HOME_INDEX && *i <= LAST_HOME_INDEX))
    }).collect();

    possible_moves.append(&mut basic_moves);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_base_exit_moves() {
        let dice = (Some(1), Some(6));
        let mut moves = BTreeSet::new();
        base_exit_rules(dice, Entity::from_raw(12), &mut moves);
        let mut iter = moves.iter();
        assert_eq!(5, iter.len());
        assert_eq!(vec![0], iter.next().unwrap().1); // use die 1 to exit
        assert_eq!(vec![0], iter.next().unwrap().1); // use die 2 to exit
        assert_eq!(vec![0, 1], iter.next().unwrap().1); // use die 2 to exit then die 1 to move
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6], iter.next().unwrap().1); // use die 1 to exit then die 2 to move
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 53], iter.next().unwrap().1); // use die 1 to exit then die 2 to move to center
    }

    #[test]
    fn test_center_exit_moves() {
        let dice = (Some(1), Some(4));
        let mut moves = BTreeSet::new();
        center_exit_rules(dice, Entity::from_raw(12), &mut moves);
        let mut iter = moves.iter();
        assert_eq!(2, iter.len());
        assert_eq!(vec![41], iter.next().unwrap().1); // use die 1 to exit
        assert_eq!(vec![41, 42, 43, 44, 45], iter.next().unwrap().1); // use die 1 to exit then die 2 to move
    }

    #[test]
    fn test_basic_moves() {
        let dice = (Some(5), Some(5));
        let marble = Marble{ index: 43, prev_index: 42, origin: Vec3::ZERO };
        let mut moves = BTreeSet::new();
        basic_rules(dice, Entity::from_raw(12), &marble, &mut moves);
        let mut iter = moves.iter();
        assert_eq!(2, moves.len());
        assert_eq!(vec![44, 45, 46, 47, 48], iter.next().unwrap().1);
        assert_eq!(vec![44, 45, 46, 47, 48], iter.next().unwrap().1);

        let dice = (Some(4), Some(1));
        let marble = Marble{ index: 52, prev_index: 52, origin: Vec3::ZERO };
        moves = BTreeSet::new();
        basic_rules(dice, Entity::from_raw(13), &marble, &mut moves);
        assert_eq!(0, moves.len());
    }
}
