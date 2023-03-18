use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::power::PowerEvent;
use crate::resources::*;

pub struct ProcessMovePlugin;

impl Plugin for ProcessMovePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems((check_for_capture, process_index, check_for_winner).chain()
                .in_set(OnUpdate(GameState::ProcessMove))
            )
            .add_system(process_complete.in_schedule(OnExit(GameState::ProcessMove)))
            ;
    }
}

fn check_for_capture(
    mut commands: Commands,
    current_player_data: Res<CurrentPlayerData>,
    current_player_marbles: Query<&Marble, With<CurrentPlayer>>,
    mut opponent_marbles: Query<(Entity, &mut Marble, &Transform, &Player), Without<CurrentPlayer>>,
    mut power_events: EventWriter<PowerEvent>,
) {
    let cur = current_player_marbles.get(current_player_data.moved_marble.unwrap()).unwrap();

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
        power_events.send(PowerEvent::Capture{ captor: current_player_data.player, captive: *opponent }); 
    }
}

// POWERUP: add deflection check system (essentially a reverse capture)

// POWERUP: add process index system - generates index power bar event
fn process_index(
    mut power_events: EventWriter<PowerEvent>,
    current_player_data: Res<CurrentPlayerData>,
    marbles: Query<&Marble, With<CurrentPlayer>>,
    game_data: Res<GameData>,
) {
    let player_data = game_data.players.get(&current_player_data.player).unwrap();
    if !player_data.power_up_status.home_run {
        let marble = marbles.get(current_player_data.moved_marble.unwrap()).unwrap();
        power_events.send(PowerEvent::Index{
            player: current_player_data.player,
            index: marble.index,
            prev_index: marble.prev_index
        });
    }
}

fn check_for_winner(
    mut next_state: ResMut<NextState<GameState>>,
    marbles: Query<&Marble, With<CurrentPlayer>>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if marbles.iter()
        .any(|m| !(FIRST_HOME_INDEX..=LAST_HOME_INDEX).contains(&m.index))
    {
        // not a winner
        next_state.set(GameState::TurnSetup);
    } else {
        // winner
        println!("winner = {:?}", current_player_data.player);
        next_state.set(GameState::GameEnd);
    }
}

fn process_complete(
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut game_data: ResMut<GameData>,
) {
    // clear the current player data so they don't have anything selected
    current_player_data.clear();

    // clear any used power ups that should only be used one time
    game_data.players.get_mut(&current_player_data.player).unwrap()
        .power_up_status.clear_one_shots();
}

