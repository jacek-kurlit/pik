use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::processes::Process;

pub mod help_footer;
pub mod process_details;
pub mod process_table;
pub mod search_bar;

pub trait Component {
    fn handle_input(&mut self, _: KeyEvent) -> Action {
        Action::Noop
    }

    //TODO: i dont like mut here
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

//TODO: this code smells becuase some of actions are events not commands
pub enum Action {
    Noop,
    Consumed,
    ProcessSelected(Process),
    SearchForProcesses(String),
    SetSearchText(String),
    NoProcessToKill,
    ProcessKilled,
    ProcessKillFailure,
    Quit,
}
