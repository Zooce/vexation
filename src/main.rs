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
        app
            .insert_resource(HelloTimer(Timer::from_seconds(2.0, true)))
            .add_system(hello_world);
    }
}

pub struct HelloTimer(Timer);

fn hello_world(time: Res<Time>, mut timer: ResMut<HelloTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("hello world!");
    }
}
