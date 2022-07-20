use crate::resources::GameState;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonActionEvent {
    StateChange(GameState),
    NextPage,
    PrevPage,
    Quit,
}
