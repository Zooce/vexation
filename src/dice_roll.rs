use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub struct DiceRollPlugin;

impl Plugin for DiceRollPlugin {
    fn build(&self, app: &mut App) {
        app  
            // dice roll
            .add_systems(
                OnEnter(GameState::DiceRoll),
                (undim_dice, roll_dice),
            )
            .add_systems(Update, roll_animation
                .run_if(in_state(GameState::DiceRoll))
            )
            .add_systems(OnExit(GameState::DiceRoll),
                stop_roll_animation
            )
        ;
    }
}

fn undim_dice(
    mut commands: Commands,
    mut dice_sprite_query: Query<(Entity, &mut TextureAtlasSprite), With<UsedDie>>,
) {
    for (die, mut sprite) in dice_sprite_query.iter_mut() {
        commands.entity(die).remove::<UsedDie>();
        sprite.color = Color::WHITE;
    }
}

fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

fn roll_dice(
    mut dice_data: ResMut<DiceData>,
    game_data: Res<GameData>,
    current_player_data: Res<CurrentPlayerData>,
) {
    let player_data = game_data.players.get(&current_player_data.player).unwrap();
    let (d1, d2) = loop {
        let (a, b) = (roll_die(), roll_die());
        // before accepting the roll, make sure the player get's a move if they
        // haven't been able to play for two entire turns
        if player_data.consecutive_empty_turns < 2 || a == 1 || b == 1 {
            break (a, b);
        }
    };
    dice_data.dice = Dice::new(d1, d2);
}

fn roll_animation(
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut Die, &mut Transform, &mut TextureAtlasSprite)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    const DIE_MOVE_SPEED: f32 = 500.;

    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_sheet.rs
    for (mut die, mut transform, mut sprite) in &mut query {
        // advance the sprite sheet
        if die.timer.tick(time.delta()).just_finished() {
            sprite.index = (roll_die() - 1) as usize;
        }
        // move the dice
        if die.location != transform.translation {
            let dir = (die.location - transform.translation).normalize();
            transform.translation.x += dir.x * DIE_MOVE_SPEED * time.delta_seconds();
            transform.translation.y += dir.y * DIE_MOVE_SPEED * time.delta_seconds();

            let new_dir = (die.location - transform.translation).normalize();
            if new_dir.dot(dir) < 0.0 {
                transform.translation = die.location;
            }
        }
        // rotate the dice - `from_rotation_z` its argument as radians
        transform.rotate(Quat::from_rotation_z(16.0 * time.delta_seconds()));
    }

    if roll_animation_timer.0.tick(time.delta()).just_finished() {
        roll_animation_timer.0.reset();
        next_state.set(GameState::TurnSetup);
    }

    // TODO: create a 'roll buffer' timer so after the 'roll timer' stops, we have a second to see what the dice roll was before letting the player pick a move
}

fn stop_roll_animation(
    mut query: Query<(&mut TextureAtlasSprite, &mut Transform)>,
    dice_data: Res<DiceData>,
) {
    let (mut sprite, mut transform) = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (dice_data.dice.one.unwrap() - 1) as usize;
    transform.rotation = Quat::from_rotation_z(0.0);

    let (mut sprite, mut transform) = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (dice_data.dice.two.unwrap() - 1) as usize;
    transform.rotation = Quat::from_rotation_z(0.0);
}
