use bevy::prelude::*;
use crate::power::PowerDownEvent;
use crate::resources::{CurrentPlayerData, GameData, GameState, PowerDownType};

pub fn end_turn(
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut game_data: ResMut<GameData>,
    mut power_down_events: EventWriter<PowerDownEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_data = game_data.players.get_mut(&current_player_data.player).unwrap();
    if let Some(power_down) = player_data.end_of_turn() {
        match power_down {
            PowerDownType::Evading => power_down_events.send(PowerDownEvent::Evading(current_player_data.player)),
            PowerDownType::SelfJumping => power_down_events.send(PowerDownEvent::SelfJumping(current_player_data.player)),
            PowerDownType::EvadingAndSelfJumping => {
                power_down_events.send(PowerDownEvent::Evading(current_player_data.player));
                power_down_events.send(PowerDownEvent::SelfJumping(current_player_data.player));
            }
        }
    }
    current_player_data.clear();
    next_state.set(GameState::NextPlayer);
}
