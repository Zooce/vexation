// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickEvent {
    pos: Vec2,
    active_marble: Option<Entity>,
}

/// This system listens for left mouse button presses, translates the location
/// of the clicks to window coordinates, and sends a custom [`ClickEvent`] for
/// highlight/destination systems to handle.
pub fn handle_mouse_clicks(
    windows: Res<Windows>,
    mut mouse_events: EventReader<MouseButtonInput>,
    mut click_events: EventWriter<ClickEvent>,
    mut highlight_events: EventWriter<HighlightEvent>,
    current_player_data: Res<CurrentPlayerData>,
    marbles: Query<(Entity, &Transform), (With<CurrentPlayer>, With<Marble>)>,
) {
    // we really only care about the most recent left mouse button press
    if let Some(_) = mouse_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed())
        .last()
    {
        if let Some(cursor) = windows.get_primary().unwrap().cursor_position() {
            // cursor position is measured from the bottom left corner, but transforms are measured from their center
            let (x, y) = (cursor.x - WINDOW_SIZE / 2., cursor.y - WINDOW_SIZE / 2.);

            let selected_marble = marbles.iter().find_map(|(e, t)| {
                let found = x > t.translation.x - TILE_SIZE / 2. &&
                            x < t.translation.x + TILE_SIZE / 2. &&
                            y > t.translation.y - TILE_SIZE / 2. &&
                            y < t.translation.y + TILE_SIZE / 2.;
                if found { Some(e) } else { None }
            });

            click_events.send(ClickEvent{ pos: Vec2::new(x, y), active_marble: selected_marble });
            highlight_events.send(HighlightEvent{
                data: match selected_marble {
                    Some(selected_marble) => {
                        let indexes = current_player_data.get_moves(selected_marble)
                            .iter().map(|(index, _)| *index)
                            .collect();
                        Some((selected_marble, indexes))
                    }
                    None => None
                }
            });
        }
    }
}

/// This system listens for our custom [`ClickEvents`] and checks to see if that
/// event corresponds to a destination for the currently selected marble.
pub fn destination_click_handler(
    mut commands: Commands,
    mut click_events: EventReader<ClickEvent>,
    current_player_data: Res<CurrentPlayerData>,
    mut marbles: Query<(&Transform, &mut Marble), With<CurrentPlayer>>,
    mut state: ResMut<State<GameState>>,
    mut dice_data: ResMut<DiceData>,
    mut selection_data: ResMut<SelectionData>,
) {
    if let Some(click) = click_events.iter().last() {
        match click.active_marble {
            None if selection_data.marble.is_some() => {
                let marble = selection_data.marble.unwrap();
                // to compare to board coordinates, we need to snap the click event to the center of a tile
                let (col, row) = (snap(click.pos.x), snap(click.pos.y));
                // find the move that corresponds to this click position
                let mv = match BOARD.into_iter().position(|(x, y)| {
                    // rotate the board coordinates based on the current player
                    let rot = current_player_data.player.rotate_coords((x as f32, y as f32));
                    rot == (col / TILE_SIZE, row / TILE_SIZE)
                }) {
                    Some(board_index) => current_player_data.get_moves(marble).into_iter().find(|(idx, _)| *idx == board_index),
                    _ => None,
                };
                if let Some((idx, which)) = mv {
                    let (t, mut m) = marbles.get_mut(marble).unwrap();
                    let old_index = m.index; // just for logging
                    m.index = idx;
                    dice_data.use_die(which);
                    commands.entity(marble).insert(Moving::new(Vec3::new(col, row, 1.0), t.translation));
                    state.set(GameState::WaitForAnimation).unwrap();
                    println!("{:?}: {} to {} with {:?}", marble, old_index, idx, which);
                }
            },
            Some(marble) => selection_data.marble = Some(marble),
            _ => {}
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
