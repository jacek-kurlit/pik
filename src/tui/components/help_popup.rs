use itertools::Itertools;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, HighlightSpacing, List, ListState, Padding},
};

use crate::config::{
    keymappings::{AppAction, KeyMappings},
    ui::UIConfig,
};

use super::{Component, KeyAction};

//longest key binding
const KEY_PADDING: usize = 28;

#[derive(Default)]
pub struct HelpPopupComponent {
    is_open: bool,
    list_state: ListState,
    popup_content: List<'static>,
}

impl HelpPopupComponent {
    pub fn new(ui_config: &UIConfig, key_mappings: &KeyMappings) -> Self {
        let theme = &ui_config.popups;
        let esc_bindings = key_mappings.get_joined(AppAction::Close, "/");
        let popup_content = key_mappings
            .sorted()
            .map(|(key, bindings)| {
                Line::from(vec![
                    Span::styled(format!("{key:>KEY_PADDING$}: "), theme.primary),
                    Span::styled(
                        bindings.iter().map(|b| b.to_string()).join(", "),
                        theme.secondary,
                    ),
                ])
                .left_aligned()
            })
            .collect::<List>()
            .block(
                Block::bordered()
                    .title_top(Line::from(" Keybindings ").centered())
                    .title_bottom(Line::from(format!(" Press {esc_bindings} to close ")).centered())
                    .padding(Padding {
                        left: 1,
                        right: 1,
                        top: 0,
                        bottom: 0,
                    })
                    .border_style(theme.border.style)
                    .border_type(theme.border._type),
            )
            .highlight_style(theme.selected_row)
            .highlight_spacing(HighlightSpacing::Always);

        Self {
            is_open: false,
            list_state: ListState::default().with_selected(Some(0)),
            popup_content,
        }
    }
}

impl Component for HelpPopupComponent {
    fn handle_input(&mut self, _: KeyEvent, action: AppAction) -> KeyAction {
        if matches!(action, AppAction::ToggleHelp) {
            self.is_open = !self.is_open;
            return KeyAction::Consumed;
        }
        if !self.is_open {
            return KeyAction::Unhandled;
        }
        match action {
            AppAction::GoToFirstItem => {
                self.list_state.select_first();
            }
            AppAction::GoToLastItem => {
                self.list_state.select_last();
            }
            AppAction::NextItem => {
                self.list_state.select_next();
            }
            AppAction::PreviousItem => {
                self.list_state.select_previous();
            }
            AppAction::Close => {
                self.is_open = false;
            }
            _ => (),
        };

        //consume all keys if popup is open
        KeyAction::Consumed
    }

    fn render(&mut self, frame: &mut ratatui::Frame, _layout: &crate::tui::LayoutRects) {
        if !self.is_open {
            return;
        }
        let area = popup_area(frame.area(), 35, 80);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_stateful_widget(&self.popup_content, area, &mut self.list_state);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
