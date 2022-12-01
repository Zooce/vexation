use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::CursorMoved;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub struct ChooseColorPlugin;

impl Plugin for ChooseColorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(GameState::ChooseColor)
                .with_system(choose_color_setup)
            )
            .add_system_set(SystemSet::on_update(GameState::ChooseColor)
                .with_system(mouse_hover_handler)
                .with_system(mouse_click_handler)
            )
            .add_system_set(SystemSet::on_exit(GameState::ChooseColor)
                .with_system(choose_color_cleanup)
            )
            ;
    }
}

#[derive(Debug, Resource)]
struct ChooseColorData {
    pub masks: [Handle<Image>;4],
    pub current_color: Option<Player>,
    pub current_mask: Option<Entity>,
}

fn choose_color_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    // clear out mouse button clicks that carry over from the main menu
    mouse_buttons.clear();
    commands.insert_resource(ChooseColorData{
        masks: [
            asset_server.load("red-mask.png"),
            asset_server.load("green-mask.png"),
            asset_server.load("blue-mask.png"),
            asset_server.load("yellow-mask.png"),
        ],
        current_color: None,
        current_mask: None,
    });
}

fn mouse_hover_handler(
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
    let lr = (pos.x >= WINDOW_SIZE / 2.).into();
    let bt = (pos.y >= WINDOW_SIZE / 2.).into();
    match (lr, bt) {
        (0, 0) => Some(Player::Yellow),
        (0, 1) => Some(Player::Red),
        (1, 0) => Some(Player::Blue),
        (1, 1) => Some(Player::Green),
        _ => None,
    }
}

fn mouse_click_handler(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    mut app_exit_events: EventWriter<AppExit>, // FIXME: workaround for https://github.com/bevyengine/bevy/commit/07d576987a7f2bdcabc97fefcc043e19e1a30222
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let cursor = match windows.get_primary() {
            Some(w) => w.cursor_position().unwrap(),
            None => {
                app_exit_events.send(AppExit);
                return;
            }
        };
        if let Some(color) = position_to_color(cursor) {
            let human_indicator = commands.spawn(SpriteBundle{
                texture: asset_server.load("human-indicator.png"),
                transform: {
                    let (x, y) = match color {
                        Player::Red => (-4.0, 4.0),
                        Player::Green => (4.0, 4.0),
                        Player::Blue => (4.0, -4.0),
                        Player::Yellow => (-4.0, -4.0),
                    };
                    Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.0)
                },
                ..default()
            }).id();
            commands.insert_resource(HumanPlayer{ color, human_indicator });
            state.set(GameState::NextPlayer).unwrap();
        }
    }
}

fn show_mask(mut commands: Commands, mut choose_color_data: ResMut<ChooseColorData>) {
    if let Some(mask) = choose_color_data.current_mask {
        commands.entity(mask).despawn();
    }
    choose_color_data.current_mask = Some(commands.spawn(SpriteBundle{
        texture: choose_color_data.masks[choose_color_data.current_color.unwrap() as usize].clone(),
        transform: Transform::from_xyz(0., 0., 3.),
        ..default()
    }).id());
}

fn choose_color_cleanup(
    mut commands: Commands,
    mut choose_color_data: ResMut<ChooseColorData>,
) {
    if let Some(mask) = choose_color_data.current_mask {
        commands.entity(mask).despawn();
        choose_color_data.current_mask = None;
    }
    commands.remove_resource::<ChooseColorData>();
}
