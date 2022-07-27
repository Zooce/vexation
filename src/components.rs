// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use crate::constants::*;
use crate::events::*;

#[derive(Component)]
pub struct ButtonAction(pub ButtonActionEvent);

#[derive(Component)]
pub struct CurrentPlayer;

#[derive(Component)]
pub struct Die {
    pub location: Vec3,
    pub timer: Timer,
}

/// Used to mark the highlight sprites when a marble is selected, so we can
/// later remove them when a marble is no longer selected.
#[derive(Component)]
pub struct Highlight{
    pub marble: Entity,
    pub index: usize,
}

#[derive(Component, Debug)]
pub struct Marble {
    /// This is a index into the `BOARD` (i.e. which space this marble is located).
    pub index: usize,
    /// Where this marble started in their base.
    pub origin: Vec3,
}

#[derive(Component, Debug)]
pub struct Moving{
    pub destination: Vec3,
    pub direction: Vec2,
    pub speed: f32,
}

impl Moving {
    /// Creates a new `Moving` component with a destination and origin.
    pub fn new(destination: Vec3, origin: Vec3) -> Self {
        let direction = destination - origin;
        let dir_norm = direction.normalize();
        Self{
            destination,
            direction: Vec2::new(dir_norm.x, dir_norm.y),
            speed: 550.0 + direction.length(), // TODO: can we get rid of the sqrt operation?
        }
    }
}

#[derive(Component, Debug, Eq, PartialEq, Clone, Copy)]
pub enum Player {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Player {
    pub fn rotate_coords(&self, coords: (f32, f32)) -> (f32, f32) {
        match self {
            Player::Red => coords,
            Player::Green => (coords.1, -coords.0),
            Player::Blue => (-coords.0, -coords.1),
            Player::Yellow => (-coords.1, coords.0),
        }
    }

    pub fn is_same_index(p1: Player, i1: usize, p2: Player, i2: usize) -> bool {
        if i1 == CENTER_INDEX && i2 == CENTER_INDEX {
            return true;
        }
        if i1 == CENTER_INDEX || i2 == CENTER_INDEX {
            return false;
        }
        let rotations = (4 - p1 as usize) % 4;
        (i1 + (rotations + p2 as usize) * 36) % 48 == i2
    }
}

impl From<u8> for Player {
    fn from(x: u8) -> Self {
        match x {
            0 => Player::Red,
            1 => Player::Green,
            2 => Player::Blue,
            3 => Player::Yellow,
            _ => unreachable!(),
        }
    }
}

#[derive(Component)]
pub struct SelectedMarble;

#[derive(Component)]
pub struct UsedDie;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_index_test() {
        assert!(Player::is_same_index(
            Player::Yellow, 16, Player::Blue, 28
        ));
        assert!(!Player::is_same_index(
            Player::Blue, 17, Player::Green, 13
        ));
        assert!(Player::is_same_index(
            Player::Blue, 21, Player::Yellow, 9
        ));
        assert!(!Player::is_same_index(
            Player::Blue, 21, Player::Green, 16
        ));
        assert!(!Player::is_same_index(
            Player::Green, 53, Player::Red, 17
        ));
    }
}
