use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::events::GeneratePowerUpEvent;
use crate::resources::*;

pub fn check_for_capture(
    mut commands: Commands,
    current_player_data: Res<CurrentPlayerData>,
    selected_marble: Query<(Entity, &Marble), (With<CurrentPlayer>, With<SelectedMarble>)>,
    mut opponent_marbles: Query<(Entity, &mut Marble, &Transform, &Player), Without<CurrentPlayer>>,
) {
    // TODO: the `e` is only here for logging - remove this later
    let (e, cur) = selected_marble.single();

    // we don't capture in the home row
    if cur.index >= FIRST_HOME_INDEX && cur.index <= LAST_HOME_INDEX {
        return;
    }

    if let Some((entity, mut opponent_marble, transform, opponent)) = opponent_marbles.iter_mut()
        // do not check opponent marbles in their home row or at their base
        .filter(|(_, opp, _, _)| opp.index < FIRST_HOME_INDEX || opp.index == CENTER_INDEX)
        // find an opponent marble at the same index as the marble just moved by the current player
        .find(|(_, opp, _, p)| Player::is_same_index(current_player_data.player, cur.index, **p, opp.index))
        // POWERUP: only include non-deflecting marbles
    {
        println!("{:?} {:?} @ {} captured {:?} {:?} @ {}",
            current_player_data.player, e, cur.index,
            opponent, entity, opponent_marble.index
        );
        opponent_marble.index = BOARD.len();
        commands.entity(entity).insert(Moving::new(opponent_marble.origin, transform.translation));
    }
}

pub fn check_for_power_up(
    current_player_data: Res<CurrentPlayerData>,
    selected_marble: Query<&Marble, With<SelectedMarble>>,
    mut generate_power_up_events: EventWriter<GeneratePowerUpEvent>,
) {
    let marble = selected_marble.single();
    if POWER_UP_INDEXES.contains(&marble.index) {
        println!("generating power-up event for {:?}", current_player_data.player);
        generate_power_up_events.send(GeneratePowerUpEvent(current_player_data.player));
    }
}

pub fn check_for_winner(
    mut state: ResMut<State<GameState>>,
    marbles: Query<&Marble, With<CurrentPlayer>>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if marbles.iter()
        .find(|m| !(FIRST_HOME_INDEX..=LAST_HOME_INDEX).contains(&m.index))
        .is_some()
    {
        // not a winner
        state.set(GameState::TurnSetup).unwrap();
    } else {
        // winner
        println!("{:?} Wins!", current_player_data.player);
        state.set(GameState::GameEnd).unwrap();
    }
}

pub fn clear_selected_marble(
    mut commands: Commands,
    selected_marble: Query<Entity, With<SelectedMarble>>,
) {
    let selected_marble = selected_marble.single();
    commands.entity(selected_marble).remove::<SelectedMarble>();
}
