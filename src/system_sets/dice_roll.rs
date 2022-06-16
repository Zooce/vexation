// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::*;

pub fn roll_dice(
    mut dice_data: ResMut<DiceData>,
    mut die_animation_timers: Query<&mut DieAnimationTimer>,
) {
    dice_data.die_1_side = Some(1); // TODO: Some(roll_die());
    dice_data.die_2_side = Some(roll_die());

    die_animation_timers.for_each_mut(|mut t| t.0.reset());

    println!("roll_dice: {:?} {:?}", dice_data.die_1_side, dice_data.die_2_side);
}

pub fn roll_animation(
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut DieAnimationTimer, &mut TextureAtlasSprite)>,
    mut state: ResMut<State<GameState>>,
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
        state.set(GameState::TurnSetup).unwrap();
    }
}

pub fn stop_roll_animation(
    mut query: Query<&mut TextureAtlasSprite>,
    dice_data: Res<DiceData>,
) {
    let mut sprite = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (dice_data.die_1_side.unwrap() - 1) as usize;

    let mut sprite = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (dice_data.die_2_side.unwrap() - 1) as usize;
}