use ratatui::crossterm::event::KeyEvent;

use crate::config::keymappings::AppAction;

use super::{Component, ComponentEvent, KeyAction};

pub struct GeneralInputHandlerComponent;

impl Component for GeneralInputHandlerComponent {
    fn handle_input(&mut self, _: KeyEvent, action: AppAction) -> KeyAction {
        match action {
            AppAction::Close | AppAction::Quit => KeyAction::Event(ComponentEvent::QuitRequested),
            _ => KeyAction::Unhandled,
        }
    }
}
