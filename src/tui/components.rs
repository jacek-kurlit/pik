use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod help_footer;
pub mod process_table;
pub mod search_bar;
pub mod process_details;

pub trait Component {
    fn handle_input(&mut self, event: KeyEvent) -> Action {
        Action::Input(event)
    }

    //TODO: i dont like mut here
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub enum Action {
    Input(KeyEvent),
    Noop,
    SearchForProcesses(String),
    Quit,
}
