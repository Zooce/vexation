// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::system_sets::ClickEvent;

pub struct ComputerTurnTimer(pub Timer);

#[derive(Debug)]
pub struct CurrentPlayerData {
    pub player: Player,
    pub possible_moves: Vec<(Entity, usize, WhichDie)>,
}

impl CurrentPlayerData {
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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WhichDie {
    One,
    Two,
    Both,
}

#[derive(Debug)]
pub struct DiceData {
    pub die_1: Entity,
    pub die_2: Entity,
    pub die_sheet_handle: Handle<TextureAtlas>,
    pub die_1_side: Option<u8>,
    pub die_2_side: Option<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    // ChooseColor,
    NextPlayer,
    DiceRoll,
    TurnSetup,
    HumanIdle,
    HumanMarbleSelected,
    ComputerTurn,
    ProcessMove,
}

pub struct HumanPlayer {
    pub color: Player,
}

pub struct RollAnimationTimer(pub Timer);

/// The resource for selection data.
pub struct SelectionData {
    /// The marble that is currently selected
    pub marble: Option<Entity>,
    /// The highlight texture for the selected marble and its possible moves
    pub highlight_texture: Handle<Image>,
    /// The click that selected a marble - this is so we can ignore that click
    /// in the destination selection state
    pub selection_click: Option<ClickEvent>,
}
