// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn choose_next_player(
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

pub fn next_player_setup(
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
    current_player_data: Res<CurrentPlayerData>,
    mut dice: Query<(&mut Visibility, &mut Transform)>,
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

    state.set(GameState::DiceRoll).unwrap();

    println!("next_player_setup");
}
