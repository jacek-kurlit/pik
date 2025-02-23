use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::processes::Process;

pub mod help_footer;
pub mod process_details;
pub mod process_table;
pub mod search_bar;

pub trait Component {
    fn handle_input(&mut self, _: KeyEvent) -> KeyAction {
        KeyAction::Unhandled
    }

    fn handle_event(&mut self, _: &ComponentEvent) -> Option<ComponentEvent> {
        None
    }

    //TODO: i dont like mut here
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub enum KeyAction {
    //component could not handle key event
    Unhandled,
    // component handled key event but no event is published
    Consumed,
    // component handled key event and event is published
    Event(ComponentEvent),
}

pub enum ComponentEvent {
    ProcessSelected(Process),
    SearchQueryUpdated(String),
    SearchByTextRequested(String),
    NoProcessToKill,
    ProcessKilled,
    ProcessKillFailed,
    QuitRequested,
}
