// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use crate::system_sets::ClickEvent;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct CurrentPlayerData {
    pub player: Player,
    pub possible_moves: BTreeSet<(Entity, usize, WhichDie)>,
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

impl DiceData {
    pub fn get_dice_values(&self) -> Vec<(u8, WhichDie)> {
        match (self.die_1_side, self.die_2_side) {
            (Some(d1), Some(d2)) => vec![
                (d1, WhichDie::One), (d2, WhichDie::Two), (d1 + d2, WhichDie::Both)
            ],
            (Some(d1), None) => vec![(d1, WhichDie::One)],
            (None, Some(d2)) => vec![(d2, WhichDie::Two)],
            _ => vec![],
        }
    }
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
    // ChooseMoves,
    // ComputerChooseMoves,
    ProcessMove,
    // CheckWinner,
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
