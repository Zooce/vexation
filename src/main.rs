// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let size = Vec2::new(UI_BUTTON_WIDTH, UI_BUTTON_HEIGHT);
    let grid = (3, 1);
    commands.insert_resource(UiAssets{
        font: asset_server.load("Kenney Thick.ttf"),
        mini_font: asset_server.load("Kenney Mini.ttf"),
        title: asset_server.load("title.png"),
        play_button: load_sprite_sheet("buttons/play_button.png", size, grid, &asset_server, &mut texture_atlases),
        rules_button: load_sprite_sheet("buttons/rules_button.png", size, grid, &asset_server, &mut texture_atlases),
        quit_button: load_sprite_sheet("buttons/quit_button.png", size, grid, &asset_server, &mut texture_atlases),
        back_button: load_sprite_sheet("buttons/back_button.png", size, grid, &asset_server, &mut texture_atlases),
        next_button: load_sprite_sheet("buttons/next_button.png", size, grid, &asset_server, &mut texture_atlases),
    });
}

fn load_sprite_sheet(
    name: &str,
    size: Vec2,
    (cols, rows): (usize, usize),
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas>
{
    texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load(name), size, cols, rows
    ))
}

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game
