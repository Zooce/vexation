use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::input::mouse::{MouseButtonInput, MouseButton};

const TILE_SIZE: f32 = 32.;
const TILE_COUNT: f32 = 17.;
const WINDOW_SIZE: f32 = TILE_SIZE * TILE_COUNT;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    // ChooseColor,
    NextPlayer,
    TurnSetup,
    ChooseMoves,
    ComputerChooseMoves,
    CheckWinner,
}

fn main() {
    App::new()
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
            .add_startup_system(setup)
            .insert_resource(RollAnimationTimer(Timer::from_seconds(3., true)))

            .add_state(GameState::NextPlayer)

            // prototyping turn-based systems
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
}

// TOOD: when a human player's marble is clicked, send and event to highlight the tiles it can move to based on the current state of the dice
// TOOD: highlight marble when cursor hovers it's bounds

// TODO: consider using https://github.com/IyesGames/iyes_loopless to organize this turn-based game

fn next_player(mut state: ResMut<State<GameState>>) {
    println!("1. next_player");

    state.set(GameState::TurnSetup).unwrap();
}

fn turn_setup() {
    println!("2a. choose dice values randomly");
    println!("2b. calc possible moves for current player's marbles");
}

fn roll_animation(mut state: ResMut<State<GameState>>, time: Res<Time>, mut roll_animation_timer: ResMut<RollAnimationTimer>) {
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_sheet.rs
    println!("3. roll animation for X seconds");

    if roll_animation_timer.0.tick(time.delta()).just_finished() {
        // after the animation has run for X seconds, go to the ChooseMoves state
        state.set(GameState::ChooseMoves).unwrap();
    }
}

fn stop_roll_animation() {
    println!("4. stop roll animation on sprite sheet index corresponding to randomly chosen dice valuse");
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
