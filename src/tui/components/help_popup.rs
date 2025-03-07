use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{
        Modifier, Style,
        palette::tailwind::{self, SLATE},
    },
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, HighlightSpacing, List, ListState, Padding},
};

use super::Component;

#[derive(Default)]
pub struct HelpPopupComponent {
    is_open: bool,
    list_state: ListState,
}

impl Component for HelpPopupComponent {
    fn render(&mut self, frame: &mut ratatui::Frame, _layout: &crate::tui::LayoutRects) {
        //TODO: TBD
        if !self.is_open {
            return;
        }
        let area = frame.area();
        self.list_state.select_first();

        //TODO: user may have full screen so this should be dynamic
        //on full screen we may not need scroll at all
        let block = Block::bordered()
            .title_top(Line::from("Keybindings").centered())
            .title_bottom(Line::from("Press <Esc> to close").centered())
            .padding(Padding::left(1))
            .border_style(Style::new().fg(tailwind::GREEN.c400))
            .border_type(BorderType::Rounded);
        // Create a List from all list items and highlight the currently selected one
        //TODO: consider to add categories like process table/search bar etc
        let items = key_mapping_list(&[
            ("<C-x>", "Kill selected process"),
            ("<Esc>", "Close/Quit"),
            ("<C-c>", "Close/Quit"),
            ("<C-r>", "Refresh process list"),
            ("<C-f>", "Process details scroll forward"),
            ("<C-b>", "Process details scroll backward"),
            ("<Tab>", "Select next process"),
            ("<S-Tab>", "Select previous process"),
            ("<C-j>", "Select next process"),
            ("<C-k>", "Select previous process"),
            ("↓", "Select next process"),
            ("↑", "Select previous process"),
            ("<C-↓>", "Select last process"),
            ("<C-↑>", "Select frist process"),
            ("<PgDn>", "Jump 10 processes forward"),
            ("<PgUp>", "Jump 10 processes backward"),
            ("<A-p>", "Select parent process"),
            ("<A-f>", "Select process family"),
            ("<A-s>", "Select siblings processes"),
            ("<C-h>", "Toggle help popup"),
        ]);
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        let area = popup_area(area, 30, 80);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

//longest key binding
const KEY_PADDING: usize = 7;
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
