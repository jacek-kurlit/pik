use ratatui::{
    crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    style::{
        Modifier, Style,
        palette::tailwind::{self, SLATE},
    },
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, HighlightSpacing, List, ListState, Padding},
};

use super::{Component, KeyAction};

#[derive(Default)]
pub struct HelpPopupComponent {
    is_open: bool,
    list_state: ListState,
}

impl HelpPopupComponent {
    pub fn new() -> Self {
        Self {
            is_open: false,
            list_state: ListState::default().with_selected(Some(0)),
        }
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
            .padding(Padding::left(1))
            .border_style(Style::new().fg(tailwind::GREEN.c400))
            .border_type(BorderType::Rounded);
        let items = key_mapping_list(&[
            ("<C-x>", "Kill selected process"),
            ("<Esc>", "Close/Quit"),
            ("<C-c>", "Close/Quit"),
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
        ]);
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
            .highlight_spacing(HighlightSpacing::Always);

        let area = popup_area(area, 30, 80);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

//longest key binding
const KEY_PADDING: usize = 8;
fn key_mapping_list(mapping: &[(&'static str, &'static str)]) -> Vec<Line<'static>> {
    let key_style = Style::new().fg(tailwind::BLUE.c400);
    mapping
        .iter()
        .map(|(key, description)| {
            Line::from(vec![
                Span::styled(format!("{:>KEY_PADDING$}  ", key), key_style),
                Span::raw(*description),
            ])
            .left_aligned()
        })
        .collect()
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
