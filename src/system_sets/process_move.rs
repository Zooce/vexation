use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn check_for_capture(
    mut commands: Commands,
    current_player_marbles: Query<&Marble, With<CurrentPlayer>>,
    mut opponent_marbles: Query<(Entity, &mut Marble, &Transform), Without<CurrentPlayer>>,
) {
    // TODO: marbles which aren't vulnerable don't need to be included in these queries
    for (entity, mut marble, transform) in opponent_marbles.iter_mut()
        .filter(|(_, opp, _)| { // only need the marble for filtering
            current_player_marbles.iter()
                .find(|cur| cur.index != BOARD.len() && cur.index == opp.index)
                .is_some()
        })
    {
        marble.index = BOARD.len();
        commands.entity(entity).insert(
            Moving::new(marble.origin, transform.translation)
        );
    }
    println!("ProcessMove - check_for_capture");
}

pub fn check_for_winner(
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
    marbles: Query<&Marble, With<CurrentPlayer>>,
) {
    if marbles.iter()
        .find(|m| !(FIRST_HOME_INDEX..=LAST_HOME_INDEX).contains(&m.index))
        .is_some()
    {
        // not a winner
        match (dice_data.die_1_side, dice_data.die_2_side) {
            (Some(_), None) | (None, Some(_)) => state.set(GameState::TurnSetup).unwrap(),
            (None, None) => state.set(GameState::NextPlayer).unwrap(),
            _ => unreachable!(),
        }
        println!("ProcessMove - check_for_winner: no winner yet");
    } else {
        // winner
        println!("ProcessMove - check_for_winner: game over!");
    }
}
