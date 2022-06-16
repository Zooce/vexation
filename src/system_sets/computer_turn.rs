use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use rand::thread_rng;
use rand::seq::IteratorRandom;

pub fn choose_move(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    current_player_data: Res<CurrentPlayerData>,
    mut selection_data: ResMut<SelectionData>,
    mut marbles: Query<(&Transform, &mut Marble), With<CurrentPlayer>>,
    mut dice_data: ResMut<DiceData>,
    time: Res<Time>,
    mut timer: ResMut<ComputerTurnTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        timer.0.reset();
        let mut rng = thread_rng();
        let (entity, index, which) = current_player_data.possible_moves.iter().choose(&mut rng).unwrap();
        let (transform, mut marble) = marbles.get_mut(*entity).unwrap();
        let old_index = marble.index; // just for logging
        marble.index = *index;
        match which {
            WhichDie::One => dice_data.die_1_side = None,
            WhichDie::Two => dice_data.die_2_side = None,
            WhichDie::Both => {
                dice_data.die_1_side = None;
                dice_data.die_2_side = None;
            }
        }
        let destination = {
            let (c, r) = BOARD[*index];
            let d = current_player_data.player.rotate_coords((c as f32, r as f32));
            Vec3::new(d.0 * TILE_SIZE, d.1 * TILE_SIZE, 1.0)
        };
        selection_data.marble = Some(*entity);
        commands.entity(*entity).insert(Moving::new(destination, transform.translation));
        println!("Moving {:?} from {} to {} with {:?}", entity, old_index, index, which);
        state.set(GameState::ProcessMove).unwrap();
    }
}
