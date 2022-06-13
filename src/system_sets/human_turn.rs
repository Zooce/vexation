// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use bevy::input::keyboard::KeyboardInput;
use crate::constants::*;
use crate::components::*;
use crate::events::*;
use crate::resources::*;

pub fn handle_mouse_clicks(
    mut commands: Commands,
    mut mouse_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    mut marbles: Query<(Entity, &Transform, &mut Marble), With<CurrentPlayer>>,
    mut selection_events: EventWriter<SelectionEvent>,
    mut deselection_events: EventWriter<DeselectionEvent>,
    mut selection_data: ResMut<SelectionData>,
    current_player_data: Res<CurrentPlayerData>,
) {
    // we need the current position of the cursor or else we don't really care
    let cursor = match windows.get_primary() {
        Some(w) => match w.cursor_position() {
            Some(c) => c,
            None => return,
        }
        None => return,
    };

    // we really only care about the most recent left mouse button press
    if let Some(_) = mouse_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed()).last()
    {
        // cursor position is measured from the bottom left corner, but transforms are measured from their center
        let (cursor_x, cursor_y) = (cursor.x - WINDOW_SIZE / 2., cursor.y - WINDOW_SIZE / 2.);

        // check for marble click
        if let Some((entity, transform, _)) = marbles.iter().find(|(_, t, _)| {
            cursor_x > t.translation.x - TILE_SIZE / 2. &&
            cursor_x < t.translation.x + TILE_SIZE / 2. &&
            cursor_y > t.translation.y - TILE_SIZE / 2. &&
            cursor_y < t.translation.y + TILE_SIZE / 2.
        }) {
            selection_data.marble = Some(entity);
            selection_events.send(SelectionEvent(transform.translation.clone()));
        } else {
            // check for marble destination click
            if let Some(marble) = selection_data.marble {
                let destination = Vec3::new(snap(cursor_x), snap(cursor_y), 1.0);
                let (col, row) = current_player_data.player.rotate((destination.x / TILE_SIZE, destination.y / TILE_SIZE));
                if let Some(board_index) = BOARD.into_iter().position(|coord| coord == (col as i32, row as i32)) {
                    if current_player_data.get_moves(marble).contains(&board_index) {
                        let (_, transform, mut m) = marbles.get_mut(marble).unwrap();
                        m.index = board_index;
                        commands.entity(marble).insert(Moving::new(destination, transform.translation));
                    }
                }
            }
            selection_data.marble = None;
            deselection_events.send(DeselectionEvent);
        }
    }
}

pub fn handle_keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut deselection_events: EventWriter<DeselectionEvent>,
    mut selection_data: ResMut<SelectionData>,
) {
    for event in keyboard_events.iter() {
        match event.key_code {
            Some(KeyCode::Escape) => {
                selection_data.marble = None;
                deselection_events.send(DeselectionEvent);
            }
            _ => return,
        }
    }
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
