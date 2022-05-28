use bevy::prelude::*;

fn main() {
    App::new()
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
