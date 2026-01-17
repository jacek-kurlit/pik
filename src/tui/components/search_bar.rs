use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Paragraph,
};
use tui_input::{Input, InputRequest};

use crate::{config::ui::SearchBarTheme, tui::LayoutRects};

pub struct SearchBarComponent {
    search_area: Input,
    prompt_box: Paragraph<'static>,
    prompt_size: u16,
    theme: SearchBarTheme,
}

pub enum CursorMove {
    Back,
    Forward,
    Head,
    End,
    WordBack,
    WordForward,
}

impl SearchBarComponent {
    #[allow(clippy::new_without_default)]
    pub fn new(initial_query: String, theme: &SearchBarTheme, prompt_icon: &str) -> Self {
        let search_area = Input::from(initial_query);
        let prompt = format!("{prompt_icon} ");
        Self {
            search_area,
            prompt_size: unicode_width::UnicodeWidthStr::width_cjk(prompt.as_str()) as u16,
            prompt_box: Paragraph::new(prompt).style(theme.style),
            theme: theme.clone(),
        }
    }

    pub fn request_input_change(&mut self, request: InputRequest) {
        self.search_area.handle(request);
    }

    pub fn move_cursor(&mut self, direction: CursorMove) {
        let request = match direction {
            CursorMove::Back => InputRequest::GoToPrevChar,
            CursorMove::Forward => InputRequest::GoToNextChar,
            CursorMove::Head => InputRequest::GoToStart,
            CursorMove::End => InputRequest::GoToEnd,
            CursorMove::WordBack => InputRequest::GoToPrevWord,
            CursorMove::WordForward => InputRequest::GoToNextWord,
        };
        self.search_area.handle(request);
    }

    pub fn insert_char(&mut self, c: char) {
        self.search_area.handle(InputRequest::InsertChar(c));
    }

    pub fn delete_char(&mut self) {
        self.search_area.handle(InputRequest::DeletePrevChar);
    }

    pub fn delete_next_char(&mut self) {
        self.search_area.handle(InputRequest::DeleteNextChar);
    }

    pub fn delete_word(&mut self) {
        self.search_area.handle(InputRequest::DeletePrevWord);
    }

    pub fn delete_next_word(&mut self) {
        self.search_area.handle(InputRequest::DeleteNextWord);
    }

    //TODO: this is breaking change because it deletes whole line!
    pub fn delete_to_start(&mut self) {
        self.search_area.handle(InputRequest::DeleteLine);
    }

    pub fn delete_to_end(&mut self) {
        self.search_area.handle(InputRequest::DeleteTillEnd);
    }

    pub fn set_search_text(&mut self, text: &str) {
        self.search_area = Input::from(text);
    }

    pub fn get_search_text(&self) -> &str {
        self.search_area.value()
    }

    pub fn render(&mut self, f: &mut Frame, layout: &LayoutRects) {
        let rects = Layout::horizontal([Constraint::Length(self.prompt_size), Constraint::Min(2)])
            .split(layout.top_bar);
        f.render_widget(&self.prompt_box, rects[0]);
        let input = Paragraph::new(self.search_area.value()).style(self.theme.style);
        let x = self.search_area.visual_cursor();
        f.set_cursor_position((rects[1].x + x as u16, rects[1].y));
        f.render_widget(input, rects[1]);
    }
}
