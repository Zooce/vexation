use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use crate::resources::{GameState, WhichDie};

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonActionEvent {
    StateChange(GameState),
    NextPage,
    PrevPage,
    Quit,
}
pub struct ClickEvent(pub Vec2);

pub struct HighlightEvent{
    pub marble: Option<Entity>,
    /// If we only want to highlight one move index. This is ignored if `marble`
    /// is None.
    pub move_index: Option<usize>,
}

pub struct MoveEvent(pub (usize, WhichDie, Vec3));
