use bevy::prelude::*;
use crate::buttons::{load_sprite_sheet, ActionEvent, ButtonAction, ButtonSize, ButtonState};
use crate::components::{CurrentPlayer, Evading, Marble, Player};
use crate::constants::{CENTER_INDEX, TILE_BUTTON_SIZE, UI_BUTTON_SIZE, TILE_SIZE, Z_UI};
use crate::resources::{CurrentPlayerData, DiceData, GameData, GameState, GameButtonAction, GamePlayEntities, HumanPlayer};
use crate::shared_systems::{SharedSystemLabel, should_run_shared_systems};
use rand::thread_rng;
use rand::distributions::{ Distribution, WeightedIndex };

#[derive(Debug)]
pub struct GeneratePowerUpEvent(pub Player, pub PowerChange);

#[derive(Debug)]
pub enum PowerEvent {
    Capture{captor: Player, captive: Player},
    Index{player: Player, index: usize, prev_index: usize},
    Use{player: Player, power_up: PowerUp},
}

#[derive(Debug)]
pub struct ActivatePowerUpEvent(pub PowerUp);

#[derive(Debug)]
pub enum PowerChange {
    Up,
    Down,
}

pub const MAX_POWER: f32 = 30.0;

#[derive(Debug, Clone, Copy)]
pub enum PowerUp {
    RollAgain,       // weight = 4
    DoubleDice,      // weight = 4
    EvadeCapture,    // weight = 3
    SelfJump,        // weight = 2
    CaptureNearest,  // weight = 1
    HomeRun,         // weight = 1
}

const POWER_UP_WEIGHTS: [usize; 6] = [4, 4, 3, 2, 1, 1];

impl From<usize> for PowerUp {
    fn from(value: usize) -> Self {
        match value {
            0 => PowerUp::RollAgain,
            1 => PowerUp::DoubleDice,
            2 => PowerUp::EvadeCapture,
            3 => PowerUp::SelfJump,
            4 => PowerUp::CaptureNearest,
            5 => PowerUp::HomeRun,
            _ => unreachable!(),
        }
    }
}

#[derive(Resource)]
struct PowerUpDistribution(pub WeightedIndex<usize>);

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ActivatePowerUpEvent>()
            .add_event::<GeneratePowerUpEvent>()
            .add_event::<PowerEvent>()
            .add_event::<PowerBarEvent>()

            .insert_resource(PowerUpDistribution(WeightedIndex::new(&POWER_UP_WEIGHTS).unwrap()))

            .add_system_set(SystemSet::new()
                .label(SharedSystemLabel)
                .with_run_criteria(should_run_shared_systems)
                .with_system(handle_power_events)
                .with_system(generate_power_up)
                .with_system(activate_power_up)
            )
            ;
    }
}

#[derive(Component, Debug)]
pub struct PowerBar {
    pub power: f32,
    multiplier: f32,
    pub origin: f32,
}

impl PowerBar {
    pub fn new(origin: f32) -> Self {
        Self {
            power: 0.,
            multiplier: 1.,
            origin,
        }
    }
}

impl PowerBar {
    pub fn update(&mut self, delta: f32) -> Option<PowerChange> {
        if self.power == MAX_POWER && delta.is_sign_positive() { return None; }
        let new_power = (self.power + delta).clamp(0.0, MAX_POWER);
        let change = if new_power >= 10.0 * self.multiplier {
            self.multiplier += 1.0;
            Some(PowerChange::Up)
        } else if new_power < 10.0 * (self.multiplier - 1.0) {
            self.multiplier -= 1.0;
            Some(PowerChange::Down)
        } else {
            None
        };
        self.power = new_power;
        change
    }
}

pub struct PowerBarEvent {
    pub power: f32,
    pub player: Player,
}

fn handle_power_events(
    mut game_data: ResMut<GameData>,
    mut power_events: EventReader<PowerEvent>,
    mut power_up_events: EventWriter<GeneratePowerUpEvent>,
    mut activate_events: EventWriter<ActivatePowerUpEvent>,
    mut power_bars: Query<(&mut PowerBar, &mut Transform, &Player)>,
) {
    for event in power_events.iter() {
        for (player, power) in match event {
            PowerEvent::Capture{ captor, captive } => {
                vec![
                    (captor, 3.),
                    (captive, -3.),
                ]
            },
            PowerEvent::Index{ player, index, prev_index } => {
                let distance = if *index == CENTER_INDEX {
                    // TODO: with the double dice power up, the longest move you can make is 24 spaces
                    // base (54)  -> center (53) = 7
                    // prev_index -> center (53) = (5 or 17 or 29) - prev_index + 1
                    match *prev_index {
                        54 => 7,
                        _ if (0..=5).contains(prev_index) => 5 - prev_index + 1,
                        _ if (6..=17).contains(prev_index) => 17 - prev_index + 1,
                        _ if (18..=29).contains(prev_index) => 29 - prev_index + 1,
                        _ => unreachable!(),
                    }
                } else {
                    // base (54)   -> index = index + 1
                    // center (53) -> index = index + 1 - 41
                    // prev_index  -> index = index - prev_index
                    match *prev_index {
                        54 => index + 1,
                        CENTER_INDEX => index + 1 - 41,
                        _ => index - prev_index,
                    }
                } as f32;
                let points = match index {
                    0..=47 => 1.0,
                    _ => 2.0,
                } * 10.0 * distance / 48.0;
                vec![(player, points)]
            }
            PowerEvent::Use{ player, power_up } => {
                activate_events.send(ActivatePowerUpEvent(*power_up));
                vec![(player, -10.)]
            }
        } {
            let (mut bar, mut transform, _) = power_bars.iter_mut().find(|(_, _, &p)| p == *player).unwrap();
            let change = bar.update(power);
            transform.translation.y = bar.origin + bar.power * 4.;
            if let Some(change) = change {
                if bar.power != 0. && bar.power != MAX_POWER {
                    match change {
                        PowerChange::Up => transform.translation.y += 3.,
                        PowerChange::Down => transform.translation.y -= 3.,
                    }
                }
                power_up_events.send(GeneratePowerUpEvent(*player, change));
            }
        }
    }
}

fn generate_power_up(
    mut power_up_events: EventReader<GeneratePowerUpEvent>,
    mut game_data: ResMut<GameData>,
    power_up_dist: Res<PowerUpDistribution>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_play_entities: ResMut<GamePlayEntities>,
    human_player: Res<HumanPlayer>,
) {
    let mut rng = thread_rng();
    for GeneratePowerUpEvent(player, change) in power_up_events.iter() {
        let player_string = format!("{:?}", &player);
        match change {
            PowerChange::Up => {
                // spawn the power up button first
                let (x, y) = match player {
                    Player::Red => (-6.5, 2.5),
                    Player::Green => (6.5, 2.5),
                    Player::Blue => (6.5, -5.5),
                    Player::Yellow => (-6.5, -5.5),
                };
                let i = game_data.players.get(&player).unwrap().power_ups.len();
                let mut transform = Transform::from_xyz(x * TILE_SIZE, (y + 1.5 * (i as f32)) * TILE_SIZE, Z_UI);
                // TODO: here's what I really want spawning buttons to look like
                // btn_spawn_evnt.send(ButtonSpawnEvent{
                //     ButtonDesc::PowerUpButton(i), // the `i` tells it which power up slot it goes into
                //     Transform::from_xyz(...),
                //
                // })
                let mut builder = commands.spawn((
                    SpriteSheetBundle{
                        texture_atlas: load_sprite_sheet("power-ups/button.png", TILE_BUTTON_SIZE.clone(), (3, 1), &asset_server, &mut texture_atlases),
                        transform,
                        ..default()
                    },
                    ButtonAction(ActionEvent(match i {
                        0 => GameButtonAction::PowerUpOne,
                        1 => GameButtonAction::PowerUpTwo,
                        2 => GameButtonAction::PowerUpThree,
                        _ => unreachable!(),
                    })),
                ));
                if human_player.color == *player { 
                    // only want to add button state and size if this is for the human player - we don't want them interacting with the computer players' buttons
                    builder.insert((ButtonState::NotHovered, ButtonSize(UI_BUTTON_SIZE.clone())));
                }
                game_play_entities.board_entities.push(builder.id());
                // randomly generate the power up
                let power_up: PowerUp = power_up_dist.0.sample(&mut rng).into();
                game_data.players.get_mut(&player).unwrap().power_ups.push(power_up);
                println!("{:>6} ++ {:?}", player_string, game_data.players.get(&player).unwrap().power_ups);

                // TODOs:
                // mark current player to wait for animation ?? maybe not ??
                // spawn power-up sprite sheet in player's next empty power-up box
                // mark power-up for animation
            }
            PowerChange::Down => {
                // TODO: don't remove earned power-ups
                // remove power up
                game_data.players.get_mut(&player).unwrap().power_ups.pop();
                println!("{:>6} -- {:?}", player_string, game_data.players.get(&player).unwrap().power_ups);
            }
        }
    }
}

fn activate_power_up(
    mut commands: Commands,
    mut events: EventReader<ActivatePowerUpEvent>,
    mut state: ResMut<State<GameState>>,
    mut game_data: ResMut<GameData>,
    mut dice_data: ResMut<DiceData>,
    current_player_data: Res<CurrentPlayerData>,
    mut marbles: Query<Entity, (With<Marble>, With<CurrentPlayer>)>,
) {
    let player_data = game_data.players.get_mut(&current_player_data.player).unwrap();
    for event in events.iter() {
        println!("activating {:?}", event.0);
        if let Some(new_state) = match event.0 {
            PowerUp::RollAgain => Some(GameState::DiceRoll),
            PowerUp::DoubleDice => {
                dice_data.dice.multiplier = 2;
                Some(GameState::TurnSetup)
            }
            PowerUp::EvadeCapture => {
                player_data.power_up_status.evade_capture();
                for marble in marbles.iter_mut() {
                    commands.entity(marble).insert(Evading);
                }
                // TODO: there needs to be a system that highlights the evading marbles
                None
            }
            PowerUp::SelfJump => {
                player_data.power_up_status.jump_self();
                Some(GameState::TurnSetup)
            }
            PowerUp::CaptureNearest => {
                player_data.power_up_status.capture_nearest();
                Some(GameState::TurnSetup)
            }
            PowerUp::HomeRun => {
                player_data.power_up_status.home_run();
                Some(GameState::TurnSetup)
            }
        } {
            state.set(new_state).unwrap();
        }
    }
}
