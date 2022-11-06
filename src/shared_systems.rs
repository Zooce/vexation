use bevy::prelude::*;
use bevy::ecs::schedule::ShouldRun;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;

#[derive(SystemLabel)]
pub struct SharedSystemLabel;

pub fn should_run_shared_systems(
    state: Res<State<GameState>>,
) -> ShouldRun {
    match state.current() {
        GameState::MainMenu |
        GameState::GameStart |
        GameState::GameEnd |
        GameState::ChooseColor => ShouldRun::No,
        _ => ShouldRun::Yes,
    }
}

pub fn animate_marble_moves(
    mut commands: Commands,
    time: Res<Time>,
    mut moving_marbles: Query<(Entity, &Moving, &Player, &mut Transform)>,
    mut animation_done_events: EventWriter<MarbleAnimationDoneEvent>,
) {
    for (entity, moving, player, mut transform) in &mut moving_marbles {
        transform.translation.x += moving.direction.x * moving.speed * time.delta_seconds();
        transform.translation.y += moving.direction.y * moving.speed * time.delta_seconds();

        // we've arrived if the direction to the destination has flipped, meaning we've passed it
        let arrived = {
            let d = (moving.destination - transform.translation).normalize();
            Vec2::new(d.x, d.y).dot(moving.direction) < 0.0
        };

        if arrived {
            transform.translation = moving.destination; // just to make sure it's perfect
            commands.entity(entity).remove::<Moving>();
            animation_done_events.send(MarbleAnimationDoneEvent(*player));
        }
    }
}

/// This system spawns and/or despawns highlights based on the latest
/// [`HighlightEvent`].
pub fn highlighter(
    mut commands: Commands,
    marbles: Query<&Marble, With<CurrentPlayer>>, // we do this instead of using SelectedMarble because it may not have been set yet
    highlights: Query<(Entity, &Highlight)>,
    current_player_data: Res<CurrentPlayerData>,
    highlight_data: Res<HighlightData>,
    mut highlight_events: EventReader<HighlightEvent>,
) {
    if let Some(event) = highlight_events.iter().last() {
        match &event.marble {
            None => remove_all_highlights(commands, highlights),
            Some(selected_marble) => {
                let marble = marbles.get(*selected_marble).unwrap();
                let indexes = if let Some(index) = event.move_index {
                    vec![current_player_data.possible_moves[index].1]
                } else {
                    current_player_data.get_moves(*selected_marble)
                        .iter().map(|(index, _)| *index)
                        .collect()
                };

                // remove any "old" highlights
                highlights.iter()
                    .filter_map(|(e, h)| {
                        if h.marble != *selected_marble || !indexes.contains(&h.index) {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .for_each(|h| commands.entity(h).despawn());

                let rotated_transform_fn = |index| {
                    let tile: (i32, i32) = BOARD[index];
                    let (x, y) = current_player_data.player.rotate_coords((tile.0 as f32, tile.1 as f32));
                    Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 2.0)
                };
                // highlight the marble
                commands.spawn_bundle(SpriteBundle{
                    texture: highlight_data.marble_texture.clone(),
                    transform: if marble.index == BOARD.len() {
                        Transform::from_xyz(marble.origin.x, marble.origin.y, 2.0)
                    } else {
                        rotated_transform_fn(marble.index)
                    },
                    ..default()
                })
                .insert(Highlight{ marble: *selected_marble, index: marble.index })
                .insert(SelectedMarble);

                // highlight the move tiles
                for index in indexes {
                    let tile = BOARD[index];
                    let (x, y) = current_player_data.player.rotate_coords((tile.0 as f32, tile.1 as f32));
                    commands.spawn_bundle(SpriteBundle{
                        texture: highlight_data.tile_texture.clone(),
                        transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 2.0),
                        ..default()
                    })
                    .insert(Highlight{ marble: *selected_marble, index })
                    ;
                }
            }
        }
    }
}

pub fn animate_tile_highlights(
    time: Res<Time>,
    mut marble_transform: Query<&mut Transform, (With<Highlight>, Without<SelectedMarble>)>,
) {
    for mut transform in &mut marble_transform {
        transform.rotate(Quat::from_rotation_z(0.5 * time.delta_seconds()));
    }
}

/// This system removes all highlights and should be run as the exit system of
/// a "turn" state.
pub fn remove_all_highlights(
    mut commands: Commands,
    highlights: Query<(Entity, &Highlight)>,
) {
    highlights.for_each(|(e, _)| commands.entity(e).despawn());
}

pub fn wait_for_marble_animation(
    mut state: ResMut<State<GameState>>,
    mut animation_done_events: EventReader<MarbleAnimationDoneEvent>,
    current_player_data: Res<CurrentPlayerData>,
) {
    for done_event in animation_done_events.iter() {
        if done_event.0 == current_player_data.player {
            state.set(GameState::ProcessMove).unwrap();
        }
    }
}

// FIXME: there are cases where we basically flash this dimming (i.e. we undim immediately)
pub fn dim_used_die(
    mut dice_sprite_query: Query<&mut TextureAtlasSprite, Added<UsedDie>>,
) {
    for mut sprite in &mut dice_sprite_query {
        sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.4);
    }
}


