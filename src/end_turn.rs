use bevy::prelude::*;
use crate::resources::{CurrentPlayerData, GameData, GameState};

pub fn end_turn(
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut game_data: ResMut<GameData>,
    mut state: ResMut<State<GameState>>, 
) {
    game_data.players.get_mut(&current_player_data.player).unwrap().end_of_turn();
    current_player_data.selected_marble = None;
    current_player_data.selected_move = None;
    state.set(GameState::NextPlayer).unwrap();
}

