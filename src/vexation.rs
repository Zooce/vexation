// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::shared_systems::*;
use crate::system_sets::*;
use crate::utils::*;

pub struct VexationPlugin;

impl Plugin for VexationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BufferTimer(Timer::from_seconds(1.0, false)))
            .insert_resource(ComputerTurnTimer(Timer::from_seconds(1.5, false)))
            .insert_resource(RollAnimationTimer(Timer::from_seconds(1.5, false)))

            .add_event::<ClickEvent>()

            .add_startup_system(setup)

            .add_state(GameState::ChooseColor)

            // marble animation system
            .add_system_set(SystemSet::new()
                .with_run_criteria(should_animate_moves)
                .with_system(animate_marble_moves)
            )

            // choose color
            .add_system_set(SystemSet::on_update(GameState::ChooseColor)
                .with_system(mouse_hover_handler)
                .with_system(mouse_click_handler)
            )
            .add_system_set(SystemSet::on_exit(GameState::ChooseColor).with_system(human_player_chosen))

            // next player
            .add_system_set(SystemSet::on_enter(GameState::NextPlayer)
                .with_system(choose_next_player)
                .with_system(next_player_setup.after(choose_next_player))
            )

            // dice roll
            .add_system_set(SystemSet::on_enter(GameState::DiceRoll).with_system(roll_dice))
            .add_system_set(SystemSet::on_update(GameState::DiceRoll).with_system(roll_animation))
            .add_system_set(SystemSet::on_exit(GameState::DiceRoll).with_system(stop_roll_animation))

            // turn setup
            .add_system_set(SystemSet::on_enter(GameState::TurnSetup).with_system(calc_possible_moves))
            .add_system_set(SystemSet::on_update(GameState::TurnSetup).with_system(buffer_timer))

            // computer turn
            .add_system_set(SystemSet::on_update(GameState::ComputerTurn)
                .with_system(choose_move)
            )

            // human turn
            .add_system_set(SystemSet::on_update(GameState::HumanTurn)
                .with_system(handle_mouse_clicks)
                .with_system(destination_click_handler)
                .with_system(highlighter)
            )
            .add_system_set(SystemSet::on_exit(GameState::HumanTurn)
                .with_system(remove_all_highlights)
            )

            // process move
            .add_system_set(SystemSet::on_enter(GameState::ProcessMove)
                .with_system(check_for_capture)
                .with_system(check_for_winner.after(check_for_capture))
            )
            ;
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // need a 2D camera so we can see things
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // board
    commands.spawn_bundle(SpriteBundle{
        texture: asset_server.load("board.png"),
        ..default()
    });

    // choose color resource
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

    // pick the first player randomly
    let current_player: Player = ((roll_die() - 1) % 4).into();
    commands.insert_resource(CurrentPlayerData{
        player: current_player.clone(),
        possible_moves: Vec::new(),
    });

    // marbles
    let red_marble = asset_server.load("red-marble.png");
    let green_marble = asset_server.load("green-marble.png");
    let blue_marble = asset_server.load("blue-marble.png");
    let yellow_marble = asset_server.load("yellow-marble.png");
    for (x, y) in vec![(3., 3.5), (3., 4.5), (4., 3.), (4., 4.), (4., 5.)] {
        // green marbles
        let origin = Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.);
        let mut green = commands
            .spawn_bundle(SpriteBundle{
                texture: green_marble.clone(),
                transform: origin.clone(),
                ..default()
            });
        green
            .insert(Marble{ index: BOARD.len(), origin: origin.translation })
            .insert(Player::Green)
            ;
        if current_player == Player::Green {
            green.insert(CurrentPlayer);
        }
        // yellow marbles
        let origin = Transform::from_xyz(-x * TILE_SIZE, -y * TILE_SIZE, 1.);
        let mut yellow = commands
            .spawn_bundle(SpriteBundle{
                texture: yellow_marble.clone(),
                transform: origin,
                ..default()
            });
        yellow
            .insert(Marble{ index: BOARD.len(), origin: origin.translation })
            .insert(Player::Yellow)
            ;
        if current_player == Player::Yellow {
            yellow.insert(CurrentPlayer);
        }
        // red marbles
        let origin = Transform::from_xyz(-y * TILE_SIZE, x * TILE_SIZE, 1.);
        let mut red = commands
            .spawn_bundle(SpriteBundle{
                texture: red_marble.clone(),
                transform: origin,
                ..default()
            });
        red
            .insert(Marble{ index: BOARD.len(), origin: origin.translation })
            .insert(Player::Red)
            ;
        if current_player == Player::Red {
            red.insert(CurrentPlayer);
        }
        // blue marbles
        let origin = Transform::from_xyz(y * TILE_SIZE, -x * TILE_SIZE, 1.);
        let mut blue = commands
            .spawn_bundle(SpriteBundle{
                texture: blue_marble.clone(),
                transform: origin,
                ..default()
            });
        blue
            .insert(Marble{ index: BOARD.len(), origin: origin.translation })
            .insert(Player::Blue)
            ;
        if current_player == Player::Blue {
            blue.insert(CurrentPlayer);
        }
    }

    // die sprite sheet
    let texture_atlas = TextureAtlas::from_grid(
        asset_server.load("die-sheet.png"), Vec2::new(32.0, 32.0), 6, 1
    );
    let die_sheet_handle = texture_atlases.add(texture_atlas);
    let die_1 = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: die_sheet_handle.clone(),
            visibility: Visibility{ is_visible: false },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..default()
        })
        .insert(Die { location: Vec3::new(0.0, 0.0, 2.0), timer: Timer::from_seconds(0.1, true) })
        .id()
        ;
    let die_2 = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: die_sheet_handle.clone(),
            visibility: Visibility{ is_visible: false },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..default()
        })
        .insert(Die { location: Vec3::new(0.0, 0.0, 2.0), timer: Timer::from_seconds(0.1, true) })
        .id()
        ;

    commands.insert_resource(DiceData {
        die_1,
        die_2,
        die_sheet_handle,
        die_1_side: None,
        die_2_side: None,
        doubles: false
    });

    // selection data
    commands.insert_resource(SelectionData{
        marble: None,
    });
    // highlight data
    commands.insert_resource(HighlightData{
        texture: asset_server.load("tile-highlight.png"),
    });
}
