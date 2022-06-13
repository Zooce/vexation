// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;

pub fn animate_marble_moves(
    mut commands: Commands,
    time: Res<Time>,
    mut moving_marbles: Query<(Entity, &mut Moving, &mut Transform)>,
) {
    for (entity, mut moving, mut transform) in moving_marbles.iter_mut() {
        // using a timer so at the end of the animation we force the translation
        // to be at the exact destination
        if moving.timer.tick(time.delta()).just_finished() {
            transform.translation = moving.destination;
            commands.entity(entity).remove::<Moving>();
            continue;
        }
        transform.translation += moving.anim_step_dist * time.delta_seconds();
    }
}
