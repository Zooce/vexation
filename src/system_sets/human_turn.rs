// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;

pub fn translate_mouse_input(
    windows: Res<Windows>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut click_events: EventWriter<ClickEvent>,
) {
    if mouse_button_input_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed())
        .last().is_some()
    {
        if let Some(cursor) = windows.get_primary().unwrap().cursor_position() {
            let (x, y) = (cursor.x - WINDOW_SIZE / 2.0, cursor.y - WINDOW_SIZE / 2.0);
            // TODO: ignore this click if it's on a power-up button
            click_events.send(ClickEvent(Vec2::new(x, y)));
        }
    }
}

pub fn interpret_click_event(
    mut commands: Commands,
    mut highlight_events: EventWriter<HighlightEvent>,
    mut move_events: EventWriter<MoveEvent>,
    mut click_events: EventReader<ClickEvent>,
    current_player_data: Res<CurrentPlayerData>,
    marbles_query: Query<(Entity, &Transform), (With<Marble>, With<CurrentPlayer>)>,
    selected_marble: Query<Entity, (With<Marble>, With<SelectedMarble>)>,
) {
    if let Some(click_event) = click_events.iter().last() {
        println!("interpret click event: {:?}", click_event.0);
        // interpret click as marble selection
        if let Some(marble) = marbles_query.iter().find_map(|(e, t)| {
                let found = click_event.0.x > t.translation.x - TILE_SIZE / 2.0 &&
                            click_event.0.x < t.translation.x + TILE_SIZE / 2.0 &&
                            click_event.0.y > t.translation.y - TILE_SIZE / 2.0 &&
                            click_event.0.y < t.translation.y + TILE_SIZE / 2.0;
                if found { Some(e) } else { None }
            })
        {
            if let Ok(old_marble) = selected_marble.get_single() {
                if old_marble != marble {
                    println!("clicked different marble");
                    commands.entity(old_marble).remove::<SelectedMarble>();
                } else {
                    println!("clicked same marble");
                    return; // ignore clicks on a marble that is already selected
                }
            } else {
                println!("clicked new marble");
            }
            commands.entity(marble).insert(SelectedMarble);
            highlight_events.send(HighlightEvent{ marble: Some(marble), move_index: None });
        }
        // interpret click as move selection
        else if let Ok(marble) = selected_marble.get_single() {
            // to compare to board coordinates, we need to snap the click event to the center of a tile
            let (col, row) = (snap(click_event.0.x), snap(click_event.0.y));
            // find the move that corresponds to this click position
            let selected_move = match BOARD.into_iter().position(|(x, y)| {
                // rotate the board coordinates based on the current player
                let rot = current_player_data.player.rotate_coords((x as f32, y as f32));
                // find the board index that matches the click position
                rot == (col / TILE_SIZE, row / TILE_SIZE)
            }) {
                // find a move for this board index
                Some(clicked_board_index) => current_player_data
                    .get_moves(marble).into_iter().find(|(idx, _)| *idx == clicked_board_index),
                _ => None,
            };
            if let Some((idx, which)) = selected_move {
                println!("clicked destination");
                move_events.send(MoveEvent((idx, which, Vec3::new(col, row, 1.0))));
            } else {
                println!("clicked empty space");
                commands.entity(marble).remove::<SelectedMarble>();
            }

            // since we didn't click on another marble, we need all highlights to be removed
            highlight_events.send(HighlightEvent{ marble: None, move_index: None });
        }
    }
}

pub fn move_event_handler(
    mut commands: Commands,
    mut move_events: EventReader<MoveEvent>,
    mut marbles: Query<(Entity, &Transform, &mut Marble), With<SelectedMarble>>,
    mut dice_data: ResMut<DiceData>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some(MoveEvent((idx, which, dest))) = move_events.iter().last() {
        let (e, t, mut m) = marbles.single_mut();
        let old_index = m.index; // just for logging
        m.index = *idx;
        dice_data.use_die(*which);
        commands.entity(e).insert(Moving::new(*dest, t.translation));
        state.set(GameState::WaitForAnimation).unwrap();
        // TODO: if `idx` is also a power-up tile for the current player, initiate the power-up generator
        println!("{:?}: {} to {} with {:?}", e, old_index, idx, which);
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
