use bevy::prelude::*;
use bevy::app::AppExit;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::shared_systems::*;
use crate::utils::ui;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(GameState::MainMenu) // the starting state
            .add_event::<ActionEvent<MainMenuAction>>()

            .insert_resource(UiPageNumber(0))

            .add_system_set(SystemSet::on_enter(GameState::MainMenu)
                .with_system(main_menu_enter)
            )
            .add_system_set(SystemSet::on_update(GameState::MainMenu)
                .with_system(execute_menu_action.before(mouse_watcher::<MainMenuAction>)) // I actually want a frame delay here so we can see the button "animation"
                .with_system(mouse_watcher::<MainMenuAction>)
                .with_system(watch_button_state_changes.after(mouse_watcher::<MainMenuAction>))

                .with_system(menu_page_renderer.after(watch_button_state_changes))
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu)
                .with_system(main_menu_exit)
            )
            ;
    }
}

fn main_menu_enter(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
) {
    let ui = create_main_menu(&mut commands, &ui_assets);
    let camera = commands.spawn_bundle(UiCameraBundle::default()).id();
    commands.insert_resource(RootUiEntities{ ui, camera });
}

fn main_menu_exit(
    mut commands: Commands,
    root_entities: Res<RootUiEntities>,
) {
    for entity in &root_entities.ui {
        commands.entity(*entity).despawn_recursive();
    }
    commands.entity(root_entities.camera).despawn();
    commands.remove_resource::<RootUiEntities>();
}

fn execute_menu_action(
    mut action_events: EventReader<ActionEvent<MainMenuAction>>,
    mut state: ResMut<State<GameState>>,
    mut page_number: ResMut<UiPageNumber>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for action in action_events.iter() {
        match action.0 {
            MainMenuAction::StartGame => state.set(GameState::GameStart).unwrap(),
            MainMenuAction::NextPage => page_number.0 += 1,
            MainMenuAction::PrevPage => page_number.0 -= 1,
            MainMenuAction::Quit => app_exit_events.send(AppExit),
        }
    }
}

/// Renders the current page in the menu if a page change occurred.
fn menu_page_renderer(
    page_number: Res<UiPageNumber>,
    mut current_page_number: Local<Option<usize>>,
    mut commands: Commands,
    mut root_entities: ResMut<RootUiEntities>,
    ui_assets: Res<UiAssets>,
) {
    // check to see if we event need to render anything
    let render_page = match *current_page_number {
        Some(p) if page_number.0 != p => {
            // destroy the current page so the next one can be rendered
            for entity in &root_entities.ui {
                commands.entity(*entity).despawn_recursive();
            }
            Some(page_number.0)
        }
        None => { // this is the first time this system has run - just set the local current page number
            *current_page_number = Some(page_number.0);
            None
        }
        _ => None,
    };

    match render_page {
        Some(p) => {
            *current_page_number = Some(p);
            let ui = match p {
                0 => create_main_menu(&mut commands, &ui_assets),
                1 | 2 | 3 => create_rules_page(&mut commands, ui_assets, page_number),
                _ => unreachable!(),
            };
            root_entities.ui = ui;
        }
        _ => {}
    }
}

fn create_main_menu(
    commands: &mut Commands,
    ui_assets: &Res<UiAssets>,
) -> Vec<Entity> {
    let root = commands
        .spawn_bundle(TransformBundle::default())
        .with_children(|parent| {
            // title
            let y_title = 100.0;
            parent
                .spawn_bundle(SpriteBundle{
                    texture: ui_assets.title.clone(),
                    transform: Transform::from_xyz(0.0, y_title, 1.0),
                    ..default()
                });

            // buttons
            let mut transform = Transform::from_xyz(0.0, y_title - 100.0, 1.0);
            ui::spawn_sprite_sheet_button(
                parent,
                ui_assets.play_button.clone(),
                transform.clone(),
                ButtonAction(ActionEvent(MainMenuAction::StartGame)),
                true,
            );

            let y_offset = 48.0 + 20.0; // 48 = height of a button, 20 = spacing between buttons
            transform.translation -= Vec3::new(0.0, y_offset, 0.0);
            ui::spawn_sprite_sheet_button(
                parent,
                ui_assets.rules_button.clone(),
                transform.clone(),
                ButtonAction(ActionEvent(MainMenuAction::NextPage)),
                true,
            );

            transform.translation -= Vec3::new(0.0, y_offset, 0.0);
            ui::spawn_sprite_sheet_button(
                parent,
                ui_assets.quit_button.clone(),
                transform.clone(),
                ButtonAction(ActionEvent(MainMenuAction::Quit)),
                true,
            );
        })
        .id()
        ;

    vec![root]
}

const RULES_P1: &str =
r#"- Objective -

Move all your marbles counter-clockwise around the board from your BASE to your HOME row.

- Movement -

You can use either the value of a single die or the sum of the dice to move a marble. Once both dice values have been used to make moves, your turn is over.

NOTE - The dice are rolled automatically at the beginning of your turn.
"#;
const RULES_P2: &str =
r#"- Base -

You must roll a 1 or a 6 to exit the BASE.

- Jumping -

You may jump opponents' marbles but NOT your own.

- Capturing -

Landing on an opponent's marble captures it, sending it back to its BASE.
"#;
const RULES_P3: &str =
r#"- Center Tile -

The center tile is a special space allowing a marble to skip ahead on the board.

A marble can only enter using the exact sum of the dice.

A marble can only enter from a corner with a different colored arrow.

A marble can only exit with a die roll of 1 but can then optionally use the other die to continue thier move.

A marble can only exit to the corner with the same colored arrow.
"#;
// TOOD: POWER-UPS

fn create_rules_page(
    commands: &mut Commands,
    ui_assets: Res<UiAssets>,
    page_number: Res<UiPageNumber>,
) -> Vec<Entity> {
    let text = commands
        .spawn_bundle(TextBundle{
            text: Text::with_section(
                match page_number.0 {
                    1 => RULES_P1,
                    2 => RULES_P2,
                    3 => RULES_P3,
                    _ => unreachable!(),
                },
                TextStyle{
                    font: ui_assets.mini_font.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
                TextAlignment::default()
            ),
            style: Style{
                size: Size::new(Val::Px(WINDOW_SIZE - 10.0 * 2.0), Val::Auto),
                align_self: AlignSelf::FlexEnd,
                position: Rect{
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .id()
        ;

    let buttons = commands
        .spawn_bundle(TransformBundle::default())
        .with_children(|parent| {
            let x_offset = match page_number.0 {
                1 | 2 => {
                    let x_offset = (160.0 / 2.0) + 20.0;
                    ui::spawn_sprite_sheet_button(
                        parent,
                        ui_assets.next_button.clone(),
                        Transform::from_xyz(x_offset, BOTTOM_BUTTON_Y, 5.0),
                        ButtonAction(ActionEvent(MainMenuAction::NextPage)),
                        true,
                    );
                    Some(-x_offset)
                }
                _ => None,
            };
            ui::spawn_sprite_sheet_button(
                parent,
                ui_assets.back_button.clone(),
                Transform::from_xyz(x_offset.unwrap_or_default(), BOTTOM_BUTTON_Y, 5.0),
                ButtonAction(ActionEvent(MainMenuAction::PrevPage)),
                true
            );
        })
        .id()
        ;

    vec![text, buttons]
}
