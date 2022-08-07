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

    // TODO: create a builder for this sprite sheet button stuff

    pub fn spawn_sprite_sheet_button<T: Send + Sync + 'static>(
        parent: &mut ChildBuilder,
        texture_atlas: Handle<TextureAtlas>,
        transform: Transform,
        action: ButtonAction<T>,
        is_visible: bool,
        button_state: ButtonState,
    ) {
        parent
            .spawn_bundle(SpriteSheetBundle{
                sprite: TextureAtlasSprite {
                    index: match button_state {
                        ButtonState::None => 0,
                        ButtonState::Hovered => 1,
                        ButtonState::Pressed | ButtonState::PressedNotHovered => 2,
                    },
                    ..default()
                },
                texture_atlas,
                transform,
                visibility: Visibility{ is_visible },
                ..default()
            })
            .insert(button_state)
            .insert(action)
            ;
    }
}
