// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::components::*;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct CurrentPlayerData {
    pub player: Player,
    pub possible_moves: BTreeSet<(Entity, usize)>,
}

impl CurrentPlayerData {
    pub fn get_moves(&self, marble: Entity) -> Vec<usize> {
        self.possible_moves.iter()
            .filter_map(|(e, i)| {
                if *e == marble {
                    Some(*i)
                } else {
                    None
                }
        }).collect()
    }
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
    pub fn get_dice_values(&self) -> Vec<u8> {
        match (self.die_1_side, self.die_2_side) {
            (Some(d1), Some(d2)) => vec![d1, d2, d1 + d2],
            (Some(d1), None) => vec![d1],
            (None, Some(d2)) => vec![d2],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    // ChooseColor,
    NextPlayer,
    DiceRoll,
    PlayTurn,
    HumanTurn,
    ComputerTurn,
    // ChooseMoves,
    // ComputerChooseMoves,
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
}
