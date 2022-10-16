use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::PresentMode;

mod components;
mod constants;
mod events;
mod human_turn;
mod main_menu;
mod resources;
mod shared_systems;
mod system_sets;
mod utils;
mod vexation;

use constants::*;
use main_menu::*;
use vexation::VexationPlugin;

fn main() {
    App::new()
        // resources
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            title: "Vexation".to_string(),
            width: WINDOW_SIZE,
            height: WINDOW_SIZE,
            resizable: false,
            present_mode: PresentMode::Fifo,
            ..default()
        })

        .add_startup_system(setup)

        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(MainMenuPlugin)
        .add_plugin(VexationPlugin)

        // go!
        .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game
