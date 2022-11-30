use bevy::prelude::*;
use crate::constants::*;

#[derive(Component)]
pub struct CurrentPlayer;

#[derive(Component)]
pub struct Die {
    pub location: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct Evading;

// shared_systems.rs
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
    /// The previous index where this marble was.
    pub prev_index: usize,
    /// Where this marble started in their base.
    pub origin: Vec3,
}

impl Marble {
    pub fn new(origin: Vec3) -> Self {
        Self {
            index: BOARD.len(),
            prev_index: BOARD.len(),
            origin,
        }
    }

    pub fn update_index(&mut self, new_index: usize) {
        self.prev_index = self.index;
        self.index = new_index;
    }
}

#[derive(Component, Debug)]
pub struct Moving {
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

#[derive(Component, Debug, Eq, PartialEq, Hash, Clone, Copy)]
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

    /// Shifts the index of the `from` player such that it becomes the index of
    /// the `to` player at that same location. For example, the 0 index for
    /// `Player::Red` is the 12 index for `Player::Yellow`, so this function
    /// would be called like this:
    /// 
    /// ```rust
    /// Player::rotate_index(0, Player::Red, Player::Yellow); // returns 12
    /// ```
    ///
    /// NOTE: This does not work for the home row!
    pub fn shift_index(i: usize, from: Player, to: Player) -> usize {
        if i == CENTER_INDEX { return CENTER_INDEX; }
        if i == BOARD.len() { return BOARD.len(); }
        let rotations = (4 - from as usize) % 4;
        (i + (rotations + to as usize) * 36) % 48
    }

    pub fn is_same_index(p1: Player, i1: usize, p2: Player, i2: usize) -> bool {
        if i1 == CENTER_INDEX && i2 == CENTER_INDEX {
            return true;
        }
        if i1 == CENTER_INDEX || i2 == CENTER_INDEX {
            return false;
        }
        Player::shift_index(i1, p1, p2) == i2
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
        assert!(Player::is_same_index(
            Player::Blue, 28, Player::Yellow, 16
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

    #[test]
    fn shift_index_test() {
        let tests = [
            (0, Player::Red, 36, Player::Green),
            (1, Player::Red, 25, Player::Blue),
            (2, Player::Red, 14, Player::Yellow),
            (BOARD.len(), Player::Red, BOARD.len(), Player::Green),
            (CENTER_INDEX, Player::Red, CENTER_INDEX, Player::Blue),
        ];
        for (i, from, expected, to) in tests {
            assert_eq!(Player::shift_index(i, from, to), expected);
        }
    }
}
