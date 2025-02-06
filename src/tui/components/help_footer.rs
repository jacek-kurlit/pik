use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::Component;

#[derive(Default)]
pub struct HelpFooterComponent {
    error_message: Option<&'static str>,
}

impl HelpFooterComponent {
    //TODO: should be event?
    pub fn set_error_message(&mut self, message: &'static str) {
        self.error_message = Some(message);
    }

    pub fn reset_error_message(&mut self) {
        self.error_message = None;
    }
}

const HELP_TEXT: &str =
    "ESC/<C+C> quit | <C+X> kill process | <C+R> refresh | <C+F> details forward | <C+B> details backward ";

impl Component for HelpFooterComponent {
    fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let rects = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
            .horizontal_margin(1)
            .split(area);
        let error = Paragraph::new(Span::from(self.error_message.unwrap_or("")).fg(Color::Red))
            .left_aligned()
            .block(Block::default().borders(Borders::NONE));
        let help = Paragraph::new(Line::from(HELP_TEXT)).right_aligned();
        f.render_widget(error, rects[0]);
        f.render_widget(help, rects[1]);
    }
}
