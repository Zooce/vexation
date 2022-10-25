use bevy::prelude::*;
use bevy::ecs::event::Events;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use rand::{Rng, thread_rng};

pub fn clear_animation_events(
    mut animation_events: ResMut<Events<MarbleAnimationDoneEvent>>,
) {
    animation_events.clear();
}

pub fn computer_choose_move(
    mut commands: Commands,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut highlight_events: EventWriter<HighlightEvent>,
    mut computer_turn_timers: ResMut<ComputerTurnTimers>,
) {
    computer_turn_timers.reset();
    if current_player_data.possible_moves.is_empty() {
        return;
    }
    let mut rng = thread_rng();
    let move_index = rng.gen_range(0..current_player_data.possible_moves.len());
    current_player_data.select_move(move_index);
    let selected_marble = current_player_data.possible_moves[move_index].0;
    commands.entity(selected_marble).insert(SelectedMarble);
    highlight_events.send(HighlightEvent{ marble: Some(selected_marble), move_index: Some(move_index) });
}

pub fn computer_move_buffer(
    mut computer_turn_timers: ResMut<ComputerTurnTimers>,
    time: Res<Time>,
    mut commands: Commands,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut marbles: Query<(&Transform, &mut Marble), With<CurrentPlayer>>,
    mut dice_data: ResMut<DiceData>,
    mut state: ResMut<State<GameState>>,
    mut highlight_events: EventWriter<HighlightEvent>,
) {
    let no_moves = current_player_data.possible_moves.is_empty();
    if no_moves && dice_data.doubles && dice_data.did_use_die() {
        state.set(GameState::DiceRoll).unwrap();
        return;
    }

    let timer_finished = if no_moves && (dice_data.doubles || dice_data.did_use_die()) {
        computer_turn_timers.buffer_timer.tick(time.delta()).just_finished()
    } else {
        computer_turn_timers.move_timer.tick(time.delta()).just_finished()
    };
    if timer_finished {
        if let Some((entity, index, which)) = current_player_data.use_selected_move() {
            let (transform, mut marble) = marbles.get_mut(entity).unwrap();
            // TODO: need to share this logic with human player somehow..
            marble.update_index(index);
            dice_data.use_die(which, &mut commands);
            let destination = {
                let (c, r) = BOARD[index];
                let d = current_player_data.player.rotate_coords((c as f32, r as f32));
                Vec3::new(d.0 * TILE_SIZE, d.1 * TILE_SIZE, 1.0)
            };
            commands.entity(entity).insert(Moving::new(destination, transform.translation));
            highlight_events.send(HighlightEvent{ marble: None, move_index: None });
            state.set(GameState::WaitForAnimation).unwrap();
            println!("{:?}: from {} to {} with {:?}", entity, marble.prev_index, marble.index, which);
        } else if dice_data.doubles {
            state.set(GameState::DiceRoll).unwrap();
        } else {
            state.set(GameState::NextPlayer).unwrap();
        }
    }
}
