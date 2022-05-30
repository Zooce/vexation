use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::input::mouse::{MouseButtonInput, MouseButton};

const TILE_SIZE: f32 = 32.;
const TILE_COUNT: f32 = 17.;
const WINDOW_SIZE: f32 = TILE_SIZE * TILE_COUNT;

fn main() {
    App::new()
        // resources
        .insert_resource(WindowDescriptor {
            title: "Aggravation".to_string(),
            width: WINDOW_SIZE,
            height: WINDOW_SIZE,
            resizable: false,
            present_mode: PresentMode::Fifo,
            ..default()
        })

        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(AggravationPlugin)

        .run();
}

pub struct AggravationPlugin;

impl Plugin for AggravationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(handle_mouse_clicks)
            ;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // need a 2D camera so we can see things
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // board
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("board.png"),
        ..default()
    });
}

fn handle_mouse_clicks(mut mouse_events: EventReader<MouseButtonInput>, windows: Res<Windows>) {
    for mouse_event in mouse_events.iter() {
        if mouse_event.button == MouseButton::Left && mouse_event.state.is_pressed() {
            if let Some(window) = windows.get_primary() {
                // cursor position is measured from the bottom left corner
                if let Some(pos) = window.cursor_position() {
                    let (col, row) = ((pos.x / WINDOW_SIZE * TILE_COUNT).floor(), (pos.y / WINDOW_SIZE * TILE_COUNT).floor());
                    println!("mouse click @ {},{} -> col={}, row={}", pos.x, pos.y, col, row);
                }
            }
        }
    }
}
