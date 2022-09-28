use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::*;

pub fn undim_dice(
    mut commands: Commands,
    mut dice_sprite_query: Query<(Entity, &mut TextureAtlasSprite), With<UsedDie>>,
) {
    for (die, mut sprite) in dice_sprite_query.iter_mut() {
        commands.entity(die).remove::<UsedDie>();
        sprite.color = Color::WHITE;
    }
}

pub fn roll_dice(
    mut dice_data: ResMut<DiceData>,
) {
    let (d1, d2) = (roll_die(), roll_die());
    dice_data.doubles = d1 == d2;
    dice_data.die_1_side = Some(d1);
    dice_data.die_2_side = Some(d2);

    println!("{:?} and {:?}", d1, d2);
}

pub fn roll_animation(
    time: Res<Time>,
    mut roll_animation_timer: ResMut<RollAnimationTimer>,
    mut query: Query<(&mut Die, &mut Transform, &mut TextureAtlasSprite)>,
    mut state: ResMut<State<GameState>>,
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
        // rotate the dice
        transform.rotate(Quat::from_rotation_z(16.0 * time.delta_seconds()));
    }

    if roll_animation_timer.0.tick(time.delta()).just_finished() {
        roll_animation_timer.0.reset();
        state.set(GameState::TurnSetup).unwrap();
    }

    // TODO: create a 'roll buffer' timer so after the 'roll timer' stops, we have a second to see what the dice roll was before letting the player pick a move
}

pub fn stop_roll_animation(
    mut query: Query<(&mut TextureAtlasSprite, &mut Transform)>,
    dice_data: Res<DiceData>,
) {
    let (mut sprite, mut transform) = query.get_mut(dice_data.die_1).expect("Unable to get die 1");
    sprite.index = (dice_data.die_1_side.unwrap() - 1) as usize;
    transform.rotation = Quat::from_rotation_z(0.0);

    let (mut sprite, mut transform) = query.get_mut(dice_data.die_2).expect("Unable to get die 2");
    sprite.index = (dice_data.die_2_side.unwrap() - 1) as usize;
    transform.rotation = Quat::from_rotation_z(0.0);
}
