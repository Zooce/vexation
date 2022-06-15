// TODO: Bring only what we're actually using into scope - I'm bringing in everything help me code faster.

use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseButton};
use crate::constants::*;

mod idle;
mod selected;

pub use idle::*;
pub use selected::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickEvent {
    x: f32,
    y: f32,
}

pub fn handle_mouse_clicks(
    windows: Res<Windows>,
    mut mouse_events: EventReader<MouseButtonInput>,
    mut click_events: EventWriter<ClickEvent>,
) {
    // we need the current position of the cursor or else we don't really care
    let cursor = match windows.get_primary() {
        Some(w) => match w.cursor_position() {
            Some(c) => c,
            None => return,
        }
        None => return,
    };

    // we really only care about the most recent left mouse button press
    if let Some(_) = mouse_events.iter()
        .filter(|e| e.button == MouseButton::Left && e.state.is_pressed())
        .last()
    {
        // cursor position is measured from the bottom left corner, but transforms are measured from their center
        let (x, y) = (cursor.x - WINDOW_SIZE / 2., cursor.y - WINDOW_SIZE / 2.);
        click_events.send(ClickEvent{ x, y });
    }
}
