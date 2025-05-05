use ratatui::{
    crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, HighlightSpacing, List, ListState, Padding},
};

use crate::config::ui::{PopupsTheme, UIConfig};

use super::{Component, KeyAction};

//longest key binding
const KEY_PADDING: usize = 8;

#[derive(Default)]
pub struct HelpPopupComponent {
    is_open: bool,
    list_state: ListState,
    key_mappings: &'static [(&'static str, &'static str)],
    theme: PopupsTheme,
}

impl HelpPopupComponent {
    pub fn new(ui_config: &UIConfig) -> Self {
        let key_mappings = &[
            ("<C-x>", "Kill selected process"),
            ("<Esc>", "Close/Quit"),
            ("<C-c>", "Close/Quit"),
            ("<C-y>", "Copy selected process pid"),
            ("<C-h>", "Toggle help popup"),
            ("<C-r>", "Refresh process list"),
            ("<C-f>", "Process details scroll forward"),
            ("<C-b>", "Process details scroll backward"),
            ("<Tab>", "Select next"),
            ("<S-Tab>", "Select previous"),
            ("<C-j>", "Select next"),
            ("<C-k>", "Select previous"),
            ("↓", "Select next"),
            ("↑", "Select previous"),
            ("<C-↓>", "Select last"),
            ("<C-↑>", "Select frist"),
            ("<C-End>", "Select last"),
            ("<C-Home>", "Select frist"),
            ("<PgDn>", "Jump 10 items forward"),
            ("<PgUp>", "Jump 10 items backward"),
            ("<A-p>", "Select parent process"),
            ("<A-f>", "Select process family"),
            ("<A-s>", "Select siblings processes"),
        ];
        Self {
            is_open: false,
            list_state: ListState::default().with_selected(Some(0)),
            key_mappings,
            theme: ui_config.popups.clone(),
        }
    }

    fn create_list_items(&self) -> Vec<Line<'static>> {
        self.key_mappings
            .iter()
            .map(|(key, description)| {
                Line::from(vec![
                    Span::styled(format!("{:>KEY_PADDING$}  ", key), self.theme.primary),
                    Span::styled(*description, self.theme.secondary),
                ])
                .left_aligned()
            })
            .collect()
    }
}

impl Component for HelpPopupComponent {
    fn handle_input(&mut self, key: KeyEvent) -> KeyAction {
        if matches!(key.code, Char('h') if key.modifiers.contains(KeyModifiers::CONTROL)) {
            self.is_open = !self.is_open;
            return KeyAction::Consumed;
        }
        if !self.is_open {
            return KeyAction::Unhandled;
        }
        match key.code {
            Up | Home if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.list_state.select_first()
            }
            Down | End if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.list_state.select_last()
            }
            Up | BackTab => self.list_state.select_previous(),
            Down | Tab => self.list_state.select_next(),
            Char('j') | Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.list_state.select_next();
            }
            Char('k') | Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.list_state.select_previous();
            }
            Esc => {
                self.is_open = false;
            }
            _ => (),
        }
        //consume all keys if popup is open
        KeyAction::Consumed
    }

    fn render(&mut self, frame: &mut ratatui::Frame, _layout: &crate::tui::LayoutRects) {
        if !self.is_open {
            return;
        }
        let area = frame.area();

        let block = Block::bordered()
            .title_top(Line::from(" Keybindings ").centered())
            .title_bottom(Line::from(" Press <Esc> to close ").centered())
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            })
            .border_style(self.theme.border.style)
            .border_type(self.theme.border._type);
        let list = List::new(self.create_list_items())
            .block(block)
            .highlight_style(self.theme.selected_row)
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_widget(Clear, area); //this clears out the background
        let area = popup_area(area, 35, 80);
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
