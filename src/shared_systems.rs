// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;

pub fn animate_marble_moves(
    mut commands: Commands,
    time: Res<Time>,
    mut moving_marbles: Query<(Entity, &Moving, &mut Transform)>,
) {
    const PIXELS_PER_SEC: f32 = 650.;

    if let Ok((entity, moving, mut transform)) = moving_marbles.get_single_mut() {
        transform.translation.x += moving.direction.x * PIXELS_PER_SEC * time.delta_seconds();
        transform.translation.y += moving.direction.y * PIXELS_PER_SEC * time.delta_seconds();

        // we've arrived if the direction to the destination has flipped, meaning we've passed it
        let arrived = {
            let d = (moving.destination - transform.translation).normalize();
            Vec2::new(d.x, d.y).dot(moving.direction) == -1.0
        };

        if arrived {
            transform.translation = moving.destination; // just to make sure it's perfect
            commands.entity(entity).remove::<Moving>();
        }
    }
}
