use bevy::prelude::*;
use bevy::ecs::event::Events;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use rand::{Rng, thread_rng};

pub fn clear_animation_events(
    mut animation_events: ResMut<Events<MarbleAnimationDoneEvent>>,
) {
    animation_events.clear();
}

pub fn computer_choose_move(
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut highlight_events: EventWriter<HighlightEvent>,
    mut selection_data: ResMut<SelectionData>,
    mut computer_turn_timer: ResMut<ComputerTurnTimer>,
) {
    let mut rng = thread_rng();
    let move_index = rng.gen_range(0..current_player_data.possible_moves.len());
    current_player_data.selected_move_index = Some(move_index);
    let selected_marble = current_player_data.possible_moves[move_index].0;
    selection_data.marble = Some(selected_marble);
    highlight_events.send(HighlightEvent{
        data: Some((
            selected_marble,
            vec![current_player_data.possible_moves[move_index].1]
        ))
    });
    computer_turn_timer.0.reset();
    println!("computer selected move index {}", move_index);
}

pub fn computer_move_buffer(
    mut computer_turn_timer: ResMut<ComputerTurnTimer>,
    time: Res<Time>,
    mut commands: Commands,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut marbles: Query<(&Transform, &mut Marble), With<CurrentPlayer>>,
    mut dice_data: ResMut<DiceData>,
    mut state: ResMut<State<GameState>>,
) {
    if computer_turn_timer.0.tick(time.delta()).just_finished() {
        let (entity, index, which) = current_player_data.get_selected_move().unwrap();
        current_player_data.selected_move_index = None;
        let (transform, mut marble) = marbles.get_mut(entity).unwrap();
        let old_index = marble.index; // just for logging
        marble.index = index;
        dice_data.use_die(which);
        let destination = {
            let (c, r) = BOARD[index];
            let d = current_player_data.player.rotate_coords((c as f32, r as f32));
            Vec3::new(d.0 * TILE_SIZE, d.1 * TILE_SIZE, 1.0)
        };
        commands.entity(entity).insert(Moving::new(destination, transform.translation));
        state.set(GameState::WaitForAnimation).unwrap();
        println!("{:?}: from {} to {} with {:?}", entity, old_index, index, which);
    }
}
