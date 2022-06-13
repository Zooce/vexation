// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

/// Event to inform the selection system that it needs to highlight things.
pub struct SelectionEvent(pub Vec3);

pub fn handle_selection_events(
    mut commands: Commands,
    mut selection_events: EventReader<SelectionEvent>,
    selection_data: Res<SelectionData>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if let Some(selection) = selection_events.iter().last() {
        let marble = selection_data.marble.unwrap();

        let mut t = selection.0.clone();
        t.z += 1.0; // make sure it's drawn on top

        // create a sprite located at the same location as the marble entity
        commands.spawn_bundle(SpriteBundle{
            texture: selection_data.highlight_texture.clone(),
            transform: Transform::from_translation(t),
            ..default()
        })
        .insert(Highlight(marble))
        ;

        // create sprites located at the possible moves for the selected marble
        for board_index in current_player_data.get_moves(marble) {
            let tile = BOARD[board_index];
            let (x, y) = current_player_data.player.rotate((tile.0 as f32, tile.1 as f32));
            commands.spawn_bundle(SpriteBundle{
                texture: selection_data.highlight_texture.clone(),
                transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, t.z),
                ..default()
            })
            .insert(Highlight(marble))
            ;
        }
    }
}

/// Event to inform the selection system that it needs to remove highlights.
pub struct DeselectionEvent;

pub fn handle_deselection_events(
    mut commands: Commands,
    mut deselection_events: EventReader<DeselectionEvent>,
    entities: Query<Entity, With<Highlight>>,
    selection_data: Res<SelectionData>,
) {
    if deselection_events.iter().last().is_some() {
        match selection_data.marble {
            Some(marble) => entities.for_each(|e| if e != marble {
                commands.entity(e).despawn();
            }),
            None => entities.for_each(|e| commands.entity(e).despawn()),
        }
    }
}
