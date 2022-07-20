use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

pub mod ui {
    use bevy::prelude::*;
    use bevy::ui::FocusPolicy;
    use crate::components::*;
    use crate::resources::*;

    pub fn spawn_button(
        parent: &mut ChildBuilder,
        ui_assets: &UiAssets,
        button_text: &str,
        action: ButtonAction,
    ) {
        parent
            // button root
            .spawn_bundle(ButtonBundle{
                style: Style{
                    align_self: AlignSelf::Center,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    size: Size::new(Val::Px(190.0), Val::Px(49.0)),
                    margin: Rect::all(Val::Px(10.0)),
                    ..default()
                },
                image: ui_assets.normal_button.clone().into(),
                focus_policy: FocusPolicy::Block,
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle{
                    text: Text::with_section(
                        button_text,
                        TextStyle{
                            font: ui_assets.font.clone(),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default()
                    ),
                    ..default()
                })
                ;
            })
            .insert(action)
            ;
    }

    pub fn vertical_node_bundle() -> NodeBundle {
        NodeBundle{
            style: Style{
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }
    }

    pub fn horizontal_node_bundle() -> NodeBundle {
        NodeBundle{
            style: Style{
                size: Size::new(Val::Percent(100.0), Val::Auto),
                margin: Rect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }
    }
}
