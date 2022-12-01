use bevy::prelude::*;
use crate::constants::*;

/// An `ActionEvent` that is sent when a button is clicked. The type `T` defines
/// what those actions really are.
#[derive(Clone, Copy)]
pub struct ActionEvent<T>(pub T);

#[derive(Component)]
pub struct ButtonAction<T>(pub ActionEvent<T>);

#[derive(Component, Clone, Copy, Debug)]
pub enum ButtonState {
    NotHovered,
    Hovered,
    Pressed,
    PressedNotHovered,
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

// TODO: create a builder for this sprite sheet button stuff
pub fn spawn_sprite_sheet_button<T: Send + Sync + 'static>(
    parent: &mut ChildBuilder,
    texture_atlas: Handle<TextureAtlas>,
    transform: Transform,
    action: ButtonAction<T>,
    is_visible: bool,
    button_state: ButtonState,
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
            action,
        ));
}

