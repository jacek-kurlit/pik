use crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Paragraph,
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use super::{Action, Component};

pub struct SearchBarComponent {
    search_area: TextArea<'static>,
}

impl SearchBarComponent {
    pub fn new(search_text: String) -> Self {
        let mut search_area = TextArea::from(search_text.lines());
        search_area.move_cursor(CursorMove::End);
        Self { search_area }
    }

    pub fn set_search_text(&mut self, text: String) {
        self.clear_search_area();
        self.search_area.insert_str(text);
    }

    pub fn clear_search_area(&mut self) {
        self.search_area.move_cursor(CursorMove::Head);
        self.search_area.delete_line_by_end();
    }

    pub fn get_search_text(&self) -> &str {
        &self.search_area.lines()[0]
    }
}

impl Component for SearchBarComponent {
    fn handle_input(&mut self, event: KeyEvent) -> Action {
        match event.code {
            Left => {
                self.search_area.move_cursor(CursorMove::Back);
                Action::Noop
            }
            Right => {
                self.search_area.move_cursor(CursorMove::Forward);
                Action::Noop
            }
            Home => {
                self.search_area.move_cursor(CursorMove::Head);
                Action::Noop
            }
            End => {
                self.search_area.move_cursor(CursorMove::End);
                Action::Noop
            }
            Backspace => {
                self.search_area.delete_char();
                Action::SearchForProcesses(self.search_area.lines().join(""))
            }
            Delete => {
                self.search_area.delete_next_char();
                Action::SearchForProcesses(self.search_area.lines().join(""))
            }
            Char('w') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_area.delete_word();
                Action::SearchForProcesses(self.search_area.lines().join(""))
            }
            Char(c) => {
                self.search_area.insert_char(c);
                Action::SearchForProcesses(self.search_area.lines().join(""))
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let rects = Layout::horizontal([Constraint::Length(2), Constraint::Min(2)]).split(area);
        f.render_widget(Paragraph::new("> "), rects[0]);
        f.render_widget(&self.search_area, rects[1]);
    }
}
