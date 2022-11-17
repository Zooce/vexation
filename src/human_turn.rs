use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use crate::buttons::*;
use crate::components::*;
use crate::constants::*;
use crate::power::ActivatePowerUpEvent;
use crate::shared_systems::HighlightEvent;
use crate::resources::*;

#[derive(Debug)]
struct ClickEvent(pub Vec2);

struct MoveEvent(pub (Entity, usize, WhichDie, Vec3));

pub struct HumanTurnPlugin;

impl Plugin for HumanTurnPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ClickEvent>()
            .add_event::<MoveEvent>()
            
            .add_system_set(SystemSet::on_enter(GameState::HumanTurn)
                .with_system(enable_ui)
            )
            .add_system_set(SystemSet::on_update(GameState::HumanTurn)
                // temp
                .with_system(use_power_up_from_keyboard)
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
            ;
    }
}

fn enable_ui(
    mouse_button_inputs: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut button_query: Query<(&mut ButtonState, &mut TextureAtlasSprite, &Transform)>,
    mut app_exit_events: EventWriter<AppExit>, // FIXME: workaround for https://github.com/bevyengine/bevy/commit/07d576987a7f2bdcabc97fefcc043e19e1a30222
) {
    let cursor_pos = match windows.get_primary() {
        Some(w) => w.cursor_position(),
        None => {
            app_exit_events.send(AppExit);
            return;
        }
    };
    let mouse_pressed = mouse_button_inputs.pressed(MouseButton::Left);

    for (mut button_state, mut button_sprite, button_transform) in &mut button_query {
        *button_state = get_button_state(cursor_pos, button_transform.translation, mouse_pressed);
        button_sprite.color = Color::WHITE;
    }
}

fn disable_ui(
    mut button_query: Query<(&mut TextureAtlasSprite, &mut ButtonState)>,
) {
    for (mut sprite, mut state) in &mut button_query {
        sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.4);
        sprite.index = 0;
        *state = ButtonState::NotHovered;
    }
}

fn translate_mouse_input(
    windows: Res<Windows>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut click_events: EventWriter<ClickEvent>,
    mut app_exit_events: EventWriter<AppExit>, // FIXME: workaround for https://github.com/bevyengine/bevy/commit/07d576987a7f2bdcabc97fefcc043e19e1a30222
) {
    if mouse_button_input_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed())
        .last().is_some()
    {
        if let Some(cursor) = match windows.get_primary() {
            Some(w) => w.cursor_position(),
            None => {
                app_exit_events.send(AppExit);
                return;
            }
        } {
            let (x, y) = (cursor.x - WINDOW_SIZE / 2.0, cursor.y - WINDOW_SIZE / 2.0);
            // POWERUP: ignore this click if it's on a power-up button
            click_events.send(ClickEvent(Vec2::new(x, y)));
        }
    }
}

fn interpret_click_event(
    mut highlight_events: EventWriter<HighlightEvent>,
    mut move_events: EventWriter<MoveEvent>,
    mut click_events: EventReader<ClickEvent>,
    mut current_player_data: ResMut<CurrentPlayerData>,
    marbles_query: Query<(Entity, &Transform), (With<Marble>, With<CurrentPlayer>)>,
) {
    if let Some(click_event) = click_events.iter().last() {
        // interpret click as marble selection
        if let Some(marble) = marbles_query.iter().find_map(|(e, t)| {
                let found = click_event.0.x > t.translation.x - TILE_SIZE / 2.0 &&
                            click_event.0.x < t.translation.x + TILE_SIZE / 2.0 &&
                            click_event.0.y > t.translation.y - TILE_SIZE / 2.0 &&
                            click_event.0.y < t.translation.y + TILE_SIZE / 2.0;
                if found { Some(e) } else { None }
            })
        {
            if let Some(old_marble) = current_player_data.selected_marble {
                if old_marble == marble {
                    return; // ignore clicks on a marble that is already selected
                }
            }
            current_player_data.selected_marble = Some(marble);
            highlight_events.send(HighlightEvent::On);
        }
        // interpret click as move selection
        else if let Some(marble) = current_player_data.selected_marble {
            // to compare to board coordinates, we need to snap the click event to the center of a tile
            let (col, row) = (snap(click_event.0.x), snap(click_event.0.y));
            // find the move that corresponds to this click position
            let selected_move = match BOARD.into_iter().position(|(x, y)| {
                // rotate the board coordinates based on the current player
                let rot = current_player_data.player.rotate_coords((x as f32, y as f32));
                // find the board index that matches the click position
                rot == (col / TILE_SIZE, row / TILE_SIZE)
            }) {
                // find a move for this board index
                Some(clicked_board_index) => current_player_data
                    .get_moves(marble).into_iter().find(|MarbleMove{ destination, .. }| *destination == clicked_board_index),
                _ => None,
            };
            if let Some(MarbleMove{ destination, which, .. }) = selected_move {
                current_player_data.move_marble();
                move_events.send(MoveEvent((marble, destination, which, Vec3::new(col, row, 1.0))));
            } else {
                // deselect the marble if the click was anywhere unimportant
                current_player_data.selected_marble = None;
            }

            // since we didn't click on another marble and we no longer have a
            // selected marble then all highlights can be removed
            highlight_events.send(HighlightEvent::Off);
        }
        // POWERUP: ignore power up button clicks
    }
}

fn move_event_handler(
    mut commands: Commands,
    mut move_events: EventReader<MoveEvent>,
    mut marbles: Query<(Entity, &Transform, &mut Marble), With<CurrentPlayer>>,
    mut dice_data: ResMut<DiceData>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some(MoveEvent((e, idx, which, dest))) = move_events.iter().last() {
        let (e, t, mut m) = marbles.get_mut(*e).unwrap();
        m.update_index(*idx);
        dice_data.use_die(*which, &mut commands);
        commands.entity(e).insert(Moving::new(*dest, t.translation));
        state.set(GameState::WaitForAnimation).unwrap();
    }
}

fn execute_button_actions(
    mut action_events: EventReader<ActionEvent<GameButtonAction>>,
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
) {
    for action in action_events.iter() {
        match action.0 {
            GameButtonAction::Done => if dice_data.dice.doubles {
                state.set(GameState::DiceRoll).unwrap();
            } else {
                state.set(GameState::EndTurn).unwrap();
            }
        }
    }
}

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

fn use_power_up_from_keyboard(
    mut kb_events: EventReader<KeyboardInput>,
    mut game_data: ResMut<GameData>,
    current_player_data: Res<CurrentPlayerData>,
    mut pu_events: EventWriter<ActivatePowerUpEvent>,
) {
    if let Some(event) = kb_events.iter().last() {
        if !event.state.is_pressed() {
            return;
        }
        if let Some(keycode) = event.key_code {
            if keycode == KeyCode::D {
                println!("\n===---[ Debug Info ] ---===");
                for (p, d) in &game_data.players {
                    println!("{p:?}: {}", d.debug_info());
                }
                return;
            }
            let player_data = game_data.players.get_mut(&current_player_data.player).unwrap();
            if player_data.power_ups.is_empty() {
                return;
            }
            if let Some(power_up) = match keycode {
                KeyCode::Key1 => player_data.use_power_up(0),
                KeyCode::Key2 => player_data.use_power_up(1),
                KeyCode::Key3 => player_data.use_power_up(2),
                _ => None,
            } {
                pu_events.send(ActivatePowerUpEvent(power_up));
            }
        }
    }
}
