use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::input::mouse::{MouseButtonInput, MouseButton};

use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

use std::collections::BTreeSet;

const TILE_SIZE: f32 = 32.;
const TILE_COUNT: f32 = 17.;
const WINDOW_SIZE: f32 = TILE_SIZE * TILE_COUNT;

const START_INDEX: usize = 0;
const CENTER_INDEX: usize = 53;
const LAST_HOME_INDEX: usize = 52;
const CENTER_ENTRANCE_INDEXES: [usize; 3] = [5, 17, 29];
const CENTER_EXIT_INDEX: usize = 41;

/// Main board cell indexes - rotate clockwise for each color
///
///                10 11 12
///                 9 -- 13
///                 8 -- 14
///                 7 -- 15
/// red             6 -- 16          green
///  0  1  2  3  4  5 -- 17 18 19 20 21 22
/// 47 48 49 50 51 52 53 -- -- -- -- -- 23
/// 46 45 44 43 42 41 -- 29 28 27 26 25 24
/// yellow         40 -- 30           blue
///                39 -- 31
///                38 -- 32
///                37 -- 33
///                36 35 34
///
const BOARD: [(i32, i32); 54] = [
    ((-6, 1)), // 0: start
    ((-5, 1)),
    ((-4, 1)),
    ((-3, 1)),
    ((-2, 1)),

    ((-1, 1)), // 5: shortcut entrance

    ((-1, 2)),
    ((-1, 3)),
    ((-1, 4)),
    ((-1, 5)),
    ((-1, 6)),

    ((0, 6)),

    ((1, 6)),
    ((1, 5)),
    ((1, 4)),
    ((1, 3)),
    ((1, 2)),

    ((1, 1)), // 17: shortcut entrance

    ((2, 1)),
    ((3, 1)),
    ((4, 1)),
    ((5, 1)),
    ((6, 1)),

    ((6, 0)),

    ((6, -1)),
    ((5, -1)),
    ((4, -1)),
    ((3, -1)),
    ((2, -1)),

    ((1, -1)), // 29: shortcut entrance

    ((1, -2)),
    ((1, -3)),
    ((1, -4)),
    ((1, -5)),
    ((1, -6)),

    ((0, -6)),

    ((-1, -6)),
    ((-1, -5)),
    ((-1, -4)),
    ((-1, -3)),
    ((-1, -2)),

    ((-1, -1)),

    ((-2, -1)),
    ((-3, -1)),
    ((-4, -1)),
    ((-5, -1)),
    ((-6, -1)),

    ((-6, 0)), // 47: home entrance

    // 48-52: home
    ((-5, 0)),
    ((-4, 0)),
    ((-3, 0)),
    ((-2, 0)),
    ((-1, 0)),

    ((0, 0)), // 53: center
];

const fn red(coord: (i32, i32)) -> (i32, i32) {
    coord
}

const fn green(coord: (i32, i32)) -> (i32, i32) {
    rotate(red(coord))
}

const fn blue(coord: (i32, i32)) -> (i32, i32) {
    rotate(green(coord))
}

const fn yellow(coord: (i32, i32)) -> (i32, i32) {
    rotate(blue(coord))
}

const fn rotate(coord: (i32, i32)) -> (i32, i32) {
    (coord.1, -coord.0)
}

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
    pub die_1_side: u8,
    pub die_2_side: u8,
}

#[derive(Component)]
pub struct DieAnimationTimer(Timer);

#[derive(Component, Eq, PartialEq, Clone)]
pub enum Player {
    Red,
    Green,
    Blue,
    Yellow,
}

impl From<u8> for Player {
    fn from(x: u8) -> Self {
        match x {
            0 => Player::Red,
            1 => Player::Green,
            2 => Player::Blue,
            3 => Player::Yellow,
            _ => panic!("Cannot convert {} to Player", x),
        }
    }
}

pub struct CurrentPlayerData {
    player: Player,
    possible_moves: BTreeSet<(Entity, usize)>,
}
}

pub struct RollAnimationTimer(Timer);

#[derive(Component)]
pub struct Marble {
    index: usize,
}

#[derive(Component)]
pub struct CurrentPlayer;

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

            .add_startup_system(setup)

            .add_state(GameState::NextPlayer)

            // TODO: define ChooseColor state

            .add_system_set(SystemSet::on_enter(GameState::NextPlayer).with_system(next_player))

            .add_system_set(SystemSet::on_enter(GameState::TurnSetup)
                .with_system(turn_setup)
                .with_system(calc_possible_moves.after(turn_setup))
            )
            .add_system_set(SystemSet::on_update(GameState::TurnSetup).with_system(roll_animation))
            .add_system_set(SystemSet::on_exit(GameState::TurnSetup).with_system(stop_roll_animation))

            .add_system_set(SystemSet::on_enter(GameState::ChooseMoves).with_system(choose_moves))
            .add_system_set(SystemSet::on_update(GameState::ChooseMoves).with_system(handle_mouse_clicks))

            .add_system_set(SystemSet::on_enter(GameState::ComputerChooseMoves).with_system(computer_choose_moves))

            .add_system_set(SystemSet::on_enter(GameState::CheckWinner).with_system(check_winner))
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

    // pick the first player randomly
    let current_player: Player = ((roll_die() - 1) % 4).into();
    commands.insert_resource(CurrentPlayerData{ player: current_player.clone(), possible_moves: BTreeSet::new() });

    // marbles
    for (x, y) in vec![(3., 3.5), (3., 4.5), (4., 3.), (4., 4.), (4., 5.)] {
        // green marbles
        let mut green = commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("green-marble.png"),
                transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.),
                ..default()
            });
        green
            .insert(Marble{ index: BOARD.len() })
            .insert(Player::Green)
            ;
        if current_player == Player::Green {
            green.insert(CurrentPlayer);
        }
        // yellow marbles
        let mut yellow = commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("yellow-marble.png"),
                transform: Transform::from_xyz(-x * TILE_SIZE, -y * TILE_SIZE, 1.),
                ..default()
            });
        yellow
            .insert(Marble{ index: BOARD.len() })
            .insert(Player::Yellow)
            ;
        if current_player == Player::Yellow {
            yellow.insert(CurrentPlayer);
        }
        // red marbles
        let mut red = commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("red-marble.png"),
                transform: Transform::from_xyz(-y * TILE_SIZE, x * TILE_SIZE, 1.),
                ..default()
            });
        red
            .insert(Marble{ index: BOARD.len() })
            .insert(Player::Red)
            ;
        if current_player == Player::Red {
            red.insert(CurrentPlayer);
        }
        // blue marbles
        let mut blue = commands
            .spawn_bundle(SpriteBundle{
                texture: asset_server.load("blue-marble.png"),
                transform: Transform::from_xyz(y * TILE_SIZE, -x * TILE_SIZE, 1.),
                ..default()
            });
        blue
            .insert(Marble{ index: BOARD.len() })
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
            ..default()
        })
        .insert(DieAnimationTimer(Timer::from_seconds(0.1, true)))
        .id()
        ;
    let die_2 = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: die_sheet_handle.clone(),
            visibility: Visibility{ is_visible: false },
            ..default()
        })
        .insert(DieAnimationTimer(Timer::from_seconds(0.1, true)))
        .id()
        ;

    commands.insert_resource(DiceData{
        die_1,
        die_2,
        die_sheet_handle,
        die_1_side: roll_die(),
        die_2_side: roll_die(),
    });
}

// TOOD: when a human player's marble is clicked, send and event to highlight the tiles it can move to based on the current state of the dice
// TOOD: highlight marble when cursor hovers it's bounds

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game

fn next_player(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    marbles: Query<(Entity, &Player, Option<&CurrentPlayer>), With<Marble>>,
) {
    // move clockwise to the next player
    current_player_data.player = match current_player_data.player {
        Player::Red => Player::Green,
        Player::Green => Player::Blue,
        Player::Blue => Player::Yellow,
        Player::Yellow => Player::Red,
    };

    // update the marbles accordingly
    for (marble, color, current_player) in marbles.iter() {
        if current_player.is_some() {
            commands.entity(marble).remove::<CurrentPlayer>();
        }
        if *color == current_player_data.player {
            commands.entity(marble).insert(CurrentPlayer);
        }
    }

    state.set(GameState::TurnSetup).unwrap();
}

fn turn_setup(
    mut dice_data: ResMut<DiceData>,
    current_player_data: Res<CurrentPlayerData>,
    mut dice: Query<(&mut Visibility, &mut Transform, &mut DieAnimationTimer)>,
) {
    let (d1_loc, d2_loc) = match current_player_data.player {
        Player::Red    => ((-3.0,  5.5), (-5.0,  5.5)),
        Player::Green  => (( 5.5,  3.0), ( 5.5,  5.0)),
        Player::Blue   => (( 3.0, -5.5), ( 5.0, -5.5)),
        Player::Yellow => ((-5.5, -3.0), (-5.5, -5.0)),
    };

    let (mut visibility, mut transform, mut die_animation_timer) = dice.get_mut(dice_data.die_1).expect("Unable to get die 1");
    visibility.is_visible = true;
    transform.translation.x = d1_loc.0 * TILE_SIZE;
    transform.translation.y = d1_loc.1 * TILE_SIZE;
    die_animation_timer.0.reset();

    let (mut visibility, mut transform, mut die_animation_timer) = dice.get_mut(dice_data.die_2).expect("Unable to get dice 2");
    visibility.is_visible = true;
    transform.translation.x = d2_loc.0 * TILE_SIZE;
    transform.translation.y = d2_loc.1 * TILE_SIZE;
    die_animation_timer.0.reset();

    dice_data.die_1_side = roll_die();
    dice_data.die_2_side = roll_die();

    println!("2b. calc possible moves for current player's marbles"); // with system after this one
}

fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
) {
    // !!!!! BIG TODO !!! - we must calculate possible moves twice since one move can change the possible moves for the
    //                      next move if the player uses the dice independently

    let mut possible_moves = std::collections::BTreeSet::new(); // so we disregard duplicates
    for (entity, marble) in marbles.iter() {
        for side in [dice_data.die_1_side, dice_data.die_2_side, dice_data.die_1_side + dice_data.die_2_side] {
            // exit base / enter board - only one possible move for this marble
            if marble.index == BOARD.len() {
                if side == 1 {
                    possible_moves.insert((entity, START_INDEX));
                }
                continue;
            }

            // exit center space - only one possible move for this marble
            if marble.index == CENTER_INDEX {
                if side == 1 {
                    possible_moves.insert((entity, CENTER_EXIT_INDEX));
                }
                continue;
            }

            // basic move
            let next_index = marble.index + side as usize;
            if next_index <= LAST_HOME_INDEX { // the very last home position
                possible_moves.insert((entity, next_index));
            }

            // enter center space
            if CENTER_ENTRANCE_INDEXES.contains(&(next_index - 1)) {
                possible_moves.insert((entity, CENTER_INDEX));
            }
        }
    }

    // remove possible moves that violate "self-hop" rule per marble
    for (entity_a, marble_a) in marbles.iter() {
        for (entity_b, marble_b) in marbles.iter() {
            if entity_a == entity_b {
                continue;
            }
            possible_moves = possible_moves.into_iter().filter(|(_, next_index)| {
                marble_b.index != *next_index && !((marble_a.index + 1)..=*next_index).contains(&marble_b.index)
            }).collect();
        }
    }

    current_player_data.possible_moves = possible_moves;
}

fn roll_animation(
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut DieAnimationTimer, &mut TextureAtlasSprite)>,
) {
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_sheet.rs
    for (mut die_animation_timer, mut sprite) in query.iter_mut() {
        if die_animation_timer.0.tick(time.delta()).just_finished() {
            sprite.index = (roll_die() - 1) as usize;
        }
    }

    // TODO: also rotate the dice

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
    mut query: Query<&mut TextureAtlasSprite>,
    dice_data: Res<DiceData>,
) {
    let mut sprite = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (dice_data.die_1_side - 1) as usize;

    let mut sprite = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (dice_data.die_2_side - 1) as usize;
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
