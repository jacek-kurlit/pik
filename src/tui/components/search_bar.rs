use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Paragraph,
};
use tui_textarea::{CursorMove, TextArea};

use crate::{config::ui::SearchBarTheme, tui::LayoutRects};

pub struct SearchBarComponent {
    search_area: TextArea<'static>,
    prompt_box: Paragraph<'static>,
    prompt_size: u16,
}

impl SearchBarComponent {
    #[allow(clippy::new_without_default)]
    pub fn new(initial_query: String, theme: &SearchBarTheme, prompt_icon: &str) -> Self {
        let mut search_area = TextArea::default();
        search_area.set_cursor_line_style(theme.style);
        search_area.set_cursor_style(theme.cursor_style);
        search_area.insert_str(&initial_query);
        let prompt = format!("{prompt_icon} ");
        Self {
            search_area,
            prompt_size: unicode_width::UnicodeWidthStr::width_cjk(prompt.as_str()) as u16,
            prompt_box: Paragraph::new(prompt).style(theme.style),
        }
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

    pub fn delete_to_start(&mut self) {
        self.search_area.delete_line_by_head();
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
        let rects = Layout::horizontal([Constraint::Length(self.prompt_size), Constraint::Min(2)])
            .split(layout.top_bar);
        f.render_widget(&self.prompt_box, rects[0]);
        f.render_widget(&self.search_area, rects[1]);
    }
}
