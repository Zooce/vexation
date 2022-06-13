// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::window::PresentMode;

mod components;
mod constants;
mod events;
mod resources;
mod shared_systems;
mod system_sets;
mod utils;
mod vexation;

use constants::*;
use vexation::VexationPlugin;

fn main() {
    App::new()
        // resources
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            title: "Vexation".to_string(),
            width: WINDOW_SIZE,
            height: WINDOW_SIZE,
            resizable: false,
            present_mode: PresentMode::Fifo,
            ..default()
        })

        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(VexationPlugin)

        // go!
        .run();
}

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game
