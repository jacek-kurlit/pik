use ratatui::{text::Line, widgets::Paragraph};

use crate::{
    config::keymappings::{AppAction, KeyMappings},
    tui::LayoutRects,
};

use super::Component;

pub struct HelpFooterComponent {
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
        Self { help_bar }
    }
}

impl Component for HelpFooterComponent {
    fn render(&mut self, f: &mut ratatui::Frame, layout: &LayoutRects) {
        f.render_widget(&self.help_bar, layout.help_text);
    }
}
