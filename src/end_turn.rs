use bevy::prelude::*;
use crate::components::{CurrentPlayer, Evading};
use crate::resources::{CurrentPlayerData, GameData, GameState};

pub fn end_turn(
    mut commands: Commands,
    evading_marbles: Query<Entity, (With<Evading>, With<CurrentPlayer>)>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut game_data: ResMut<GameData>,
    mut state: ResMut<State<GameState>>, 
) {
    let player_data = game_data.players.get_mut(&current_player_data.player).unwrap();
    player_data.end_of_turn();
    if player_data.power_up_status.evade_capture_turns == 0 {
        evading_marbles.iter().for_each(|e| { commands.entity(e).remove::<Evading>(); });
    }
    current_player_data.clear();
    state.set(GameState::NextPlayer).unwrap();
}
