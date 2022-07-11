use bevy::prelude::*;
use bevy::app::AppExit;

use crate::resources::{GameState, MainMenuEntities};

const NORMAL_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_COLOR: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(GameState::MainMenu) // the starting state

            .add_system_set(SystemSet::on_enter(GameState::MainMenu)
                .with_system(create)
            )
            .add_system_set(SystemSet::on_update(GameState::MainMenu)
                .with_system(interact)
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu)
                .with_system(cleanup)
            )
            ;
    }
}

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct QuitButton;

pub fn create(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // ui camera
    let camera = commands.spawn_bundle(UiCameraBundle::default()).id();

    let font = asset_server.load("FiraCode-Bold.ttf");

    use crate::utils::ui;
    let ui = commands
        .spawn_bundle(ui::vertical_container(Color::rgba(0.0, 0.0, 0.0, 0.3).into()))
        .with_children(|root| {
            root
                .spawn_bundle(ui::vertical_container(Color::NONE.into()))
                .with_children(|button_container| {
                    // play button
                    button_container
                        .spawn_bundle(ui::button_bundle(NORMAL_COLOR.into()))
                        .with_children(|parent| {
                            parent.spawn_bundle(ui::text_bundle("Play", font.clone(), 40.0));
                        })
                        .insert(PlayButton)
                        ;

                    // quit button
                    button_container
                        .spawn_bundle(ui::button_bundle(NORMAL_COLOR.into()))
                        .with_children(|parent| {
                            parent.spawn_bundle(ui::text_bundle("Quit", font.clone(), 40.0));
                        })
                        .insert(QuitButton)
                        ;
                });
        })
        .id()
        ;

    commands.insert_resource(MainMenuEntities{ camera, ui });
}

pub fn interact(
    mut state: ResMut<State<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    play_button: Query<Entity, With<PlayButton>>,
    quit_button: Query<Entity, With<QuitButton>>,
) {
    for (entity, interaction, mut ui_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                ui_color.0 = PRESSED_COLOR.into();
                if entity == play_button.single() {
                    state.set(GameState::GameStart).unwrap(); // TODO: set game state to GameCreate
                } else if entity == quit_button.single() {
                    app_exit_events.send(AppExit);
                }
            }
            Interaction::Hovered => {
                ui_color.0 = HOVERED_COLOR.into();
            }
            Interaction::None => {
                ui_color.0 = NORMAL_COLOR.into();
            }
        }
    }
}

pub fn cleanup(
    mut commands: Commands,
    menu_entities: Res<MainMenuEntities>,
) {
    commands.entity(menu_entities.ui).despawn_recursive();
    commands.entity(menu_entities.camera).despawn();
}
