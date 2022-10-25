use bevy::prelude::*;
use bevy::ecs::schedule::ShouldRun;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;

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

pub fn mouse_watcher<T: Copy + Send + Sync + 'static>(
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut button_query: Query<(&mut ButtonState, &ButtonAction<T>, &Transform)>,
    mut action_events: EventWriter<ActionEvent<T>>,
) {
    let cursor_move_event = cursor_moved_events.iter().last();

    for (mut button_state, action, transform) in &mut button_query {
        match (*button_state, cursor_move_event) {
            (ButtonState::NotHovered, Some(move_event)) => {
                if is_in_bounds(move_event.position, transform.translation) {
                    *button_state = ButtonState::Hovered;
                }
            }
            (ButtonState::Hovered, moved) => {
                if mouse_button_inputs.just_pressed(MouseButton::Left) {
                    *button_state = ButtonState::Pressed;
                } else if let Some(move_event) = moved {
                    if !is_in_bounds(move_event.position, transform.translation) {
                        *button_state = ButtonState::NotHovered;
                    }
                }
            }
            (ButtonState::Pressed, moved) => {
                if mouse_button_inputs.just_released(MouseButton::Left) {
                    *button_state = ButtonState::Hovered;
                    action_events.send(action.0)
                } else if let Some(move_event) = moved {
                    if !is_in_bounds(move_event.position, transform.translation) {
                        *button_state = ButtonState::PressedNotHovered;
                    }
                }
            }
            (ButtonState::PressedNotHovered, moved) => {
                if mouse_button_inputs.just_released(MouseButton::Left) {
                    *button_state = ButtonState::NotHovered;
                } else if let Some(move_event) = moved {
                    if is_in_bounds(move_event.position, transform.translation) {
                        *button_state = ButtonState::Pressed;
                    }
                }
            }
            _ => {}
        }
    }
}

/// This is a helper function used specifically in this file.
fn is_in_bounds(cursor_pos: Vec2, button_pos: Vec3) -> bool {
    let (x, y) = (cursor_pos.x - WINDOW_SIZE / 2.0, cursor_pos.y - WINDOW_SIZE / 2.0);
    x > button_pos.x - UI_BUTTON_WIDTH / 2.0 &&
    x < button_pos.x + UI_BUTTON_WIDTH / 2.0 &&
    y > button_pos.y - UI_BUTTON_HEIGHT / 2.0 &&
    y < button_pos.y + UI_BUTTON_HEIGHT / 2.0
}

/// This is a helper function used to get the state of a button.
pub fn get_button_state(
    cursor_pos: Option<Vec2>,
    button_pos: Vec3,
    mouse_pressed: bool
) -> ButtonState {
    if let Some(cursor_pos) = cursor_pos {
        if is_in_bounds(cursor_pos, button_pos) {
            if mouse_pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Hovered
            }
        } else {
            ButtonState::NotHovered
        }
    } else {
        ButtonState::NotHovered
    }
}

pub fn watch_button_state_changes(
    mut button_query: Query<(&mut TextureAtlasSprite, &ButtonState), Changed<ButtonState>>
) {
    for (mut sprite, state) in &mut button_query {
        match *state {
            ButtonState::NotHovered => sprite.index = 0,
            ButtonState::Hovered => sprite.index = 1,
            ButtonState::Pressed => sprite.index = 2,
            _ => {}
        }
    }
}

// POWERUP: add update power bar system - recieves power bar events
pub fn update_power_bars(
    mut power_bar_events: EventReader<PowerBarEvent>,
    mut game_data: ResMut<GameData>,
) {
    for event in power_bar_events.iter() {
        println!("recieved {:?}", event);
        match event {
            PowerBarEvent::Capture{ captor, captive } => {
                game_data.players.get_mut(captor).unwrap().update_power(3.0);
                game_data.players.get_mut(captive).unwrap().update_power(-3.0);
            },
            PowerBarEvent::Deflection{ deflector, deflected } => {},
            PowerBarEvent::Index{player, index, prev_index} => {
                let distance = if *index == CENTER_INDEX {
                    // home (54)  -> center (53) = 7
                    // prev_index -> center (53) = (17 or 29) - prev_index + 1
                    match *prev_index {
                        54 => 7,
                        _ if (6..=17).contains(prev_index) => 17 - prev_index + 1,
                        _ if (18..=29).contains(prev_index) => 29 - prev_index + 1,
                        _ => unreachable!(),
                    }
                } else {
                    // home (54)   -> index = index + 1
                    // center (53) -> index = index + 1 - 41 
                    // prev_index  -> index = index - prev_index
                    match *prev_index {
                        54 => index + 1,
                        CENTER_INDEX => index + 1 - 41,
                        _ => index - prev_index,
                    }
                } as f32;
                let points = match index {
                    0..=47 => 1.0,
                    _ => 2.0,
                } * 10.0 * distance / 48.0;
                println!("distance: {distance}, points: {points}");
                game_data.players.get_mut(player).unwrap().update_power(points);
            }
        }
    }
}

pub fn generate_power_up(
    mut power_up_events: EventReader<GeneratePowerUpEvent>,
    mut game_data: ResMut<GameData>,
) {
    for event in power_up_events.iter() {
        println!("generating power up for {:?}", event.0);
        // pick random power-up
        // add power-up to player's list
        // mark current player to wait for animation
        // spawn power-up sprite in player's next empty power-up box
        // mark power-up for animation
    }
}

