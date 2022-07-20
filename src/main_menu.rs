use bevy::prelude::*;
use bevy::app::AppExit;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::utils::ui::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(GameState::MainMenu) // the starting state
            .add_event::<ButtonActionEvent>()

            .insert_resource(UiPageNumber(0))

            .add_system_set(SystemSet::on_enter(GameState::MainMenu)
                .with_system(main_menu_enter)
            )
            .add_system_set(SystemSet::on_update(GameState::MainMenu)
                .with_system(button_interactions)
                .with_system(execute_menu_action)
                .with_system(menu_page_renderer)
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
    let ui = create_main_menu(&mut commands, ui_assets);
    commands.insert_resource(RootUiEntity(ui));
}

fn main_menu_exit(
    mut commands: Commands,
    root_entity: Res<RootUiEntity>,
) {
    commands.entity(root_entity.0).despawn_recursive();
    commands.remove_resource::<RootUiEntity>();
}

/// Handles UI button interactions, sending the associated [`ButtonActionEvent`]
/// when clicked.
fn button_interactions(
    mut button_action_events: EventWriter<ButtonActionEvent>,
    mut interaction_query: Query<
        (&Interaction, &ButtonAction, &mut UiImage, &Children),
        (Changed<Interaction>, With<Button>)
    >,
    mut text_query: Query<&mut Text>,
    ui_assets: Res<UiAssets>,
) {
    for (interaction, action, mut ui_image, children) in interaction_query.iter_mut() {
        let (image, text_color) = match *interaction {
            Interaction::Clicked => {
                button_action_events.send(action.0);
                (ui_assets.pressed_button.clone().into(), Color::WHITE)
            }
            Interaction::Hovered => (ui_assets.hovered_button.clone().into(), Color::rgb_u8(232, 106, 23)),
            Interaction::None => (ui_assets.normal_button.clone().into(), Color::WHITE),
        };

        // update button image
        ui_image.0 = image;

        // update button text color
        for child in children.iter() {
            let mut text = text_query.get_mut(*child).unwrap();
            for section in text.sections.iter_mut() {
                section.style.color = text_color;
            }
        }
    }
}

fn execute_menu_action(
    mut button_action_events: EventReader<ButtonActionEvent>,
    mut state: ResMut<State<GameState>>,
    mut page_number: ResMut<UiPageNumber>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for action in button_action_events.iter() {
        match *action {
            ButtonActionEvent::StateChange(s) => state.set(s).unwrap(),
            ButtonActionEvent::NextPage => page_number.0 += 1,
            ButtonActionEvent::PrevPage => page_number.0 -= 1,
            ButtonActionEvent::Quit => app_exit_events.send(AppExit),
        }
    }
}

/// Renders the current page in the menu if a page change occurred.
fn menu_page_renderer(
    page_number: Res<UiPageNumber>,
    mut current_page_number: Local<Option<usize>>,
    mut commands: Commands,
    mut root_entity: ResMut<RootUiEntity>,
    ui_assets: Res<UiAssets>,
) {
    // check to see if we event need to render anything
    let render_page = match *current_page_number {
        Some(p) if page_number.0 != p => {
            // destroy the current page so the next one can be rendered
            commands.entity(root_entity.0).despawn_recursive();
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
                0 => create_main_menu(&mut commands, ui_assets),
                1 | 2 | 3 => create_rules_page(&mut commands, ui_assets, page_number),
                _ => unreachable!(),
            };
            root_entity.0 = ui;
        }
        _ => {}
    }
}

fn create_main_menu(
    commands: &mut Commands,
    ui_assets: Res<UiAssets>,
) -> Entity {
    commands
        .spawn_bundle(vertical_node_bundle())
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle{
                    text: Text::with_section(
                        "Vexation",
                        TextStyle{
                            font: ui_assets.font.clone(),
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

            spawn_button(parent, &ui_assets, "Play", ButtonAction(ButtonActionEvent::StateChange(GameState::GameStart)));
            spawn_button(parent, &ui_assets, "Rules", ButtonAction(ButtonActionEvent::NextPage));
            spawn_button(parent, &ui_assets, "Quit", ButtonAction(ButtonActionEvent::Quit));
        })
        .id()
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
) -> Entity {
    commands
        .spawn_bundle(vertical_node_bundle())
        .with_children(|parent| {
            parent
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
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default()
                    ),
                    style: Style{
                        size: Size::new(Val::Px(WINDOW_SIZE - 10.0 * 2.0), Val::Auto),
                        margin: Rect{
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Px(10.0),
                            bottom: Val::Px(10.0),
                        },
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                ;

            parent
                .spawn_bundle(horizontal_node_bundle())
                .with_children(|parent| {
                    spawn_button(parent, &ui_assets, "Back", ButtonAction(ButtonActionEvent::PrevPage));
                    match page_number.0 {
                        1 | 2 => spawn_button(parent, &ui_assets, "Next", ButtonAction(ButtonActionEvent::NextPage)),
                        _ => {}
                    }
                })
                ;
        })
        .id()
}
