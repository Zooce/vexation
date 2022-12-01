use bevy::prelude::*;
use bevy::window::PresentMode;

mod buttons;
mod components;
mod computer_turn;
mod constants;
mod choose_color;
mod dice_roll;
mod end_turn;
mod human_turn;
mod main_menu;
mod next_player;
mod power;
mod process;
mod resources;
mod shared_systems;
mod turn_setup;
mod vexation;

use constants::*;
use main_menu::*;
use vexation::VexationPlugin;

fn main() {
    App::new()
        // resources
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))

        .add_startup_system(setup)

        // plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Vexation".to_string(),
                width: WINDOW_SIZE,
                height: WINDOW_SIZE,
                resizable: false,
                present_mode: PresentMode::Fifo,
                ..default()
            },
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugin(MainMenuPlugin)
        .add_plugin(VexationPlugin)

        // go!
        .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game
