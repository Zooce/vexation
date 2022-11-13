use bevy::prelude::*;
use crate::components::Player;
use crate::constants::CENTER_INDEX;
use crate::resources::{CurrentPlayerData, GameData, GameState};
use crate::shared_systems::{SharedSystemLabel, should_run_shared_systems};
use rand::thread_rng;
use rand::distributions::{ Distribution, WeightedIndex };

#[derive(Debug)]
pub struct GeneratePowerUpEvent(pub Player, pub PowerChange);

#[derive(Debug)]
pub enum PowerBarEvent {
    Capture{captor: Player, captive: Player},
    Index{player: Player, index: usize, prev_index: usize},
}

#[derive(Debug)]
pub struct ActivatePowerUpEvent(pub PowerUp);

#[derive(Debug)]
pub enum PowerChange {
    Up,
    Down,
}

pub const MAX_POWER: f32 = 30.0;

#[derive(Debug)]
pub enum PowerUp {
    RollAgain,       // weight = 4
    DoubleDice,      // weight = 4
    EvadeCapture,    // weight = 3
    SelfJump,        // weight = 2 
    HomeRun,         // weight = 1
}

const POWER_UP_WEIGHTS: [usize; 5] = [4, 4, 3, 2, 1];

impl From<usize> for PowerUp {
    fn from(value: usize) -> Self {
        match value {
            0 => PowerUp::RollAgain,
            1 => PowerUp::DoubleDice,
            2 => PowerUp::EvadeCapture,
            3 => PowerUp::SelfJump,
            4 => PowerUp::HomeRun,
            _ => unreachable!(),
        }
    }
}

struct PowerUpDistribution(pub WeightedIndex<usize>);

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ActivatePowerUpEvent>()
            .add_event::<GeneratePowerUpEvent>()
            .add_event::<PowerBarEvent>()
            
            .insert_resource(PowerUpDistribution(WeightedIndex::new(&POWER_UP_WEIGHTS).unwrap()))

            .add_system_set(SystemSet::new()
                .label(SharedSystemLabel)
                .with_run_criteria(should_run_shared_systems)
                .with_system(update_power_bars)
                .with_system(generate_power_up)
                .with_system(activate_power_up)
            )
            ;
    }
}

fn update_power_bars(
    mut game_data: ResMut<GameData>,
    mut power_bar_events: EventReader<PowerBarEvent>,
    mut power_up_events: EventWriter<GeneratePowerUpEvent>,
) {
    for event in power_bar_events.iter() {
        for (player, power_change) in match event {
            PowerBarEvent::Capture{ captor, captive } => {
                vec![
                    (captor, game_data.players.get_mut(captor).unwrap().update_power(3.0)),
                    (captive, game_data.players.get_mut(captive).unwrap().update_power(-3.0)),
                ]
            },
            PowerBarEvent::Index{player, index, prev_index} => {
                let distance = if *index == CENTER_INDEX {
                    // home (54)  -> center (53) = 7
                    // prev_index -> center (53) = (5 or 17 or 29) - prev_index + 1
                    match *prev_index {
                        54 => 7,
                        _ if (0..=5).contains(prev_index) => 5 - prev_index + 1,
                        _ if (6..=17).contains(prev_index) => 17 - prev_index + 1,
                        _ if (18..=29).contains(prev_index) => 29 - prev_index + 1,
                        _ => unreachable!(),
                    }
                } else {
                    // home (54)   -> index = index + 1
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
                vec![(player, game_data.players.get_mut(player).unwrap().update_power(points))]
            }
        } {
            if let Some(power_change) = power_change {
                power_up_events.send(GeneratePowerUpEvent(*player, power_change));
            }
        }
    }
}

fn generate_power_up(
    mut power_up_events: EventReader<GeneratePowerUpEvent>,
    mut game_data: ResMut<GameData>,
    power_up_dist: Res<PowerUpDistribution>,
) {
    let mut rng = thread_rng();
    for event in power_up_events.iter() {
        match event.1 {
            PowerChange::Up => {
                // pick random power-up
                let power_up: PowerUp = power_up_dist.0.sample(&mut rng).into();
                println!("{:?}", power_up);
                // add power-up to player's list
                game_data.players.get_mut(&event.0).unwrap().power_ups.push(power_up);
                // mark current player to wait for animation
                // spawn power-up sprite in player's next empty power-up box
                // mark power-up for animation
            }
            PowerChange::Down => {
                // remove power up
                game_data.players.get_mut(&event.0).unwrap().power_ups.pop();
            }
        }
    }
}

fn activate_power_up(
    mut events: EventReader<ActivatePowerUpEvent>,
    mut state: ResMut<State<GameState>>,
    mut game_data: ResMut<GameData>,
    current_player_data: Res<CurrentPlayerData>,
    // mut marbles: Query<Entity, (With<Marble>, With<CurrentPlayer>)>,
) {
    let player_data = game_data.players.get_mut(&current_player_data.player).unwrap();
    for event in events.iter() {
        println!("activating {:?}", event);
        if let Some(new_state) = match event.0 {
            PowerUp::RollAgain => Some(GameState::DiceRoll),
            PowerUp::DoubleDice => {
                player_data.power_up_status.double_dice();
                Some(GameState::TurnSetup)
            }
            PowerUp::EvadeCapture => {
                // player_data.power_up_status.evade_capture();
                // for marble in marbles.iter_mut() {
                //     commands.entity(marble).insert(Evading);
                // }
                None
            }
            PowerUp::SelfJump => {
                player_data.power_up_status.jump_self();
                Some(GameState::TurnSetup)
            }
            PowerUp::HomeRun => {
                // player_data.power_up_status.home_run();
                // Some(GameState::TurnSetup)
                None
            }
        } {
            state.set(new_state).unwrap();
        }
    }
}
