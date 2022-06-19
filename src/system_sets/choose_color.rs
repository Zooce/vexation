use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use bevy::window::CursorMoved;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn choose_color(
    commands: Commands,
    mut cursor_moved: EventReader<CursorMoved>,
    mut choose_color_data: ResMut<ChooseColorData>,
) {
    if let Some(event) = cursor_moved.iter().last() {
        let color = position_to_color(event.position);
        if color.is_some()
            && (choose_color_data.current_color.is_none()
                || choose_color_data.current_color != color)
        {
            choose_color_data.current_color = color;
            show_mask(commands, choose_color_data);
        }
    }
}

fn position_to_color(pos: Vec2) -> Option<Player> {
    let lr = if pos.x < WINDOW_SIZE / 2. {
        0
    } else {
        1
    };
    let bt = if pos.y < WINDOW_SIZE / 2. {
        0
    } else  {
        1
    };
    match (lr, bt) {
        (0, 0) => Some(Player::Yellow),
        (0, 1) => Some(Player::Red),
        (1, 0) => Some(Player::Blue),
        (1, 1) => Some(Player::Green),
        _ => None,
    }
}

pub fn mouse_click_handler(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    windows: Res<Windows>,
    mut mouse_events: EventReader<MouseButtonInput>,
) {
    let cursor = match windows.get_primary() {
        Some(w) => match w.cursor_position() {
            Some(c) => c,
            None => return,
        }
        None => return,
    };
    if let Some(_) = mouse_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed())
        .last()
    {
        if let Some(color) = position_to_color(cursor) {
            commands.insert_resource(HumanPlayer{ color });
            state.set(GameState::NextPlayer).unwrap();
        }
    }
}

fn show_mask(mut commands: Commands, mut choose_color_data: ResMut<ChooseColorData>) {
    if let Some(mask) = choose_color_data.current_mask {
        commands.entity(mask).despawn();
    }
    choose_color_data.current_mask = Some(commands.spawn_bundle(SpriteBundle{
        texture: choose_color_data.masks[choose_color_data.current_color.unwrap() as usize].clone(),
        transform: Transform::from_xyz(0., 0., 3.),
        ..default()
    }).id());
}

pub fn human_player_chosen(
    mut commands: Commands,
    mut choose_color_data: ResMut<ChooseColorData>,
) {
    if let Some(mask) = choose_color_data.current_mask {
        commands.entity(mask).despawn();
        choose_color_data.current_mask = None;
    }
}
