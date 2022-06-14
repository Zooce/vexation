// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use super::ClickEvent;

pub fn check_destination_clicked(
    mut commands: Commands,
    mut click_event_reader: EventReader<ClickEvent>,
    // mut click_event_writer: EventWriter<ClickEvent>,
    mut selection_data: ResMut<SelectionData>,
    current_player_data: Res<CurrentPlayerData>,
    mut marbles: Query<(&Transform, &mut Marble), With<CurrentPlayer>>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some(click) = click_event_reader.iter().last() {
        let marble = selection_data.marble.unwrap();
        let destination = Vec3::new(snap(click.x), snap(click.y), 1.0);
        let (col, row) = current_player_data.player.rotate((destination.x / TILE_SIZE, destination.y / TILE_SIZE));
        let dest_clicked = match BOARD.into_iter().position(|coord| coord == (col as i32, row as i32)) {
            Some(board_index) if current_player_data.get_moves(marble).contains(&board_index) => {
                let (t, mut m) = marbles.get_mut(marble).unwrap();
                m.index = board_index;
                commands.entity(marble).insert(Moving::new(destination, t.translation));
                // TODO: state.set(GameState::ProcessMove).unwrap();
                true
            }
            _ => false
        };
        if !dest_clicked {
            selection_data.marble = None;
            selection_data.prev_click = Some(click.clone());
            state.set(GameState::HumanIdle).unwrap();
        }
        println!("check_destination_clicked: {}", dest_clicked);
    }
}

pub fn remove_highlights(
    mut commands: Commands,
    entities: Query<Entity, With<Highlight>>,
) {
    entities.for_each(|e| commands.entity(e).despawn());
    println!("remove_highlights")
}

/// Snaps the given coordinate to the center of the tile it's inside of.
fn snap(coord: f32) -> f32 {
    // let's only deal with positive values for now
    let c = coord.abs();
    // how far away is the coordinate from the center of the tile
    let remainder = c % TILE_SIZE;
    let result = if remainder < TILE_SIZE / 2. {
        // if the coordinate is past the center (going away from the origin)
        // then snap it back to the center
        // |    X     |
        // |    <---c |
        c - remainder
    } else {
        // otherwise shift the coordinate to the next tile (going away from the
        // origin) then snap it back to the center
        // |    X    |
        // | c-------|->
        // |    <----|-c
        let shift = c + TILE_SIZE;
        shift - (shift % TILE_SIZE)
    };
    // just flip the result if the original coordinate was negative
    if coord < 0.0 && result > 0.0 {
        result * -1.0
    } else {
        result
    }
}
