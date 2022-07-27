// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;

pub struct BufferTimer(pub Timer);

#[derive(Debug)]
pub struct ChooseColorData {
    pub masks: [Handle<Image>;4],
    pub current_color: Option<Player>,
    pub current_mask: Option<Entity>,
}

pub struct ComputerTurnTimer(pub Timer);

#[derive(Debug)]
pub struct CurrentPlayerData {
    pub player: Player,
    pub possible_moves: Vec<(Entity, usize, WhichDie)>,
    selected_move_index: Option<usize>,
}

impl CurrentPlayerData {
    pub fn new(player: Player) -> Self {
        Self{
            player,
            possible_moves: Vec::new(),
            selected_move_index: None,
        }
    }
    pub fn get_moves(&self, marble: Entity) -> Vec<(usize, WhichDie)> {
        self.possible_moves.iter()
            .filter_map(|(e, i, d)| {
                if *e == marble {
                    Some((*i, *d))
                } else {
                    None
                }
        }).collect()
    }

    pub fn select_move(&mut self, index: usize) {
        self.selected_move_index = Some(index);
    }

    pub fn use_selected_move(&mut self) -> Option<(Entity, usize, WhichDie)> {
        let mv = match self.selected_move_index {
            Some(index) => Some(self.possible_moves[index]),
            None => None,
        };
        self.selected_move_index = None;
        mv
    }
}

#[derive(Debug)]
pub struct DiceData {
    pub die_1: Entity,
    pub die_2: Entity,
    pub die_sheet_handle: Handle<TextureAtlas>,
    pub die_1_side: Option<u8>,
    pub die_2_side: Option<u8>,
    pub doubles: bool,
}

impl DiceData {
    pub fn use_die(&mut self, which: WhichDie, commands: &mut Commands) {
        match which {
            WhichDie::One => {
                self.die_1_side = None;
                commands.entity(self.die_1).insert(UsedDie);
            }
            WhichDie::Two => {
                self.die_2_side = None;
                commands.entity(self.die_2).insert(UsedDie);
            }
            WhichDie::Both => {
                self.die_1_side = None;
                self.die_2_side = None;
                commands.entity(self.die_1).insert(UsedDie);
                commands.entity(self.die_2).insert(UsedDie);
            }
        }
    }

    pub fn sides(&self) -> (Option<u8>, Option<u8>) {
        (self.die_1_side, self.die_2_side)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum GameState {
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
    GameEnd,
}

pub struct GamePlayEntities {
    pub camera: Entity,
    pub board: Entity,
}

/// The resource for highlight data.
pub struct HighlightData {
    /// the highlight texture for the selected marble
    pub marble_texture: Handle<Image>,
    /// The highlight texture for the selected marble's possible moves
    pub tile_texture: Handle<Image>,
}

pub struct HumanPlayer {
    pub color: Player,
    pub human_indicator: Entity,
}

pub struct MarbleAnimationDoneEvent(pub Player);

pub struct RollAnimationTimer(pub Timer);

pub struct RootUiEntity(pub Entity);

pub struct UiAssets {
    pub font: Handle<Font>,
    pub mini_font: Handle<Font>,
    pub normal_button: Handle<Image>,
    pub hovered_button: Handle<Image>,
    pub pressed_button: Handle<Image>,
}

pub struct UiPageNumber(pub usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WhichDie {
    One,
    Two,
    Both,
}
