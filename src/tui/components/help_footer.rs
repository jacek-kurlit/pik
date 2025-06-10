use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    config::keymappings::{AppAction, KeyMappings},
    tui::LayoutRects,
};

use super::{Component, ComponentEvent};

pub struct HelpFooterComponent {
    error_message: Option<&'static str>,
    help_bar: Paragraph<'static>,
}

impl HelpFooterComponent {
    pub fn new(keymappings: &KeyMappings) -> Self {
        let quit = keymappings.get_joined(AppAction::Quit, "/");
        let close = keymappings.get_joined(AppAction::Close, "/");
        let kill_process = keymappings.get_joined(AppAction::KillProcess, "/");
        let help_toggle = keymappings.get_joined(AppAction::ToggleHelp, "/");
        let help_bar = Paragraph::new(Line::from(format!(
            "{quit}/{close} quit | {kill_process} kill process | {help_toggle} toggle help"
        )))
        .centered();
        Self {
            error_message: None,
            help_bar,
        }
    }

    pub fn set_error_message(&mut self, message: &'static str) {
        self.error_message = Some(message);
    }

    pub fn reset_error_message(&mut self) {
        self.error_message = None;
    }
}

impl Component for HelpFooterComponent {
    fn render(&mut self, f: &mut ratatui::Frame, layout: &LayoutRects) {
        let rects = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
            .horizontal_margin(1)
            .split(layout.help_text);
        let error = Paragraph::new(Span::from(self.error_message.unwrap_or("")).fg(Color::Red))
            .left_aligned()
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(error, rects[0]);
        f.render_widget(&self.help_bar, rects[1]);
    }

    fn handle_event(&mut self, event: &ComponentEvent) -> Option<ComponentEvent> {
        match event {
            ComponentEvent::ProcessListRefreshed => self.reset_error_message(),

            ComponentEvent::ProcessKilled | ComponentEvent::NoProcessToKill => {
                self.reset_error_message()
            }
            ComponentEvent::ProcessKillFailed => {
                self.set_error_message("Failed to kill process. Check permissions");
            }
            _ => (),
        }

        None
    }
}
