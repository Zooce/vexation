// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::window::PresentMode;

mod components;
mod constants;
mod events;
mod main_menu;
mod resources;
mod shared_systems;
mod system_sets;
mod utils;
mod vexation;

use constants::*;
use main_menu::*;
use resources::*;
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

        .add_startup_system(global_setup)

        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(MainMenuPlugin)
        .add_plugin(VexationPlugin)

        // go!
        .run();
}

pub fn global_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // we need the UI camera for the entire game so make it now
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(UiAssets{
        font: asset_server.load("Kenney Thick.ttf"),
        mini_font: asset_server.load("Kenney Mini.ttf"),
        normal_button: asset_server.load("red_button11.png"),
        hovered_button: asset_server.load("red_button10.png"),
        pressed_button: asset_server.load("red_button12.png"),
    });
}

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game
