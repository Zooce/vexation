use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

pub fn roll_die() -> u8 {
    let mut rng = thread_rng();
    let die = Uniform::new_inclusive(1u8, 6u8);
    rng.sample(die)
}

pub mod ui {
    use bevy::prelude::*;

    pub fn vertical_container(color: UiColor) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color,
            ..default()
        }
    }

    pub fn horizontal_container(color: UiColor) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            color,
            ..default()
        }
    }

    pub fn button_bundle(color: UiColor) -> ButtonBundle {
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color,
            ..default()
        }
    }

    pub fn text_container() -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }
    }

    pub fn text_bundle(text: &str, font: Handle<Font>, font_size: f32) -> TextBundle {
        TextBundle {
            text: Text::with_section(
                text,
                TextStyle {
                    font,
                    font_size,
                    color: Color::WHITE,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            ..default()
        }
    }
}
