use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use bevy::math::const_vec2;

use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

const TILE_SIZE: f32 = 32.;
const TILE_COUNT: f32 = 17.;
const WINDOW_SIZE: f32 = TILE_SIZE * TILE_COUNT;
const RED_DICE_LOCATIONS: (Vec2, Vec2) = (const_vec2!([-3.0 * TILE_SIZE, 5.5 * TILE_SIZE]), const_vec2!([-5.0 * TILE_SIZE, 5.5 * TILE_SIZE]));
const GREEN_DICE_LOCATIONS: (Vec2, Vec2) = (const_vec2!([5.5 * TILE_SIZE, 3.0 * TILE_SIZE]), const_vec2!([5.5 * TILE_SIZE, 5.0 * TILE_SIZE]));
const BLUE_DICE_LOCATIONS: (Vec2, Vec2) = (const_vec2!([3.0 * TILE_SIZE, -5.5 * TILE_SIZE]), const_vec2!([5.0 * TILE_SIZE, -5.5 * TILE_SIZE]));
const YELLOW_DICE_LOCATIONS: (Vec2, Vec2) = (const_vec2!([-5.5 * TILE_SIZE, -3.0 * TILE_SIZE]), const_vec2!([-5.5 * TILE_SIZE, -5.0 * TILE_SIZE]));

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    // ChooseColor,
    NextPlayer,
    TurnSetup,
    ChooseMoves,
    ComputerChooseMoves,
    CheckWinner,
}

#[derive(Debug)]
pub struct DiceData {
    pub die_1: Entity,
    pub die_2: Entity,
    pub die_sheet_handle: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct Die {
    animation_timer: Timer,
    side: u8,
}

#[derive(Component)]
pub enum Player {
    Red,
    Green,
    Blue,
    Yellow,
}

pub struct PlayerData {
    current_player: Player,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))

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
            .insert_resource(RollAnimationTimer(Timer::from_seconds(3., false)))
            .insert_resource(PlayerData{ current_player: Player::Red }) // TODO: pick starting player by rolling dice for each player - biggest roll wins

            .add_startup_system(setup)

            .add_state(GameState::NextPlayer)

            // TODO: define ChooseColor state

            .add_system_set(SystemSet::on_enter(GameState::NextPlayer).with_system(next_player))

            .add_system_set(SystemSet::on_enter(GameState::TurnSetup).with_system(turn_setup))
            .add_system_set(SystemSet::on_update(GameState::TurnSetup).with_system(roll_animation))
            .add_system_set(SystemSet::on_exit(GameState::TurnSetup).with_system(stop_roll_animation))

            .add_system_set(SystemSet::on_enter(GameState::ChooseMoves).with_system(choose_moves))
            .add_system_set(SystemSet::on_update(GameState::ChooseMoves).with_system(handle_mouse_clicks))

            .add_system_set(SystemSet::on_enter(GameState::ComputerChooseMoves).with_system(computer_choose_moves))

            .add_system_set(SystemSet::on_enter(GameState::CheckWinner).with_system(check_winner))
            ;
    }
}

pub struct RollAnimationTimer(Timer);

#[derive(Component)]
pub struct Marble;

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

    // marbles
    for (x, y) in vec![(3., 3.5), (3., 4.5), (4., 3.), (4., 4.), (4., 5.)] {
        // green marbles
        commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("green-marble.png"),
                transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.),
                ..default()
            })
            .insert(Marble)
            ;
        // yellow marbles
        commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("yellow-marble.png"),
                transform: Transform::from_xyz(-x * TILE_SIZE, -y * TILE_SIZE, 1.),
                ..default()
            })
            .insert(Marble)
            ;
        // red marbles
        commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("red-marble.png"),
                transform: Transform::from_xyz(-y * TILE_SIZE, x * TILE_SIZE, 1.),
                ..default()
            })
            .insert(Marble)
            ;
        // blue marbles
        commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("blue-marble.png"),
                transform: Transform::from_xyz(y * TILE_SIZE, -x * TILE_SIZE, 1.),
                ..default()
            })
            .insert(Marble)
            ;
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
            ..default()
        })
        .insert(Die{
            animation_timer: Timer::from_seconds(0.1, true),
            side: roll_die(),
            })
        .id()
        ;
    let die_2 = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: die_sheet_handle.clone(),
            visibility: Visibility{ is_visible: false },
            ..default()
        })
        .insert(Die{
            animation_timer: Timer::from_seconds(0.1, true),
            side: roll_die(),
            })
        .id()
        ;

    commands.insert_resource(DiceData{
        die_1,
        die_2,
        die_sheet_handle,
    });
}

// TOOD: when a human player's marble is clicked, send and event to highlight the tiles it can move to based on the current state of the dice
// TOOD: highlight marble when cursor hovers it's bounds

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game

fn next_player(mut state: ResMut<State<GameState>>, mut player_data: ResMut<PlayerData>) {
    println!("1. next_player");

    player_data.current_player = match player_data.current_player {
        Player::Red => Player::Green,
        Player::Green => Player::Blue,
        Player::Blue => Player::Yellow,
        Player::Yellow => Player::Red,
    };

    state.set(GameState::TurnSetup).unwrap();
}

fn turn_setup(dice_data: Res<DiceData>, player_data: Res<PlayerData>, mut dice: Query<(&mut Visibility, &mut Transform, &mut Die)>) {
    let (d1_loc, d2_loc) = match player_data.current_player {
        Player::Red => RED_DICE_LOCATIONS,
        Player::Green => GREEN_DICE_LOCATIONS,
        Player::Blue => BLUE_DICE_LOCATIONS,
        Player::Yellow => YELLOW_DICE_LOCATIONS,
    };

    let (mut visibility, mut transform, mut die) = dice.get_mut(dice_data.die_1).expect("Unable to get die 1");
    visibility.is_visible = true;
    transform.translation.x = d1_loc.x;
    transform.translation.y = d1_loc.y;
    die.animation_timer.reset();
    die.side = roll_die();

    let (mut visibility, mut transform, mut die) = dice.get_mut(dice_data.die_2).expect("Unable to get dice 2");
    visibility.is_visible = true;
    transform.translation.x = d2_loc.x;
    transform.translation.y = d2_loc.y;
    die.animation_timer.reset();
    die.side = roll_die();

    println!("2b. calc possible moves for current player's marbles");
}

fn roll_animation(
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut Die, &mut TextureAtlasSprite)>,
) {
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_sheet.rs
    for (mut die, mut sprite) in query.iter_mut() {
        if die.animation_timer.tick(time.delta()).just_finished() {
            sprite.index = (roll_die() - 1) as usize;
        }
    }

    if roll_animation_timer.0.tick(time.delta()).just_finished() {
        roll_animation_timer.0.reset();
        // after the animation has run for X seconds, go to the ChooseMoves state
        state.set(GameState::ChooseMoves).unwrap();
    }
}

fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

fn stop_roll_animation(
    mut query: Query<(&Die, &mut TextureAtlasSprite)>,
    dice_data: Res<DiceData>,
) {
    let (die, mut sprite) = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (die.side - 1) as usize;

    let (die, mut sprite) = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (die.side - 1) as usize;
}

fn choose_moves(mut state: ResMut<State<GameState>>) {
    println!("5. choose moves");

    // if the current player is a computer then go to the ComputerChooseMoves state
    state.set(GameState::ComputerChooseMoves).unwrap();
}

fn handle_mouse_clicks(
    mut mouse_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    marbles: Query<(&Handle<Image>, &Transform), With<Marble>>,
) {
    // we need the current position of the cursor or else we don't really care
    let cursor = match windows.get_primary() {
        Some(w) => match w.cursor_position() {
            Some(c) => c,
            None => return,
        }
        None => return,
    };

    // we really only care about the most recent left mouse button press
    if let Some(_) = mouse_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed()).last()
    {
        // cursor position is measured from the bottom left corner, but transforms are measured from their center
        let (cursor_x, cursor_y) = (cursor.x - WINDOW_SIZE / 2., cursor.y - WINDOW_SIZE / 2.);

        // find the marble under the cursor
        if let Some(_) = marbles.iter().find(|(handle, marble_transform)| {
            match images.get(*handle) {
                Some(img) => {
                    let marble_size = img.size();
                       cursor_x > marble_transform.translation.x - marble_size.x / 2.
                    && cursor_x < marble_transform.translation.x + marble_size.x / 2.
                    && cursor_y > marble_transform.translation.y - marble_size.y / 2.
                    && cursor_y < marble_transform.translation.y + marble_size.y / 2.
                }
                None => false,
            }
        }) {
            println!("clicked on marble!");
        }

        let (col, row) = ((cursor.x / WINDOW_SIZE * TILE_COUNT).floor(), (cursor.y / WINDOW_SIZE * TILE_COUNT).floor());
    }
}

fn computer_choose_moves(mut state: ResMut<State<GameState>>) {
    println!("6. computer choose moves");

    state.set(GameState::CheckWinner).unwrap();
}

fn check_winner(mut state: ResMut<State<GameState>>) {
    println!("7. check for winner");

    state.set(GameState::NextPlayer).unwrap();
}
