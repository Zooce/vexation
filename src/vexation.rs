// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::shared_systems::*;
use crate::system_sets::*;
use crate::utils::*;

pub struct VexationPlugin;

impl Plugin for VexationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ClickEvent>()
            .add_event::<HighlightEvent>()
            .add_event::<MarbleAnimationDoneEvent>()
            .add_event::<MoveEvent>()
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
                .with_run_criteria(should_run_shared_systems)
                .with_system(animate_marble_moves)
                .with_system(highlighter)
                .with_system(animate_tile_highlights)
                .with_system(dim_used_die)
            )

            // choose color
            .add_system_set(SystemSet::on_enter(GameState::ChooseColor)
                .with_system(clear_mouse_events)
            )
            .add_system_set(SystemSet::on_update(GameState::ChooseColor)
                .with_system(mouse_hover_handler)
                .with_system(mouse_click_handler)
            )
            .add_system_set(SystemSet::on_exit(GameState::ChooseColor)
                .with_system(choose_color_cleanup)
            )

            // next player
            .add_system_set(SystemSet::on_update(GameState::NextPlayer)
                .with_system(choose_next_player)
                .with_system(show_or_hide_buttons.after(choose_next_player))
                .with_system(next_player_setup.after(show_or_hide_buttons))
            )

            // dice roll
            .add_system_set(SystemSet::on_enter(GameState::DiceRoll)
                .with_system(undim_dice)
                .with_system(remove_all_highlights)
                .with_system(roll_dice.after(remove_all_highlights))
            )
            .add_system_set(SystemSet::on_update(GameState::DiceRoll)
                .with_system(roll_animation)
            )
            .add_system_set(SystemSet::on_exit(GameState::DiceRoll)
                .with_system(stop_roll_animation)
            )

            // turn setup
            .add_system_set(SystemSet::on_update(GameState::TurnSetup)
                .with_system(calc_possible_moves)
                .with_system(turn_setup_complete.after(calc_possible_moves))
            )

            // computer turn
            .add_system_set(SystemSet::on_enter(GameState::ComputerTurn)
                .with_system(clear_animation_events)
                .with_system(computer_choose_move.after(clear_animation_events))
            )
            .add_system_set(SystemSet::on_update(GameState::ComputerTurn)
                .with_system(computer_move_buffer)
            )

            // human turn
            .add_system_set(SystemSet::on_enter(GameState::HumanTurn)
                .with_system(enable_ui)
            )
            .add_system_set(SystemSet::on_update(GameState::HumanTurn)
                // ui
                .with_system(execute_button_actions.before(mouse_watcher::<GameButtonAction>))
                .with_system(mouse_watcher::<GameButtonAction>)
                .with_system(watch_button_state_changes.after(mouse_watcher::<GameButtonAction>))
                // game play
                .with_system(translate_mouse_input)
                .with_system(interpret_click_event.after(translate_mouse_input))
                .with_system(move_event_handler.after(interpret_click_event))
            )
            .add_system_set(SystemSet::on_exit(GameState::HumanTurn)
                .with_system(disable_ui)
            )

            .add_system_set(SystemSet::on_update(GameState::WaitForAnimation)
                .with_system(wait_for_marble_animation)
            )

            // process move
            .add_system_set(SystemSet::on_update(GameState::ProcessMove)
                .with_system(check_for_capture)
                .with_system(check_for_winner.after(check_for_capture))
            )
            .add_system_set(SystemSet::on_exit(GameState::ProcessMove)
                .with_system(clear_selected_marble)
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
    commands.insert_resource(BufferTimer(Timer::from_seconds(1.0, false)));
    commands.insert_resource(ComputerTurnTimers{
        move_timer: Timer::from_seconds(COMPUTER_MOVE_TIMER_SECS, false),
        buffer_timer: Timer::from_seconds(COMPUTER_BUFFER_TIMER_SECS, false),
    });
    commands.insert_resource(RollAnimationTimer(Timer::from_seconds(1.5, false)));
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
    commands.insert_resource(CurrentPlayerData::new(current_player.clone()));

    // board
    let board = commands.spawn_bundle(SpriteBundle{
        texture: asset_server.load("board.png"),
        ..default()
    }).id();

    // marbles
    let red_marble = asset_server.load("marbles/red-marble.png");
    let green_marble = asset_server.load("marbles/green-marble.png");
    let blue_marble = asset_server.load("marbles/blue-marble.png");
    let yellow_marble = asset_server.load("marbles/yellow-marble.png");
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
    let die_sheet_handle = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("die-sheet.png"), Vec2::new(32.0, 32.0), 6, 1
    ));
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
        doubles: false,
    });

    // highlight data
    commands.insert_resource(HighlightData{
        marble_texture: asset_server.load("marble-highlight.png"),
        tile_texture: asset_server.load("tile-highlight.png"),
    });

    // UI buttons (power-ups + turn end)
    let ui = commands
        .spawn_bundle(SpatialBundle::default())
        .with_children(|parent| {

            let sprite_sheet = texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("buttons/done_button.png"), Vec2::new(160.0, 48.0), 3, 1
            ));
            let transform = Transform::from_xyz(0.0, (-WINDOW_SIZE / 2.0) + TILE_SIZE, 5.0);
            ui::spawn_sprite_sheet_button(
                parent,
                sprite_sheet,
                transform,
                ButtonAction(ActionEvent(GameButtonAction::Done)),
                false,
                ButtonState::None,
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

    texture_atlases.remove(dice_data.die_sheet_handle.id);

    commands.remove_resource::<GamePlayEntities>();
    commands.remove_resource::<BufferTimer>();
    commands.remove_resource::<ComputerTurnTimers>();
    commands.remove_resource::<RollAnimationTimer>();
    commands.remove_resource::<ChooseColorData>();
    commands.remove_resource::<CurrentPlayerData>();
    commands.remove_resource::<DiceData>();
    commands.remove_resource::<HighlightData>();

    for marble in &marbles {
        commands.entity(marble).despawn();
    }

    state.set(GameState::MainMenu).unwrap();
}
