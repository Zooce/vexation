use bevy::prelude::*;
use bevy::app::AppExit;

use crate::resources::{GameState, MainMenuEntities, MainMenuAssets};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(GameState::MainMenu) // the starting state

            .add_system_set(SystemSet::on_enter(GameState::MainMenu)
                .with_system(create_main_menu)
            )
            .add_system_set(SystemSet::on_update(GameState::MainMenu)
                .with_system(interact_main_menu)
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu)
                .with_system(destroy_main_menu)
            )
            ;
    }
}

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct QuitButton;

pub fn create_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // ui camera
    let camera = commands.spawn_bundle(UiCameraBundle::default()).id();

    let main_menu_assets = MainMenuAssets{
        font: asset_server.load("Kenney Thick.ttf"),
        normal_button: asset_server.load("red_button11.png"),
        hovered_button: asset_server.load("red_button10.png"),
        pressed_button: asset_server.load("red_button12.png"),
    };

    let ui = commands
        .spawn_bundle(NodeBundle{
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle{
                    text: Text::with_section(
                        "Vexation",
                        TextStyle{
                            font: main_menu_assets.font.clone(),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default()
                    ),
                    style: Style{
                        margin: Rect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                })
                ;

            spawn_button(parent, &main_menu_assets, "Play", PlayButton);
            // spawn_button(button_container, &ui_assets, "Rules", PlayButton);
            spawn_button(parent, &main_menu_assets, "Quit", QuitButton);
        })
        .id()
        ;

    commands.insert_resource(main_menu_assets);
    commands.insert_resource(MainMenuEntities{ camera, ui });
}

pub fn interact_main_menu(
    mut state: ResMut<State<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiImage, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_text_query: Query<&mut Text>,
    main_menu_assets: Res<MainMenuAssets>,
    play_button: Query<Entity, With<PlayButton>>,
    quit_button: Query<Entity, With<QuitButton>>,
) {
    for (entity, interaction, mut ui_image, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                // println!("clicked");
                for child in children.iter() {
                    let mut text = button_text_query.get_mut(*child).unwrap();
                    text.sections[0].style.color = Color::WHITE;
                }
                ui_image.0 = main_menu_assets.pressed_button.clone().into();
                if entity == play_button.single() {
                    state.set(GameState::GameStart).unwrap(); // TODO: set game state to GameCreate
                } else if entity == quit_button.single() {
                    app_exit_events.send(AppExit);
                }
            }
            Interaction::Hovered => {
                // println!("hovered");
                for child in children.iter() {
                    let mut text = button_text_query.get_mut(*child).unwrap();
                    text.sections[0].style.color = Color::rgb_u8(232, 106, 23);
                }
                ui_image.0 = main_menu_assets.hovered_button.clone().into();
            }
            Interaction::None => {
                // println!("normal");
                for child in children.iter() {
                    let mut text = button_text_query.get_mut(*child).unwrap();
                    text.sections[0].style.color = Color::WHITE;
                }
                ui_image.0 = main_menu_assets.normal_button.clone().into();
            }
        }
    }
}

pub fn destroy_main_menu(
    mut commands: Commands,
    menu_entities: Res<MainMenuEntities>,
) {
    commands.entity(menu_entities.ui).despawn_recursive();
    commands.entity(menu_entities.camera).despawn();
}

fn spawn_button(
    parent: &mut ChildBuilder,
    ui_assets: &MainMenuAssets,
    button_text: &str,
    component: impl Component,
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
        .insert(component)
        ;
}
