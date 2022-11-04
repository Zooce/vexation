use bevy::ecs::entity::Entity;

pub struct HighlightEvent{
    pub marble: Option<Entity>,
    /// If we only want to highlight one move index. This is ignored if `marble`
    /// is None.
    pub move_index: Option<usize>,
}

