use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Component, ComponentEvent, KeyAction};

pub struct GeneralInputHandlerComponent;

impl Component for GeneralInputHandlerComponent {
    fn handle_input(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Esc => KeyAction::Event(ComponentEvent::QuitRequested),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                KeyAction::Event(ComponentEvent::QuitRequested)
            }
            _ => KeyAction::Unhandled,
        }
    }
}
