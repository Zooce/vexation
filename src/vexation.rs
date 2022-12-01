use std::collections::HashMap;

use bevy::prelude::*;
use crate::buttons::*;
use crate::components::*;
use crate::computer_turn::*;
use crate::constants::*;
use crate::end_turn::*;
use crate::choose_color::ChooseColorPlugin;
use crate::dice_roll::DiceRollPlugin;
use crate::human_turn::HumanTurnPlugin;
use crate::next_player::*;
use crate::power::PowerUpPlugin;
use crate::process::ProcessMovePlugin;
use crate::resources::*;
use crate::shared_systems::*;
use crate::turn_setup::*;
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub struct VexationPlugin;

impl Plugin for VexationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<HighlightEvent>()
            .add_event::<MarbleAnimationDoneEvent>()
            .add_event::<ActionEvent<GameButtonAction>>()

            // game play enter
            .add_system_set(SystemSet::on_update(GameState::GameStart)
                .with_system(create_game)
            )

            // game play exit
            .add_system_set(SystemSet::on_update(GameState::GameEnd)
                .with_system(destroy_game)
            )

            // --- states + systems -- TODO: move each to their own plugin to keep things smaller?

            // shared systems
            .add_system_set(SystemSet::new()
                .label(SharedSystemLabel)
                .with_run_criteria(should_run_shared_systems)
                .with_system(animate_marble_moves)
                .with_system(highlighter)
                .with_system(animate_tile_highlights)
                .with_system(dim_used_die)
            )
            .add_plugin(PowerUpPlugin)

            // next player
            .add_system_set(SystemSet::on_update(GameState::NextPlayer)
                .with_system(choose_next_player)
                .with_system(show_or_hide_buttons.after(choose_next_player))
                .with_system(next_player_setup.after(show_or_hide_buttons))
            )

            // turn setup
            .add_system_set(SystemSet::on_update(GameState::TurnSetup)
                .with_system(calc_possible_moves)
                .with_system(count_moves.after(calc_possible_moves))
                .with_system(turn_setup_complete.after(count_moves))
            )

            // computer turn
            .add_system_set(SystemSet::on_enter(GameState::ComputerTurn)
                .with_system(clear_animation_events)
                .with_system(computer_choose_move.after(clear_animation_events))
            )
            .add_system_set(SystemSet::on_update(GameState::ComputerTurn)
                .with_system(computer_move_buffer)
            )

            .add_system_set(SystemSet::on_update(GameState::WaitForAnimation)
                .with_system(wait_for_marble_animation)
            )

            .add_plugin(ChooseColorPlugin)
            .add_plugin(DiceRollPlugin)
            .add_plugin(HumanTurnPlugin)
            .add_plugin(ProcessMovePlugin)
            
            // end turn
            .add_system_set(SystemSet::on_update(GameState::EndTurn)
                .with_system(end_turn)
            )
            ;
    }
}

pub fn create_game(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut state: ResMut<State<GameState>>,
    asset_server: Res<AssetServer>,
) {
    // insert resources
    commands.insert_resource(BufferTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    commands.insert_resource(ComputerTurnTimers{
        move_timer: Timer::from_seconds(COMPUTER_MOVE_TIMER_SECS, TimerMode::Once),
        buffer_timer: Timer::from_seconds(COMPUTER_BUFFER_TIMER_SECS, TimerMode::Once),
    });
    commands.insert_resource(RollAnimationTimer(Timer::from_seconds(1.5, TimerMode::Once)));
    commands.insert_resource(GameData{
        players: HashMap::from([
            (Player::Red, PlayerData::default()),
            (Player::Green, PlayerData::default()),
            (Player::Blue, PlayerData::default()),
            (Player::Yellow, PlayerData::default()),
        ]),
    });

    // pick the first player randomly
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(0u8, 3u8);
    let current_player: Player = rng.sample(die).into();
    commands.insert_resource(CurrentPlayerData::new(current_player));

    // board
    let board = commands.spawn(SpriteBundle{
        texture: asset_server.load("board.png"),
        ..default()
    }).id();

    // marbles
    let red_marble = asset_server.load("marbles/red-marble.png");
    let green_marble = asset_server.load("marbles/green-marble.png");
    let blue_marble = asset_server.load("marbles/blue-marble.png");
    let yellow_marble = asset_server.load("marbles/yellow-marble.png");
    for (x, y) in &[(3., 3.5), (3., 4.5), (4., 3.), (4., 4.), (4., 5.)] {
        // green marbles
        let origin = Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.);
        let mut green = commands
            .spawn((
                SpriteBundle{
                    texture: green_marble.clone(),
                    transform: origin,
                    ..default()
                },
                Marble::new(origin.translation),
                Player::Green,
            ));
        if current_player == Player::Green {
            green.insert(CurrentPlayer);
        }
        // yellow marbles
        let origin = Transform::from_xyz(-x * TILE_SIZE, -y * TILE_SIZE, 1.);
        let mut yellow = commands
            .spawn((
                SpriteBundle{
                    texture: yellow_marble.clone(),
                    transform: origin,
                    ..default()
                },
                Marble::new(origin.translation),
                Player::Yellow,
            ));
        if current_player == Player::Yellow {
            yellow.insert(CurrentPlayer);
        }
        // red marbles
        let origin = Transform::from_xyz(-y * TILE_SIZE, x * TILE_SIZE, 1.);
        let mut red = commands
            .spawn((
                SpriteBundle{
                    texture: red_marble.clone(),
                    transform: origin,
                    ..default()
                },
                Marble::new(origin.translation),
                Player::Red,
            ));
        if current_player == Player::Red {
            red.insert(CurrentPlayer);
        }
        // blue marbles
        let origin = Transform::from_xyz(y * TILE_SIZE, -x * TILE_SIZE, 1.);
        let mut blue = commands
            .spawn((
                SpriteBundle{
                    texture: blue_marble.clone(),
                    transform: origin,
                    ..default()
                },
                Marble::new(origin.translation),
                Player::Blue,
            ));
        if current_player == Player::Blue {
            blue.insert(CurrentPlayer);
        }
    }

    // die sprite sheet
    let die_sheet_handle = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("die-sheet.png"), Vec2::new(32.0, 32.0), 6, 1, None, None
    ));
    let die_1 = commands
        .spawn((
            SpriteSheetBundle{
                texture_atlas: die_sheet_handle.clone(),
                visibility: Visibility{ is_visible: false },
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            },
            Die{
                location: Vec3::new(0.0, 0.0, 2.0), 
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
        ))
        .id();
    let die_2 = commands
        .spawn((
            SpriteSheetBundle{
                texture_atlas: die_sheet_handle.clone(),
                visibility: Visibility{ is_visible: false },
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            },
            Die{
                location: Vec3::new(0.0, 0.0, 2.0),
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            }
        ))
        .id();

    commands.insert_resource(DiceData {
        die_1,
        die_2,
        die_sheet_handle,
        dice: Dice::default(),
    });

    // highlight data
    commands.insert_resource(HighlightData{
        marble_texture: asset_server.load("marble-highlight.png"),
        tile_texture: asset_server.load("tile-highlight.png"),
    });

    // UI buttons (power-ups + turn end)
    let ui = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {

            let sprite_sheet = texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("buttons/done_button.png"), Vec2::new(160.0, 48.0), 3, 1, None, None
            ));
            let transform = Transform::from_xyz(0.0, (-WINDOW_SIZE / 2.0) + TILE_SIZE, 5.0);
            spawn_sprite_sheet_button(
                parent,
                sprite_sheet,
                transform,
                ButtonAction(ActionEvent(GameButtonAction::Done)),
                false,
                ButtonState::NotHovered,
            );
        })
        .id()
        ;

    commands.insert_resource(GamePlayEntities{ board, ui });

    state.set(GameState::ChooseColor).unwrap();
}

pub fn destroy_game(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
    game_play_entities: Res<GamePlayEntities>,
    human_player: Res<HumanPlayer>,
    marbles: Query<Entity, With<Marble>>,
) {
    commands.entity(game_play_entities.board).despawn();
    commands.entity(game_play_entities.ui).despawn_recursive();
    commands.entity(human_player.human_indicator).despawn();
    commands.entity(dice_data.die_1).despawn();
    commands.entity(dice_data.die_2).despawn();

    texture_atlases.remove(dice_data.die_sheet_handle.id());

    commands.remove_resource::<GamePlayEntities>();
    commands.remove_resource::<BufferTimer>();
    commands.remove_resource::<ComputerTurnTimers>();
    commands.remove_resource::<RollAnimationTimer>();
    commands.remove_resource::<CurrentPlayerData>();
    commands.remove_resource::<DiceData>();
    commands.remove_resource::<HighlightData>();

    for marble in &marbles {
        commands.entity(marble).despawn();
    }

    state.set(GameState::MainMenu).unwrap();
}
