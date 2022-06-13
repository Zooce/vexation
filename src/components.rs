// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;

#[derive(Component)]
pub struct CurrentPlayer;

#[derive(Component)]
pub struct DieAnimationTimer(pub Timer);

/// Used to mark the highlight sprites when a marble is selected, so we can
/// later remove them when a marble is no longer selected.
#[derive(Component)]
pub struct Highlight(pub Entity);

#[derive(Component)]
pub struct Marble {
    /// This is a index into the `BOARD` (i.e. which space this marble is located).
    pub index: usize,
}

#[derive(Component, Debug)]
pub struct Moving{
    pub timer: Timer,
    pub destination: Vec3,
    pub anim_step_dist: Vec3,
}

impl Moving {

    /// Creates a new `Moving` component with a destination and origin.
    pub fn new(destination: Vec3, origin: Vec3) -> Self {
        const MOVE_ANIM_DURATION: f32 = 0.2; // 200ms
        Self{
            timer: Timer::from_seconds(MOVE_ANIM_DURATION, false),
            destination,
            anim_step_dist: (destination - origin) / MOVE_ANIM_DURATION,
        }
    }
}

#[derive(Component, Debug, Eq, PartialEq, Clone)]
pub enum Player {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Player {
    pub fn rotate(&self, coords: (f32, f32)) -> (f32, f32) {
        match self {
            Player::Red => coords,
            Player::Green => (coords.1, -coords.0),
            Player::Blue => (-coords.0, -coords.1),
            Player::Yellow => (-coords.1, coords.0),
        }
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
