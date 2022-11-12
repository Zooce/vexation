use bevy::prelude::*;
use bevy::ecs::event::Events;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::shared_systems::HighlightEvent;
use rand::{Rng, thread_rng};
use rand::seq::IteratorRandom;

pub fn clear_animation_events(
    mut animation_events: ResMut<Events<MarbleAnimationDoneEvent>>,
) {
    animation_events.clear();
}

pub fn computer_choose_move(
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut highlight_events: EventWriter<HighlightEvent>,
    mut computer_turn_timers: ResMut<ComputerTurnTimers>,
) {
    computer_turn_timers.reset();
    if current_player_data.possible_moves.is_empty() {
        return;
    }
    let mut rng = thread_rng();
    let random_move = if let Some(entity) = current_player_data.selected_marble {
        let chosen = current_player_data.get_moves(entity).into_iter().choose(&mut rng).unwrap();
        (entity, chosen.0, chosen.1)
    } else {
        current_player_data.possible_moves[
            rng.gen_range(0..current_player_data.possible_moves.len())
        ]
    };
    current_player_data.select_move(random_move);
    highlight_events.send(HighlightEvent::On);
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
    // if the player rolled doubles we know they're going to roll again, but if
    // they used at least one of the dice for a move then we don't need to wait
    // for the buffer time to roll the dice again (we already waited when they
    // used their dice)
    let no_moves = current_player_data.possible_moves.is_empty();
    if no_moves && dice_data.dice.doubles && dice_data.dice.did_use_any() {
        state.set(GameState::DiceRoll).unwrap();
        return;
    }

    let timer_finished = if no_moves && (dice_data.dice.doubles || dice_data.dice.did_use_any()) {
        computer_turn_timers.buffer_timer.tick(time.delta()).just_finished()
    } else {
        computer_turn_timers.move_timer.tick(time.delta()).just_finished()
    };
    if timer_finished {
        if let Some((entity, index, which)) = current_player_data.get_selected_move() {
            let (transform, mut marble) = marbles.get_mut(entity).unwrap();
            marble.update_index(index);
            dice_data.use_die(which, &mut commands);
            let destination = {
                let (c, r) = BOARD[index];
                let (x, y) = current_player_data.player.rotate_coords((c as f32, r as f32));
                Vec3::new(x * TILE_SIZE, y * TILE_SIZE, 1.0)
            };
            commands.entity(entity).insert(Moving::new(destination, transform.translation));
            current_player_data.move_marble();
            highlight_events.send(HighlightEvent::Off);
            state.set(GameState::WaitForAnimation).unwrap();
        } else if dice_data.dice.doubles {
            state.set(GameState::DiceRoll).unwrap();
        } else {
            state.set(GameState::EndTurn).unwrap();
        }
    }
}
