use bevy::prelude::*;
use crate::buttons::{ActionEvent, ButtonAction, ButtonSize, ButtonState};
use crate::components::{CurrentPlayer, Evading, Marble, Player};
use crate::constants::{CENTER_INDEX, TILE_BUTTON_SIZE, TILE_SIZE, Z_UI};
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
    Use{player: Player, index: usize},
}

#[derive(Debug)]
pub struct ActivatePowerUpEvent(pub PowerUp);

#[derive(Debug)]
pub enum PowerChange {
    Up,
    Down,
}

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

#[derive(Resource)]
pub struct PowerUpSpriteSheets {
    pub roll_again: Handle<TextureAtlas>,
    pub double_dice: Handle<TextureAtlas>,
    pub evade_capture: Handle<TextureAtlas>,
    pub self_jump: Handle<TextureAtlas>,
    pub capture_nearest: Handle<TextureAtlas>,
    pub home_run: Handle<TextureAtlas>,
}

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
    pub power_up_count: usize,
    pub origin: f32,
}

impl PowerBar {
    pub fn new(origin: f32) -> Self {
        Self {
            power: 0.,
            power_up_count: 0,
            origin,
        }
    }
}

pub const MAX_POWER: f32 = 10.0;
pub const MAX_POWER_UPS: usize = 3;

impl PowerBar {
    pub fn update(&mut self, delta: f32) -> Option<PowerChange> {
        let new_power = (self.power + delta).max(0.0); // this reads really weird but it means this -> max(self.power + delta, 0.0)
        if new_power >= MAX_POWER {
            match self.power_up_count {
                0 | 1 => {
                    self.power = (new_power - MAX_POWER).max(0.0); // carry over
                    self.power_up_count += 1;
                    Some(PowerChange::Up)
                }
                2 => {
                    self.power = 0.0; // reset
                    self.power_up_count += 1;
                    Some(PowerChange::Up)
                }
                _ => None,
            }
        } else {
            if self.power_up_count < 3 {
                self.power = new_power;
            }
            None
        }
    }
}

pub struct PowerBarEvent {
    pub power: f32,
    pub player: Player,
}

fn handle_power_events(
    mut commands: Commands,
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
                    (captor, Some(3.)),
                    (captive, Some(-3.)),
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
                vec![(player, Some(points))]
            }
            PowerEvent::Use{ player, index } => {
                let (power_up, power_up_button) = game_data.players.get_mut(&player).unwrap().use_power_up(*index).unwrap();
                commands.entity(power_up_button).despawn();
                activate_events.send(ActivatePowerUpEvent(power_up));
                vec![(player, None)]
            }
        } {
            let (mut bar, mut transform, _) = power_bars.iter_mut().find(|(_, _, &p)| p == *player).unwrap();
            match power {
                Some(power) => {
                    let change = bar.update(power);
                    println!("{player:?} {bar:?}");
                    // power-fill sprite is 14 x 126 (that 126 represents 10 power points, so 126 / 10 = 12.6 pixels for every point)
                    transform.translation.y = bar.origin + bar.power * 12.6;
                    if change.is_some() {
                        power_up_events.send(GeneratePowerUpEvent(*player, PowerChange::Up));
                    }
                }
                None => { bar.power_up_count -= 1; }
            }
        }
    }
}

fn generate_power_up(
    mut power_up_events: EventReader<GeneratePowerUpEvent>,
    mut game_data: ResMut<GameData>,
    power_up_dist: Res<PowerUpDistribution>,
    mut commands: Commands,
    power_up_sprite_sheets: Res<PowerUpSpriteSheets>,
    mut game_play_entities: ResMut<GamePlayEntities>,
    human_player: Res<HumanPlayer>,
) {
    let mut rng = thread_rng();
    for GeneratePowerUpEvent(player, change) in power_up_events.iter() {
        let player_string = format!("{:?}", &player);
        if let PowerChange::Up = change {
            // spawn the power up button first
            let (x, y) = match player {
                Player::Red => (-6.5, 2.5),
                Player::Green => (6.5, 2.5),
                Player::Blue => (6.5, -5.5),
                Player::Yellow => (-6.5, -5.5),
            };
            // get the next unused power-up slot
            let i = match game_data.players.get(&player).unwrap().power_ups {
                [None, _, _] => 0,
                [_, None, _] => 1,
                [_, _, None] => 2,
                _ => unreachable!(),
            };

            // randomly generate the power up
            let power_up: PowerUp = power_up_dist.0.sample(&mut rng).into();

            let sprite_sheet = SpriteSheetBundle{
                texture_atlas: match power_up {
                    PowerUp::RollAgain => power_up_sprite_sheets.roll_again.clone(),
                    PowerUp::DoubleDice => power_up_sprite_sheets.double_dice.clone(),
                    PowerUp::EvadeCapture => power_up_sprite_sheets.evade_capture.clone(),
                    PowerUp::SelfJump => power_up_sprite_sheets.self_jump.clone(),
                    PowerUp::CaptureNearest => power_up_sprite_sheets.capture_nearest.clone(),
                    PowerUp::HomeRun => power_up_sprite_sheets.home_run.clone(),
                },
                transform: Transform::from_xyz(x * TILE_SIZE, (y + 1.5 * (i as f32)) * TILE_SIZE, Z_UI),
                ..default()
            };
            let action = ButtonAction(ActionEvent(match i {
                0 => GameButtonAction::PowerUpOne(*player),
                1 => GameButtonAction::PowerUpTwo(*player),
                2 => GameButtonAction::PowerUpThree(*player),
                _ => unreachable!(),
            }));

            let power_up_button = if human_player.color == *player {
                // only want to add button state and size if this is for the human player - we don't want them interacting with the computer players' buttons
                commands.spawn((
                    sprite_sheet,
                    action,
                    ButtonState::NotHovered,
                    ButtonSize(TILE_BUTTON_SIZE.clone())
                )).id()
            } else {
                commands.spawn((sprite_sheet, action)).id()
            };
            game_data.players.get_mut(&player).unwrap().power_ups[i] = Some((power_up, power_up_button));
            println!("{:>6} ++ {:?}", player_string, game_data.players.get(&player).unwrap().power_ups);

            // TODOs:
            // mark current player to wait for animation ?? maybe not ??
            // spawn power-up sprite sheet in player's next empty power-up box
            // mark power-up for animation
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
