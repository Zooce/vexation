// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use super::ClickEvent;

pub fn check_marble_clicked(
    mut click_events: EventReader<ClickEvent>,
    marbles: Query<(Entity, &Transform), (With<CurrentPlayer>, With<Marble>)>,
    mut selection_data: ResMut<SelectionData>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some(click) = click_events.iter().last().or(selection_data.prev_click.as_ref()) {
        if let Some((entity, _)) = marbles.iter().find(|(_, t)| {
            click.x > t.translation.x - TILE_SIZE / 2. &&
            click.x < t.translation.x + TILE_SIZE / 2. &&
            click.y > t.translation.y - TILE_SIZE / 2. &&
            click.y < t.translation.y + TILE_SIZE / 2.
        }) {
            selection_data.marble = Some(entity);
            state.set(GameState::HumanMarbleSelected).unwrap();
            println!("check_marble_clicked: true");
        } else {
            println!("check_marble_clicked: false");
        }
    }
    // make sure the prev click is removed
    selection_data.prev_click = None;

}

pub fn highlight_selection(
    mut commands: Commands,
    selection_data: Res<SelectionData>,
    transforms: Query<&Transform, (With<CurrentPlayer>, With<Marble>)>,
    current_player_data: Res<CurrentPlayerData>,
) {
    let marble = selection_data.marble.unwrap();

    let mut t = transforms.get(marble).unwrap().translation.clone();
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

    println!("highlight_selection");
}
