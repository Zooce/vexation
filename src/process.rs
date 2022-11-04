use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::power::PowerBarEvent;
use crate::resources::*;

pub struct ProcessMovePlugin;

impl Plugin for ProcessMovePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(GameState::ProcessMove)
                .with_system(check_for_capture.before(check_for_winner))
                .with_system(process_index.before(check_for_winner))
                .with_system(check_for_winner)
            )
            .add_system_set(SystemSet::on_exit(GameState::ProcessMove)
                .with_system(clear_selected_marble)
            )
            ;
    }
}

fn check_for_capture(
    mut commands: Commands,
    current_player_data: Res<CurrentPlayerData>,
    selected_marble: Query<&Marble, (With<CurrentPlayer>, With<SelectedMarble>)>,
    mut opponent_marbles: Query<(Entity, &mut Marble, &Transform, &Player), Without<CurrentPlayer>>,
    mut power_bar_events: EventWriter<PowerBarEvent>,
) {
    let cur = selected_marble.single();

    // we don't capture in the home row
    if cur.index >= FIRST_HOME_INDEX && cur.index <= LAST_HOME_INDEX {
        return;
    }

    if let Some((opp_entity, mut opponent_marble, transform, opponent)) = opponent_marbles.iter_mut()
        // do not check opponent marbles in their home row or at their base
        .filter(|(_, opp, _, _)| opp.index < FIRST_HOME_INDEX || opp.index == CENTER_INDEX)
        // find an opponent marble at the same index as the marble just moved by the current player
        .find(|(_, opp, _, p)| Player::is_same_index(current_player_data.player, cur.index, **p, opp.index))
        // POWERUP: only include non-deflecting marbles
    {
        opponent_marble.index = BOARD.len();
        commands.entity(opp_entity).insert(Moving::new(opponent_marble.origin, transform.translation));
        power_bar_events.send(PowerBarEvent::Capture{ captor: current_player_data.player, captive: *opponent }); 
    }
}

// POWERUP: add deflection check system (essentially a reverse capture)

// POWERUP: add process index system - generates index power bar event
fn process_index(
    mut power_bar_events: EventWriter<PowerBarEvent>,
    current_player_data: Res<CurrentPlayerData>,
    selected_marble: Query<&Marble, (With<CurrentPlayer>, With<SelectedMarble>)>,
) {
    let marble = selected_marble.single();
    power_bar_events.send(PowerBarEvent::Index{
        player: current_player_data.player,
        index: marble.index,
        prev_index: marble.prev_index
    });
}

fn check_for_winner(
    mut state: ResMut<State<GameState>>,
    marbles: Query<&Marble, With<CurrentPlayer>>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if marbles.iter()
        .any(|m| !(FIRST_HOME_INDEX..=LAST_HOME_INDEX).contains(&m.index))
    {
        // not a winner
        state.set(GameState::TurnSetup).unwrap();
    } else {
        // winner
        println!("winner = {:?}", current_player_data.player);
        state.set(GameState::GameEnd).unwrap();
    }
}

fn clear_selected_marble(
    mut commands: Commands,
    selected_marble: Query<Entity, With<SelectedMarble>>,
) {
    let selected_marble = selected_marble.single();
    commands.entity(selected_marble).remove::<SelectedMarble>();
}

