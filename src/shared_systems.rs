// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::ecs::schedule::ShouldRun;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn should_run_shared_systems(
    state: Res<State<GameState>>,
) -> ShouldRun {
    match state.current() {
        GameState::MainMenu |
        GameState::GameStart |
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
    const PIXELS_PER_SEC: f32 = 750.;
    for (entity, moving, player, mut transform) in moving_marbles.iter_mut() {
        transform.translation.x += moving.direction.x * PIXELS_PER_SEC * time.delta_seconds();
        transform.translation.y += moving.direction.y * PIXELS_PER_SEC * time.delta_seconds();

        // we've arrived if the direction to the destination has flipped, meaning we've passed it
        let arrived = {
            let d = (moving.destination - transform.translation).normalize();
            Vec2::new(d.x, d.y).dot(moving.direction) < 0.0
        };

        if arrived {
            println!("{:?} animation complete", entity);
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
    marbles: Query<&Marble, With<CurrentPlayer>>,
    highlights: Query<(Entity, &Highlight)>,
    current_player_data: Res<CurrentPlayerData>,
    highlight_data: Res<HighlightData>,
    mut highlight_events: EventReader<HighlightEvent>,
) {
    if let Some(event) = highlight_events.iter().last() {
        match &event.data {
            None => remove_all_highlights(commands, highlights),
            Some((selected_marble, indexes)) => {
                let marble = marbles.get(*selected_marble).unwrap();

                // remove any "old" highlights
                let old_highlights = highlights.iter()
                    .filter_map(|(e, h)| {
                        if h.marble != *selected_marble
                            || (h.index != marble.index && !indexes.contains(&h.index))
                        {
                            Some(e)
                        } else {
                            None
                        }
                    });
                for highlight in old_highlights {
                    println!("remvoing old highlight");
                    commands.entity(highlight).despawn();
                }

                println!("highlighting move(s) for {:?}", selected_marble);
                let indexes = if marble.index == BOARD.len() {
                    // since this marble is in its base, we have to use the origin instead of its index
                    commands.spawn_bundle(SpriteBundle{
                        texture: highlight_data.texture.clone(),
                        transform: Transform::from_xyz(marble.origin.x, marble.origin.y, 2.0),
                        ..default()
                    })
                    .insert(Highlight{ marble: *selected_marble, index: marble.index })
                    ;
                    indexes.clone()
                } else {
                    [vec![marble.index], indexes.clone()].concat()
                };
                for index in indexes {
                    let tile = BOARD[index];
                    let (x, y) = current_player_data.player.rotate_coords((tile.0 as f32, tile.1 as f32));
                    commands.spawn_bundle(SpriteBundle{
                        texture: highlight_data.texture.clone(),
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

/// This system removes all highlights and should be run as the exit system of
/// a "turn" state.
pub fn remove_all_highlights(
    mut commands: Commands,
    highlights: Query<(Entity, &Highlight)>,
) {
    println!("removing all highlights");
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
