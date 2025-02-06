use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod help_footer;
pub mod process_table;
pub mod search_bar;

pub trait Component {
    fn handle_input(&mut self, event: KeyEvent) -> Action {
        Action::Input(event)
    }

    fn render(&self, frame: &mut Frame, area: Rect);
}

pub enum Action {
    Input(KeyEvent),
    Noop,
    SearchForProcesses(String),
    Quit,
}
