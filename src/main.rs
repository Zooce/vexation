use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use bevy::input::keyboard::KeyboardInput;
use bevy::ecs::schedule::ShouldRun;

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
    (-6, 1), // 0: start
    (-5, 1),
    (-4, 1),
    (-3, 1),
    (-2, 1),

    (-1, 1), // 5: shortcut entrance

    (-1, 2),
    (-1, 3),
    (-1, 4),
    (-1, 5),
    (-1, 6),

    (0, 6),

    (1, 6),
    (1, 5),
    (1, 4),
    (1, 3),
    (1, 2),

    (1, 1), // 17: shortcut entrance

    (2, 1),
    (3, 1),
    (4, 1),
    (5, 1),
    (6, 1),

    (6, 0),

    (6, -1),
    (5, -1),
    (4, -1),
    (3, -1),
    (2, -1),

    (1, -1), // 29: shortcut entrance

    (1, -2),
    (1, -3),
    (1, -4),
    (1, -5),
    (1, -6),

    (0, -6),

    (-1, -6),
    (-1, -5),
    (-1, -4),
    (-1, -3),
    (-1, -2),

    (-1, -1),

    (-2, -1),
    (-3, -1),
    (-4, -1),
    (-5, -1),
    (-6, -1),

    (-6, 0), // 47: home entrance

    // 48-52: home
    (-5, 0),
    (-4, 0),
    (-3, 0),
    (-2, 0),
    (-1, 0),

    (0, 0), // 53: center
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
    DiceRoll,
    HumanTurn,
    ComputerTurn,
    ChooseMoves,
    ComputerChooseMoves,
    CheckWinner,
}

#[derive(Debug)]
pub struct DiceData {
    pub die_1: Entity,
    pub die_2: Entity,
    pub die_sheet_handle: Handle<TextureAtlas>,
    pub die_1_side: Option<u8>,
    pub die_2_side: Option<u8>,
}

impl DiceData {
    pub fn get_dice_values(&self) -> Vec<u8> {
        match (self.die_1_side, self.die_2_side) {
            (Some(d1), Some(d2)) => vec![d1, d2, d1 + d2],
            (Some(d1), None) => vec![d1],
            (None, Some(d2)) => vec![d2],
            _ => vec![],
        }
    }
}

fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

#[derive(Component)]
pub struct DieAnimationTimer(Timer);

pub struct RunMoveCalculation(bool);

#[derive(Component, Debug, Eq, PartialEq, Clone)]
pub enum Player {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Player {
    pub fn rotate(&self, coords: (f32, f32)) -> (f32, f32) {
        match self {
            Player::Red => coords,
            Player::Green => (coords.1, -coords.0),
            Player::Blue => (-coords.0, -coords.1),
            Player::Yellow => (-coords.1, coords.0),
        }
    }
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

#[derive(Debug)]
pub struct CurrentPlayerData {
    player: Player,
    possible_moves: BTreeSet<(Entity, usize)>,
}

impl CurrentPlayerData {
    pub fn get_moves(&self, marble: Entity) -> Vec<usize> {
        self.possible_moves.iter()
            .filter_map(|(e, i)| {
                if *e == marble {
                    Some(*i)
                } else {
                    None
                }
        }).collect()
    }
}

pub struct HumanPlayer {
    color: Player,
}

pub struct RollAnimationTimer(Timer);

#[derive(Component)]
pub struct Marble {
    index: usize,
    can_move: bool,
}

/// The resource for selection data.
pub struct SelectionData {
    /// The marble that is currently selected
    marble: Option<Entity>,
    /// The highlight texture for the selected marble and its possible moves
    highlight_texture: Handle<Image>,
}

/// Used to mark the highlight sprites when a marble is selected, so we can
/// later remove them when a marble is no longer selected.
#[derive(Component)]
pub struct Highlight(Entity);

/// Event to inform the selection system that it needs to highlight things.
pub struct SelectionEvent(Vec3);

/// Event to inform the selection system that it needs to remove highlights.
pub struct DeselectionEvent;

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
            .insert_resource(RunMoveCalculation(false))
            .insert_resource(HumanPlayer{ color: Player::Blue }) // TODO: insert this after human chooses their color

            .add_event::<SelectionEvent>()
            .add_event::<DeselectionEvent>()

            .add_startup_system(setup)

            .add_state(GameState::NextPlayer)

            // TODO: define ChooseColor state

            .add_system_set(SystemSet::on_enter(GameState::NextPlayer)
                .with_system(choose_next_player)
                .with_system(next_player_setup.after(choose_next_player))
            )

            .add_system_set(SystemSet::on_enter(GameState::DiceRoll).with_system(roll_dice))
            .add_system_set(SystemSet::on_update(GameState::DiceRoll).with_system(roll_animation))
            .add_system_set(SystemSet::on_exit(GameState::DiceRoll).with_system(stop_roll_animation))

            .add_system_set(SystemSet::new().with_run_criteria(can_calc_moves).with_system(calc_possible_moves))

            .add_system_set(SystemSet::on_update(GameState::HumanTurn)
                .with_system(handle_mouse_clicks)
                .with_system(handle_keyboard_input)
                .with_system(handle_selection_events)
                .with_system(handle_deselection_events)
                //.with_system(handle_move_events)
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

    // pick the first player randomly
    let current_player: Player = Player::Green; // TODO: ((roll_die() - 1) % 4).into();
    commands.insert_resource(CurrentPlayerData{
        player: current_player.clone(),
        possible_moves: BTreeSet::new(),
    });

    // marbles
    let red_marble = asset_server.load("red-marble.png");
    let green_marble = asset_server.load("green-marble.png");
    let blue_marble = asset_server.load("blue-marble.png");
    let yellow_marble = asset_server.load("yellow-marble.png");
    for (x, y) in vec![(3., 3.5), (3., 4.5), (4., 3.), (4., 4.), (4., 5.)] {
        // green marbles
        let mut green = commands
            .spawn_bundle(SpriteBundle{
                texture: green_marble.clone(),
                transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, 1.),
                ..default()
            });
        green
            .insert(Marble{ index: BOARD.len(), can_move: true })
            .insert(Player::Green)
            ;
        if current_player == Player::Green {
            green.insert(CurrentPlayer);
        }
        // yellow marbles
        let mut yellow = commands
            .spawn_bundle(SpriteBundle{
                texture: yellow_marble.clone(),
                transform: Transform::from_xyz(-x * TILE_SIZE, -y * TILE_SIZE, 1.),
                ..default()
            });
        yellow
            .insert(Marble{ index: BOARD.len(), can_move: true })
            .insert(Player::Yellow)
            ;
        if current_player == Player::Yellow {
            yellow.insert(CurrentPlayer);
        }
        // red marbles
        let mut red = commands
            .spawn_bundle(SpriteBundle{
                texture: red_marble.clone(),
                transform: Transform::from_xyz(-y * TILE_SIZE, x * TILE_SIZE, 1.),
                ..default()
            });
        red
            .insert(Marble{ index: BOARD.len(), can_move: true })
            .insert(Player::Red)
            ;
        if current_player == Player::Red {
            red.insert(CurrentPlayer);
        }
        // blue marbles
        let mut blue = commands
            .spawn_bundle(SpriteBundle{
                texture: blue_marble.clone(),
                transform: Transform::from_xyz(y * TILE_SIZE, -x * TILE_SIZE, 1.),
                ..default()
            });
        blue
            .insert(Marble{ index: BOARD.len(), can_move: true })
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

    commands.insert_resource(DiceData {
        die_1,
        die_2,
        die_sheet_handle,
        die_1_side: Some(roll_die()),
        die_2_side: Some(roll_die()),
    });

    // highlight data
    commands.insert_resource(SelectionData{
        marble: None,
        highlight_texture: asset_server.load("tile-highlight.png"),
    })
}

// TOOD: when a human player's marble is clicked, send and event to highlight the tiles it can move to based on the current state of the dice
// TOOD: highlight marble when cursor hovers it's bounds

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game

// ----------------------------------------------------------------------------- NextPlayer

fn choose_next_player(
    mut commands: Commands,
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

    println!("choose_next_player: {:?}", current_player_data.player);
}

fn next_player_setup(
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
    current_player_data: Res<CurrentPlayerData>,
    mut dice: Query<(&mut Visibility, &mut Transform)>,
    mut marbles: Query<&mut Marble>,
) {
    let (d1_loc, d2_loc) = match current_player_data.player {
        Player::Red    => ((-3.0,  5.5), (-5.0,  5.5)),
        Player::Green  => (( 5.5,  3.0), ( 5.5,  5.0)),
        Player::Blue   => (( 3.0, -5.5), ( 5.0, -5.5)),
        Player::Yellow => ((-5.5, -3.0), (-5.5, -5.0)),
    };

    let (mut visibility, mut transform) = dice.get_mut(dice_data.die_1).expect("Unable to get die 1");
    visibility.is_visible = true;
    transform.translation.x = d1_loc.0 * TILE_SIZE;
    transform.translation.y = d1_loc.1 * TILE_SIZE;

    let (mut visibility, mut transform) = dice.get_mut(dice_data.die_2).expect("Unable to get dice 2");
    visibility.is_visible = true;
    transform.translation.x = d2_loc.0 * TILE_SIZE;
    transform.translation.y = d2_loc.1 * TILE_SIZE;

    marbles.for_each_mut(|mut m| m.can_move = true);

    state.set(GameState::DiceRoll).unwrap();

    println!("next_player_setup");
}

// ----------------------------------------------------------------------------- DiceRoll

fn roll_dice(
    mut dice_data: ResMut<DiceData>,
    mut die_animation_timers: Query<&mut DieAnimationTimer>,
    mut run_move_calc: ResMut<RunMoveCalculation>,
) {
    dice_data.die_1_side = Some(roll_die());
    dice_data.die_2_side = Some(roll_die());

    die_animation_timers.for_each_mut(|mut t| t.0.reset());

    run_move_calc.0 = true;

    println!("roll_dice: {:?} {:?}", dice_data.die_1_side, dice_data.die_2_side);
}

fn roll_animation(
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut DieAnimationTimer, &mut TextureAtlasSprite)>,
    mut state: ResMut<State<GameState>>,
    human_player: Res<HumanPlayer>,
    current_player_data: Res<CurrentPlayerData>,
) {
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_sheet.rs
    for (mut die_animation_timer, mut sprite) in query.iter_mut() {
        if die_animation_timer.0.tick(time.delta()).just_finished() {
            sprite.index = (roll_die() - 1) as usize;
        }
    }

    // TODO: also rotate the dice

    if roll_animation_timer.0.tick(time.delta()).just_finished() {
        println!("roll_animation timer expired");
        roll_animation_timer.0.reset();
        if human_player.color == current_player_data.player {
            state.set(GameState::HumanTurn).unwrap();
        } else {
            state.set(GameState::ComputerTurn).unwrap();
        }
    }
}

fn stop_roll_animation(
    mut query: Query<&mut TextureAtlasSprite>,
    dice_data: Res<DiceData>,
) {
    let mut sprite = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (dice_data.die_1_side.unwrap() - 1) as usize;

    let mut sprite = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (dice_data.die_2_side.unwrap() - 1) as usize;

    println!("stop_roll_animation");
}

// ----------------------------------------------------------------------------- Move Calculation

fn can_calc_moves(run_calc: Res<RunMoveCalculation>) -> ShouldRun {
    if run_calc.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn calc_possible_moves(
    dice_data: Res<DiceData>,
    marbles: Query<(Entity, &Marble), With<CurrentPlayer>>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    mut run_move_calc: ResMut<RunMoveCalculation>,
) {
    let mut possible_moves = std::collections::BTreeSet::new(); // so we disregard duplicates
    for (entity, marble) in marbles.iter() {
        if !marble.can_move {
            continue;
        }

        for side in dice_data.get_dice_values() { // TODO: need to check if we've already used one or both dice in the previous move
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
    for (marble, other_marble) in marbles.iter().zip(marbles.iter()) {
        if marble.0 == other_marble.0 || marble.1.index > other_marble.1.index {
            continue;
        }
        // remove possible moves where either:
        // - marble_a lands on marble_b
        // - marble_b is between marble_a's current position and the destination
        possible_moves = possible_moves.into_iter().filter(|(_, next_index)| {
            *next_index != other_marble.1.index && !(marble.1.index..*next_index).contains(&other_marble.1.index)
        }).collect();
    }

    current_player_data.possible_moves = possible_moves;

    run_move_calc.0 = false;

    println!("calc_possible_moves");
}

// ----------------------------------------------------------------------------- Move Execution

fn handle_mouse_clicks(
    mut mouse_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    marbles: Query<(Entity, &Transform), (With<Marble>, With<CurrentPlayer>)>,
    mut selection_events: EventWriter<SelectionEvent>,
    mut deselection_events: EventWriter<DeselectionEvent>,
    mut selection_data: ResMut<SelectionData>,
    current_player_data: Res<CurrentPlayerData>,
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
        let mut did_select_marble = false;
        for (entity, transform) in marbles.iter() {
            let selected = cursor_x > transform.translation.x - TILE_SIZE / 2.
                        && cursor_x < transform.translation.x + TILE_SIZE / 2.
                        && cursor_y > transform.translation.y - TILE_SIZE / 2.
                        && cursor_y < transform.translation.y + TILE_SIZE / 2.;
            if selected {
                did_select_marble = true;
                selection_data.marble = Some(entity);
                selection_events.send(SelectionEvent(transform.translation.clone()));
                println!("handle_mouse_clicks: clicked on marble!");
            } else {
                deselection_events.send(DeselectionEvent);
            }
        }
        if !did_select_marble {
            // first check if we selected a destination
            if let Some(marble) = selection_data.marble {
                let (x, y) = (snap(cursor_x), snap(cursor_y));
                let (col, row) = current_player_data.player.rotate((x / TILE_SIZE, y / TILE_SIZE));
                if let Some(board_index) = BOARD.into_iter().position(|coord| coord == (col as i32, row as i32)) {
                    if current_player_data.get_moves(marble).contains(&board_index) {
                        println!("destination clicked!");
                        // TODO: send MoveMarbleEvent((x, y))
                    }
                }
            }
            selection_data.marble = None;
        }
    }
}

fn handle_keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut deselection_events: EventWriter<DeselectionEvent>,
    mut selection_data: ResMut<SelectionData>,
) {
    for event in keyboard_events.iter() {
        match event.key_code {
            Some(KeyCode::Escape) => {
                selection_data.marble = None;
                deselection_events.send(DeselectionEvent);
            }
            _ => return,
        }
    }
}

fn handle_deselection_events(
    mut commands: Commands,
    mut deselection_events: EventReader<DeselectionEvent>,
    entities: Query<Entity, With<Highlight>>,
    selection_data: Res<SelectionData>,
) {
    if deselection_events.iter().last().is_some() {
        for entity in entities.iter() {
            match selection_data.marble {
                // marble selected - remove highlights not related to the selected marble
                Some(marble) => if entity != marble {
                    commands.entity(entity).despawn();
                }
                // no marbles selected - remove all highlights
                None => commands.entity(entity).despawn(),
            }
        }
    }
}

fn handle_selection_events(
    mut commands: Commands,
    mut selection_events: EventReader<SelectionEvent>,
    selection_data: Res<SelectionData>,
    current_player_data: Res<CurrentPlayerData>,
) {
    if let Some(selection) = selection_events.iter().last() {
        let mut t = selection.0.clone();
        t.z += 1.0; // make sure it's drawn on top

        // create a sprite located at the same location as the marble entity
        commands.spawn_bundle(SpriteBundle{
            texture: selection_data.highlight_texture.clone(),
            transform: Transform::from_translation(t),
            ..default()
        })
        .insert(Highlight(selection_data.marble.unwrap()))
        ;

        // create sprites located at the possible moves for the selected marble
        for board_index in current_player_data.get_moves(selection_data.marble.unwrap()) {
            let tile = BOARD[board_index];
            let (x, y) = current_player_data.player.rotate((tile.0 as f32, tile.1 as f32));
            commands.spawn_bundle(SpriteBundle{
                texture: selection_data.highlight_texture.clone(),
                transform: Transform::from_xyz(x * TILE_SIZE, y * TILE_SIZE, t.z),
                ..default()
            })
            .insert(Highlight(selection_data.marble.unwrap()))
            ;
        }
    }
}

// fn select_destination() {
//     // let (col, row) = ((cursor.x / WINDOW_SIZE * TILE_COUNT).floor(), (cursor.y / WINDOW_SIZE * TILE_COUNT).floor());
// }

// fn check_end_turn(dice_data: Res<DiceData>) {
//     if dice_data.get_dice_values().is_empty() {
//         // go to the choose_next_player state
//     } else {
//         // go to the calc_moves state
//     }
// }

/// Snaps the given coordinate to the center of the tile it's inside of.
fn snap(coord: f32) -> f32 {
    // let's only deal with positive values for now
    let c = coord.abs();
    // how far away is the coordinate from the center of the tile
    let remainder = c % TILE_SIZE;
    let result = if remainder < TILE_SIZE / 2. {
        // if the coordinate is past the center (going away from the origin)
        // then snap it back to the center
        // |    X     |
        // |    <---c |
        c - remainder
    } else {
        // otherwise shift the coordinate to the next tile (going away from the
        // origin) then snap it back to the center
        // |    X    |
        // | c-------|->
        // |    <----|-c
        let shift = c + TILE_SIZE;
        shift - (shift % TILE_SIZE)
    };
    // just flip the result if the original coordinate was negative
    if coord < 0.0 && result > 0.0 {
        result * -1.0
    } else {
        result
    }
}
