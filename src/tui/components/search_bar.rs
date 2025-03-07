use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Paragraph,
};
use tui_textarea::{CursorMove, TextArea};

use crate::tui::LayoutRects;

pub struct SearchBarComponent {
    search_area: TextArea<'static>,
}

impl SearchBarComponent {
    #[allow(clippy::new_without_default)]
    pub fn new(initial_query: String) -> Self {
        let mut search_area = TextArea::default();
        search_area.insert_str(&initial_query);
        Self { search_area }
    }

    pub fn move_cursor(&mut self, direction: CursorMove) {
        self.search_area.move_cursor(direction);
    }

    pub fn insert_char(&mut self, c: char) {
        self.search_area.insert_char(c);
    }

    pub fn delete_char(&mut self) {
        self.search_area.delete_char();
    }

    pub fn delete_next_char(&mut self) {
        self.search_area.delete_next_char();
    }

    pub fn delete_word(&mut self) {
        self.search_area.delete_word();
    }

    pub fn set_search_text(&mut self, text: &str) {
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

    pub fn render(&mut self, f: &mut Frame, layout: &LayoutRects) {
        let rects =
            Layout::horizontal([Constraint::Length(2), Constraint::Min(2)]).split(layout.top_bar);
        f.render_widget(Paragraph::new("> "), rects[0]);
        f.render_widget(&self.search_area, rects[1]);
    }
}
