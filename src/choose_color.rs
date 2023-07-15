use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::CursorMoved;
use bevy::window::PrimaryWindow;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub struct ChooseColorPlugin;

impl Plugin for ChooseColorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::ChooseColor), choose_color_setup)
            .add_systems(Update,
                (mouse_hover_handler, mouse_click_handler)
                .run_if(in_state(GameState::ChooseColor))
            )
            .add_systems(OnExit(GameState::ChooseColor), choose_color_cleanup)
            ;
    }
}

#[derive(Debug, Resource)]
struct ChooseColorData {
    pub current_player: Option<Player>,
    pub mask_entity: Option<Entity>,
    pub mask_sprite: Handle<Image>,
}

#[derive(Component)]
struct Mask;

fn choose_color_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    // clear out mouse button clicks that carry over from the main menu
    mouse_buttons.clear();
    commands.insert_resource(ChooseColorData{
        current_player: None,
        mask_entity: None,
        mask_sprite: asset_server.load("mask.png"),
    });
}

fn mouse_hover_handler(
    commands: Commands,
    mut cursor_moved: EventReader<CursorMoved>,
    mut choose_color_data: ResMut<ChooseColorData>,
    mask: Query<&mut Transform, With<Mask>>,
) {
    if let Some(event) = cursor_moved.iter().last() {
        let color = position_to_color(event.position);
        if color.is_some() && color != choose_color_data.current_player {
            choose_color_data.current_player = color;
            show_mask(commands, choose_color_data, mask);
        }
    }
}

fn position_to_color(pos: Vec2) -> Option<Player> {
    let lr = (pos.x >= WINDOW_SIZE / 2.).into();
    let bt = (pos.y >= WINDOW_SIZE / 2.).into();
    match (lr, bt) {
        (0, 0) => Some(Player::Red),
        (0, 1) => Some(Player::Yellow),
        (1, 0) => Some(Player::Green),
        (1, 1) => Some(Player::Blue),
        _ => None,
    }
}

fn mouse_click_handler(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let Some(cpos) = windows.get_single().map_or(None, |w| w.cursor_position()) else {
            return;
        };
        if let Some(color) = position_to_color(cpos) {
            let human_indicator = commands.spawn(SpriteBundle{
                texture: asset_server.load("human-indicator.png"), // TODO: change indicator for power ups
                transform: {
                    let (x, y) = match color {
                        Player::Red => (-4.0, 4.0),
                        Player::Green => (4.0, 4.0),
                        Player::Blue => (4.0, -4.0),
                        Player::Yellow => (-4.0, -4.0),
                    };
                    Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, Z_HUMAN_INDICATOR)
                },
                ..default()
            }).id();
            commands.insert_resource(HumanPlayer{ color, human_indicator });
            next_state.set(GameState::NextPlayer);
        }
    }
}

fn show_mask(
    mut commands: Commands,
    mut choose_color_data: ResMut<ChooseColorData>,
    mut mask: Query<&mut Transform, With<Mask>>,
) {
    let rotation = match choose_color_data.current_player {
        Some(Player::Red) => Quat::from_rotation_z(0.0),
        Some(Player::Yellow) => Quat::from_rotation_z(PI / 2.0),
        Some(Player::Blue) => Quat::from_rotation_z(PI),
        Some(Player::Green) => Quat::from_rotation_z(3.0 * PI / 2.0),
        None => Quat::NAN,
    };
    if choose_color_data.mask_entity.is_some() {
        let mut transform = mask.single_mut();
        transform.rotation = rotation;
        return;
    }
    let mut transform = Transform::from_xyz(0., 0., Z_UI);
    transform.rotation = rotation;
    choose_color_data.mask_entity = Some(commands.spawn((
        SpriteBundle{
            texture: choose_color_data.mask_sprite.clone(),
            transform,
            ..default()
        },
        Mask,
    )).id());
}

fn choose_color_cleanup(
    mut commands: Commands,
    mut choose_color_data: ResMut<ChooseColorData>,
) {
    if let Some(mask) = choose_color_data.mask_entity {
        commands.entity(mask).despawn();
        choose_color_data.mask_entity = None;
    }
    commands.remove_resource::<ChooseColorData>();
}
