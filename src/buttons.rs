use bevy::prelude::*;
use crate::constants::*;

/// An `ActionEvent` that is sent when a button is clicked. The type `T` defines
/// what those actions really are.
#[derive(Clone, Copy, Debug)]
pub struct ActionEvent<T>(pub T);

#[derive(Component, Debug)]
pub struct ButtonAction<T>(pub ActionEvent<T>);

#[derive(Component, Clone, Copy, Debug)]
pub enum ButtonState {
    NotHovered,
    Hovered,
    Pressed,
    PressedNotHovered,
}

#[derive(Component, Debug)]
pub struct ButtonSize(pub Vec2);

/// This system is responsible for changing button states based on the mouse location and its
/// button status.
pub fn mouse_watcher<T: Copy + Send + Sync + 'static>(
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut button_query: Query<(&mut ButtonState, &ButtonAction<T>, &Transform, &ButtonSize)>,
    mut action_events: EventWriter<ActionEvent<T>>,
) {
    let cursor_move_event = cursor_moved_events.iter().last();

    for (mut button_state, action, transform, button_size) in &mut button_query {
        match (*button_state, cursor_move_event) {
            (ButtonState::NotHovered, Some(move_event)) => {
                if is_in_bounds(move_event.position, transform.translation, button_size.0) {
                    *button_state = ButtonState::Hovered;
                }
            }
            (ButtonState::Hovered, moved) => {
                if mouse_button_inputs.just_pressed(MouseButton::Left) {
                    *button_state = ButtonState::Pressed;
                } else if let Some(move_event) = moved {
                    if !is_in_bounds(move_event.position, transform.translation, button_size.0) {
                        *button_state = ButtonState::NotHovered;
                    }
                }
            }
            (ButtonState::Pressed, moved) => {
                if mouse_button_inputs.just_released(MouseButton::Left) {
                    *button_state = ButtonState::Hovered;
                    action_events.send(action.0)
                } else if let Some(move_event) = moved {
                    if !is_in_bounds(move_event.position, transform.translation, button_size.0) {
                        *button_state = ButtonState::PressedNotHovered;
                    }
                }
            }
            (ButtonState::PressedNotHovered, moved) => {
                if mouse_button_inputs.just_released(MouseButton::Left) {
                    *button_state = ButtonState::NotHovered;
                } else if let Some(move_event) = moved {
                    if is_in_bounds(move_event.position, transform.translation, button_size.0) {
                        *button_state = ButtonState::Pressed;
                    }
                }
            }
            _ => {}
        }
    }
}

/// This is a helper function used specifically in this file.
fn is_in_bounds(cursor_pos: Vec2, button_pos: Vec3, button_size: Vec2) -> bool {
    let (x, y) = (cursor_pos.x - WINDOW_SIZE / 2.0, cursor_pos.y - WINDOW_SIZE / 2.0);
    x > button_pos.x - button_size.x / 2.0 &&
    x < button_pos.x + button_size.x / 2.0 &&
    y > button_pos.y - button_size.y / 2.0 &&
    y < button_pos.y + button_size.y / 2.0
}

/// This is a helper function used to get the state of a button.
pub fn get_button_state(
    cursor_pos: Option<Vec2>,
    button_pos: Vec3,
    button_size: Vec2,
    mouse_pressed: bool,
) -> ButtonState {
    if let Some(cursor_pos) = cursor_pos {
        if is_in_bounds(cursor_pos, button_pos, button_size) {
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

/// This system is responsible for reacting to button state changes.
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

pub fn spawn_sprite_sheet_button<T: Send + Sync + 'static>(
    parent: &mut ChildBuilder,
    texture_atlas: Handle<TextureAtlas>,
    transform: Transform,
    action: ButtonAction<T>,
    is_visible: bool,
    button_state: ButtonState,
    button_size: ButtonSize,
) {
    parent
        .spawn((
            SpriteSheetBundle{
                sprite: TextureAtlasSprite {
                    index: match button_state {
                        ButtonState::NotHovered => 0,
                        ButtonState::Hovered => 1,
                        ButtonState::Pressed | ButtonState::PressedNotHovered => 2,
                    },
                    ..default()
                },
                texture_atlas,
                transform,
                visibility: Visibility{ is_visible },
                ..default()
            },
            button_state,
            button_size,
            action,
        ));
}

pub fn sprite_sheet_button_bundle<T: Send + Sync + 'static>(
    texture_atlas: Handle<TextureAtlas>,
    transform: Transform,
    action: ButtonAction<T>,
    is_visible: bool,
    button_state: ButtonState,
    button_size: ButtonSize,
) -> impl Bundle {
    (
        SpriteSheetBundle{
            sprite: TextureAtlasSprite{
                index: match button_state {
                    ButtonState::NotHovered => 0,
                    ButtonState::Hovered => 1,
                    ButtonState::Pressed | ButtonState::PressedNotHovered => 2,
                },
                ..default()
            },
            texture_atlas,
            transform,
            visibility: Visibility{ is_visible },
            ..default()
        },
        button_state,
        button_size,
        action,
    )
}

pub fn load_sprite_sheet(
    name: &str,
    size: Vec2,
    (cols, rows): (usize, usize),
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load(name), size, cols, rows, None, None
    ))
}
