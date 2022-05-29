use bevy::prelude::*;
use bevy::window::PresentMode;

const TILE_SIZE: f32 = 32.;
const WINDOW_SIZE: f32 = TILE_SIZE * 17.;

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
        app.add_startup_system(setup);
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
