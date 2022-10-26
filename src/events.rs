use bevy::ecs::entity::Entity;
use crate::components::Player;
use crate::resources::PowerLevel;

/// An `ActionEvent` that is sent when a button is clicked. The type `T` defines
/// what those actions really are.
#[derive(Clone, Copy)]
pub struct ActionEvent<T>(pub T);

#[derive(Debug)]
pub struct GeneratePowerUpEvent(pub Player, pub PowerLevel);

pub struct HighlightEvent{
    pub marble: Option<Entity>,
    /// If we only want to highlight one move index. This is ignored if `marble`
    /// is None.
    pub move_index: Option<usize>,
}

// POWERUP: add power bar event
#[derive(Debug)]
pub enum PowerBarEvent {
    Capture{captor: Player, captive: Player},
    Deflection{deflector: Player, deflected: Player},
    Index{player: Player, index: usize, prev_index: usize},
}

