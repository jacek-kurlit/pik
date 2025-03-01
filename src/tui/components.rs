use crossterm::event::KeyEvent;
use ratatui::Frame;

use super::LayoutRects;

pub mod help_footer;
pub mod process_details;
pub mod process_table;
pub mod processes_view;
pub mod search_bar;

pub trait Component {
    fn handle_input(&mut self, _: KeyEvent) -> KeyAction {
        KeyAction::Unhandled
    }

    fn handle_event(&mut self, _: &ComponentEvent) -> Option<ComponentEvent> {
        None
    }

    //TODO: I don't like the fact that all componets must have access to some global layout rects
    //each component should have it set in constructor
    fn render(&mut self, _frame: &mut Frame, _layout: &LayoutRects) {}
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
    SearchQueryUpdated(String),
    SearchByTextRequested(String),
    NoProcessToKill,
    ProcessKilled,
    ProcessKillFailed,
    QuitRequested,
}
