use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

pub mod ui {
    use bevy::prelude::*;
    use crate::components::*;

    pub fn spawn_sprite_sheet_button<T: Send + Sync + 'static>(
        parent: &mut ChildBuilder,
        texture_atlas: Handle<TextureAtlas>,
        transform: Transform,
        action: ButtonAction<T>,
        is_visible: bool,
    ) {
        println!("spawning sprite sheet @ {}", transform.translation);
        parent
            .spawn_bundle(SpriteSheetBundle{
                texture_atlas,
                transform,
                visibility: Visibility{ is_visible },
                ..default()
            })
            .insert(ButtonState::None)
            .insert(action)
            ;
    }
}
