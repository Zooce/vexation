use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct SharedSystemSet;

pub fn should_run_shared_systems(
    state: Res<State<GameState>>,
) -> bool {
    match state.get() {
        GameState::MainMenu |
        GameState::GameStart |
        GameState::GameEnd |
        GameState::ChooseColor => false,
        _ => true,
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

#[derive(Debug, Event)]
pub enum HighlightEvent {
    On,
    Off,
}

/// This system spawns and/or despawns highlights based on the latest
/// [`HighlightEvent`].
pub fn highlighter(
    mut commands: Commands,
    marbles: Query<&Marble, With<CurrentPlayer>>,
    mut highlights: Query<(Entity, &mut Highlight, Option<&SelectedMarble>)>,
    current_player_data: Res<CurrentPlayerData>,
    highlight_data: Res<HighlightData>,
    mut highlight_events: EventReader<HighlightEvent>,
) {
    if let Some(event) = highlight_events.iter().last() {
        match event {
            HighlightEvent::Off => highlights.for_each(|(e, _, _)| commands.entity(e).despawn()),
            HighlightEvent::On => {
                let entity = current_player_data.selected_marble.unwrap();
                // get the move indexes for the selected marble so we can highlight them
                // -- the computer player would have already selected a move, so we can just highlight that index 
                let indexes = if let Some(MarbleMove{ destination, .. }) = current_player_data.selected_move {
                    vec![destination]
                } else {
                    current_player_data.get_moves(entity)
                        .iter().map(|MarbleMove{ destination, .. }| *destination)
                        .collect()
                };

                // remove any "old" highlights
                highlights.iter()
                    .filter_map(|(e, h, _)| {
                        if !indexes.contains(&h.index) || (h.index == BOARD.len() && h.marble != entity) {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .for_each(|e| commands.entity(e).despawn());

                let rotated_transform_fn = |index| {
                    let tile: (i32, i32) = BOARD[index];
                    let (x, y) = current_player_data.player.rotate_coords((tile.0 as f32, tile.1 as f32));
                    Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, Z_SELECTION_HIGHLIGHT)
                };

                // highlight the marble if it's not already highlighted
                if highlights.iter().find(|(e, _, m)| m.is_some() && *e == entity).is_none() {
                    let marble = marbles.get(entity).unwrap();
                    commands.spawn((
                        SpriteBundle{
                            texture: highlight_data.marble_texture.clone(),
                            transform: if marble.index == BOARD.len() {
                                Transform::from_xyz(marble.origin.x, marble.origin.y, Z_SELECTION_HIGHLIGHT)
                            } else {
                                rotated_transform_fn(marble.index)
                            },
                            ..default()
                        },
                        Highlight{ marble: entity, index: marble.index },
                        SelectedMarble,
                    ));
                }

                // set the marble entity for all highlights
                highlights.iter_mut().for_each(|(_, mut h, _)| { h.marble = entity; });

                // highlight indexes that are not already highlighted
                indexes.iter()
                    .filter(|&&i| highlights.iter().find(|(_, h, _)| h.index == i).is_none())
                    .for_each(|&i| {
                        commands.spawn((
                            SpriteBundle{
                                texture: highlight_data.tile_texture.clone(),
                                transform: rotated_transform_fn(i),
                                ..default()
                            },
                            Highlight{ marble: entity, index: i },
                        ));
                    });
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

pub fn wait_for_marble_animation(
    mut next_state: ResMut<NextState<GameState>>,
    mut animation_done_events: EventReader<MarbleAnimationDoneEvent>,
    current_player_data: Res<CurrentPlayerData>,
) {
    for done_event in animation_done_events.iter() {
        if done_event.0 == current_player_data.player {
            next_state.set(GameState::ProcessMove);
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

