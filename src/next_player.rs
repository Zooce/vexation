use bevy::prelude::*;
use crate::buttons::ButtonState;
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
    for (marble, color, current_player) in &marbles {
        if current_player.is_some() {
            commands.entity(marble).remove::<CurrentPlayer>();
        }
        if *color == current_player_data.player {
            commands.entity(marble).insert(CurrentPlayer);
        }
    }
}

pub fn show_or_hide_buttons(
    mut button_query: Query<(&mut Visibility, &mut TextureAtlasSprite, &mut ButtonState)>,
    human_player: Res<HumanPlayer>,
    current_player_data: Res<CurrentPlayerData>,
) {
    for (mut visibility, mut sprite, mut state) in &mut button_query {
        visibility.is_visible = human_player.color == current_player_data.player;
        sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.4);
        sprite.index = 0;
        *state = ButtonState::NotHovered;
    }
}

pub fn next_player_setup(
    mut state: ResMut<State<GameState>>,
    dice_data: Res<DiceData>,
    current_player_data: Res<CurrentPlayerData>,
    mut dice: Query<(&mut Visibility, &mut Die)>,
) {
    let (d1_loc, d2_loc) = match current_player_data.player {
        Player::Red    => ((-3.0,  5.0), (-5.0,  5.0)),
        Player::Green  => (( 5.0,  3.0), ( 5.0,  5.0)),
        Player::Blue   => (( 3.0, -5.0), ( 5.0, -5.0)),
        Player::Yellow => ((-5.0, -3.0), (-5.0, -5.0)),
    };

    let (mut visibility, mut die) = dice.get_mut(dice_data.die_1).expect("Unable to get die 1");
    visibility.is_visible = true;
    die.location.x = d1_loc.0 * TILE_SIZE;
    die.location.y = d1_loc.1 * TILE_SIZE;
    die.timer.reset();

    let (mut visibility, mut die) = dice.get_mut(dice_data.die_2).expect("Unable to get dice 2");
    visibility.is_visible = true;
    die.location.x = d2_loc.0 * TILE_SIZE;
    die.location.y = d2_loc.1 * TILE_SIZE;
    die.timer.reset();

    state.set(GameState::DiceRoll).unwrap();
}
