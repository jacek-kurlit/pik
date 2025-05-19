use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;

use crate::config::keymappings::AppAction;

use super::LayoutRects;

pub mod general_input_handler;
pub mod help_footer;
pub mod help_popup;
pub mod process_details;
pub mod process_table;
pub mod processes_view;
pub mod search_bar;

pub trait Component {
    fn handle_input(&mut self, _original: KeyEvent, _app_action: AppAction) -> KeyAction {
        KeyAction::Unhandled
    }

    fn handle_event(&mut self, _: &ComponentEvent) -> Option<ComponentEvent> {
        None
    }

    fn render(&mut self, _frame: &mut Frame, _layout: &LayoutRects) {}
}

pub enum KeyAction {
    // component could not handle key event
    Unhandled,
    // component handled key event but no event is published
    Consumed,
    // component handled key event and event is published
    Event(ComponentEvent),
}

pub enum ComponentEvent {
    ProcessListRefreshed,
    NoProcessToKill,
    ProcessKilled,
    ProcessKillFailed,
    QuitRequested,
}
