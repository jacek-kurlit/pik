use crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Paragraph,
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use super::{Component, ComponentEvent, KeyAction};

pub struct SearchBarComponent {
    search_area: TextArea<'static>,
}

impl SearchBarComponent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let search_area = TextArea::default();
        Self { search_area }
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

    fn emit_search_updated(&mut self) -> ComponentEvent {
        //FIXME: cloning hurts!
        ComponentEvent::SearchQueryUpdated(self.search_area.lines().join(""))
    }
}

impl Component for SearchBarComponent {
    fn handle_input(&mut self, event: KeyEvent) -> KeyAction {
        match event.code {
            Left => {
                self.search_area.move_cursor(CursorMove::Back);
                KeyAction::Consumed
            }
            Right => {
                self.search_area.move_cursor(CursorMove::Forward);
                KeyAction::Consumed
            }
            Home => {
                self.search_area.move_cursor(CursorMove::Head);
                KeyAction::Consumed
            }
            End => {
                self.search_area.move_cursor(CursorMove::End);
                KeyAction::Consumed
            }
            //TODO: this is sad, we must emit it from search br because search needs query!
            Char('r') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                KeyAction::Event(self.emit_search_updated())
            }
            Backspace => {
                self.search_area.delete_char();
                KeyAction::Event(self.emit_search_updated())
            }
            Delete => {
                self.search_area.delete_next_char();
                KeyAction::Event(self.emit_search_updated())
            }
            Char('w') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_area.delete_word();
                KeyAction::Event(self.emit_search_updated())
            }
            Char(c) => {
                self.search_area.insert_char(c);
                KeyAction::Event(self.emit_search_updated())
            }
            _ => KeyAction::Consumed,
        }
    }

    fn handle_event(&mut self, event: &ComponentEvent) -> Option<ComponentEvent> {
        if let ComponentEvent::SearchByTextRequested(query) = event {
            self.set_search_text(query);
            return Some(self.emit_search_updated());
        }
        None
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        let rects = Layout::horizontal([Constraint::Length(2), Constraint::Min(2)]).split(area);
        f.render_widget(Paragraph::new("> "), rects[0]);
        f.render_widget(&self.search_area, rects[1]);
    }
}
