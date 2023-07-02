use std::collections::HashMap;

use bevy::prelude::*;
use crate::components::*;
use crate::power::{PowerUp, MAX_POWER_UPS};

#[derive(Resource)]
pub struct BufferTimer(pub Timer);

#[derive(Resource)]
pub struct ComputerTurnTimers {
   pub move_timer: Timer,
   pub buffer_timer: Timer,
}

impl ComputerTurnTimers {
    pub fn reset(&mut self) {
        self.move_timer.reset();
        self.buffer_timer.reset();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MarbleMove {
    pub destination: usize,
    pub distance: usize,
    pub which: WhichDie,
}

impl From<(usize, usize, WhichDie)> for MarbleMove {
    fn from(value: (usize, usize, WhichDie)) -> Self {
        Self {
            destination: value.0,
            distance: value.1,
            which: value.2,
        }
    }
}

#[derive(Debug, Resource)]
pub struct CurrentPlayerData {
    pub player: Player,
    pub possible_moves: Vec<(Entity, MarbleMove)>,
    pub selected_move: Option<MarbleMove>,
    pub selected_marble: Option<Entity>,
    pub moved_marble: Option<Entity>,
}

impl CurrentPlayerData {
    pub fn new(player: Player) -> Self {
        Self{
            player,
            possible_moves: Vec::new(),
            selected_move: None,
            selected_marble: None,
            moved_marble: None,
        }
    }

    pub fn get_moves(&self, marble: Entity) -> Vec<MarbleMove> {
        self.possible_moves.iter()
            .filter_map(|(e, m)| {
                if *e == marble {
                    Some(*m)
                } else {
                    None
                }
        }).collect()
    }

    pub fn select_move(&mut self, m: (Entity, MarbleMove)) {
        self.selected_marble = Some(m.0);
        self.selected_move = Some(m.1);
    }

    pub fn get_selected_move(&self) -> Option<(Entity, MarbleMove)> {
        match (self.selected_marble, self.selected_move) {
            (Some(entity), Some(marble_move)) => Some((entity, marble_move)),
            (None, Some(_)) => unreachable!(),
            _ => None,
        }
    }

    pub fn move_marble(&mut self) {
        self.moved_marble = self.selected_marble.take();
    }

    pub fn clear(&mut self) {
        self.possible_moves = Vec::new();
        self.selected_marble = None;
        self.selected_move = None;
        self.moved_marble = None;
    }
}

#[derive(Debug, Default)]
pub struct Dice {
    pub one: Option<u8>,
    pub two: Option<u8>,
    pub doubles: bool,
    pub multiplier: u8,
}

impl Dice {
    pub fn new(one: u8, two: u8) -> Self {
        Self {
            one: Some(one),
            two: Some(two),
            doubles: one == two,
            multiplier: 1,
        }
    }

    pub fn use_die(&mut self, which: WhichDie) {
        match which {
            WhichDie::One => self.one = None,
            WhichDie::Two => self.two = None,
            WhichDie::Both => {
                self.one = None;
                self.two = None;
            }
            WhichDie::Neither => {}
        }
        if self.is_empty() {
            self.multiplier = 1;
        }
    }

    pub fn did_use_any(&self) -> bool {
        self.one.is_none() || self.two.is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.one.is_none() && self.two.is_none()
    }
}

#[derive(Debug, Resource)]
pub struct DiceData {
    pub die_1: Entity,
    pub die_2: Entity,
    pub die_sheet_handle: Handle<TextureAtlas>,
    pub dice: Dice,
}

impl DiceData {
    pub fn use_die(&mut self, which: WhichDie, commands: &mut Commands) {
        self.dice.use_die(which);
        match which {
            WhichDie::One => {
                commands.entity(self.die_1).insert(UsedDie);
            }
            WhichDie::Two => {
                commands.entity(self.die_2).insert(UsedDie);
            }
            WhichDie::Both => {
                commands.entity(self.die_1).insert(UsedDie);
                commands.entity(self.die_2).insert(UsedDie);
            }
            WhichDie::Neither => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GameButtonAction {
    Done,
    PowerUpOne(Player),
    PowerUpTwo(Player),
    PowerUpThree(Player),
}

#[derive(Debug)]
pub struct PowerUpStatus {
    pub evade_capture_turns: u8,
    pub jump_self_turns: u8,
    pub capture_nearest: bool,
    pub home_run: bool,
}

pub enum PowerDownType {
    Evading,
    SelfJumping,
    EvadingAndSelfJumping,
}

impl PowerUpStatus {
    pub fn evade_capture(&mut self) -> bool {
        let already_evading = self.evade_capture_turns > 0;
        // translates to 3 full turns since the first one will get decremented
        // during the turn it's used
        self.evade_capture_turns = 4;
        return already_evading;
    }

    pub fn jump_self(&mut self) -> bool {
        let already_jumping = self.jump_self_turns > 0;
        // translates to 3 full turns since the first one will get decremented
        // during the turn it's used
        self.jump_self_turns = 4;
        return already_jumping;
    }

    pub fn capture_nearest(&mut self) {
        self.capture_nearest = true;
    }

    pub fn home_run(&mut self) {
        self.home_run = true;
    }

    /// Advance power-up counters - return true if counters are exhausted
    pub fn tick(&mut self) -> Option<PowerDownType> {
        self.clear_one_shots();
        let evade_done = if self.evade_capture_turns > 0 {
            self.evade_capture_turns -= 1;
            if self.evade_capture_turns == 0 {
                true
            } else { false }
        } else { false } ;
        let jump_done = if self.jump_self_turns > 0 {
            self.jump_self_turns -= 1;
            if self.jump_self_turns == 0 {
                true
            } else { false }
        } else { false };
        match (evade_done, jump_done) {
            (true, false) => Some(PowerDownType::Evading),
            (false, true) => Some(PowerDownType::SelfJumping),
            (true, true) => Some(PowerDownType::EvadingAndSelfJumping),
            _ => None,
        }
    }

    pub fn clear_one_shots(&mut self) {
        self.capture_nearest = false;
        self.home_run = false;
    }
}

impl Default for PowerUpStatus {
    fn default() -> Self {
        Self {
            evade_capture_turns: 0,
            jump_self_turns: 0,
            capture_nearest: false,
            home_run: false,
        }
    }
}

#[derive(Debug)]
pub struct PlayerData {
    pub turn_move_count: u8,
    pub consecutive_empty_turns: u8,
    pub multiplier: f32,
    pub power_ups: [Option<(PowerUp, Entity)>; MAX_POWER_UPS],
    pub power_up_status: PowerUpStatus,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            turn_move_count: 0,
            consecutive_empty_turns: 0,
            multiplier: 1.0,
            power_ups: [None, None, None],
            power_up_status: PowerUpStatus::default(),
        }
    }
}

impl PlayerData {
    pub fn end_of_turn(&mut self) -> Option<PowerDownType> {
        self.consecutive_empty_turns = if self.turn_move_count > 0 {
            0
        } else {
            self.consecutive_empty_turns + 1
        };
        self.turn_move_count = 0;
        self.power_up_status.tick()
    }

    pub fn use_power_up(&mut self, index: usize) -> Option<(PowerUp, Entity)> {
        if index < self.power_ups.len() {
            let power_up = self.power_ups[index];
            self.power_ups[index] = None;
            power_up
        } else {
            None
        }
    }
}

/// The data keeping track of the current state of the game.
#[derive(Resource)]
pub struct GameData {
    pub players: HashMap<Player, PlayerData>,
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum GameState {
    #[default]
    MainMenu,
    GameStart,
    ChooseColor,
    NextPlayer,
    DiceRoll,
    TurnSetup,
    ComputerTurn,
    HumanTurn,
    WaitForAnimation,
    ProcessMove,
    EndTurn,
    GameEnd,
}

#[derive(Resource, Default)]
pub struct GamePlayEntities {
    pub board_entities: Vec<Entity>,
}

/// The resource for highlight data.
#[derive(Resource)]
pub struct HighlightData {
    /// the highlight texture for the selected marble
    pub marble_texture: Handle<Image>,
    /// The highlight texture for the selected marble's possible moves
    pub tile_texture: Handle<Image>,
}

#[derive(Resource)]
pub struct HumanPlayer {
    pub color: Player,
    pub human_indicator: Entity,
}

pub struct MarbleAnimationDoneEvent(pub Player);

#[derive(Resource)]
pub struct RollAnimationTimer(pub Timer);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WhichDie {
    One,
    Two,
    Both,
    Neither,
}
