use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Margin},
    prelude::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

use crate::{
    processes::{MatchedBy, Process, ProcessSearchResults, ResultItem},
    tui::{highlight::highlight_text, Theme},
};

use super::{Action, Component};

pub struct ProcessTableComponent {
    theme: Theme,
    //TODO: this should not be pub
    pub search_results: ProcessSearchResults,
    use_icons: bool,
    process_table: TableState,
    process_table_scroll_state: ScrollbarState,
    process_table_number_of_items: usize,
}

const MAX_CMD_LEN: usize = 20;
const MAX_PATH_LEN: usize = 38;
const MAX_ARGS_LEN: usize = 35;
const MAX_PORTS_LEN: usize = 20;

const TABLE_HEADERS_ICONS: [&str; 8] = [
    "USER 󰋦",
    "PID ",
    "PARENT 󱖁",
    "TIME ",
    "CMD 󱃸",
    "PATH ",
    "ARGS 󱃼",
    "PORTS ",
];

const TABLE_HEADERS_PLAIN: [&str; 8] = [
    "USER", "PID", "PARENT", "RUN TIME", "CMD", "PATH", "ARGS", "PORTS",
];

const TABLE_WIDTHS: [Constraint; 8] = [
    Constraint::Percentage(5),
    Constraint::Percentage(5),
    Constraint::Percentage(5),
    Constraint::Percentage(5),
    Constraint::Percentage(10),
    Constraint::Percentage(30),
    Constraint::Percentage(25),
    Constraint::Percentage(15),
];

impl ProcessTableComponent {
    pub fn new(use_icons: bool) -> Self {
        Self {
            process_table: TableState::default(),
            process_table_scroll_state: ScrollbarState::new(0),
            theme: Theme::new(),
            search_results: ProcessSearchResults::empty(),
            use_icons,
            process_table_number_of_items: 0,
        }
    }

    pub fn select_first_row(&mut self) {
        let index = (self.process_table_number_of_items > 0).then_some(0);
        self.select_row_by_index(index);
    }

    pub fn select_last_row(&mut self) {
        let index = self.process_table_number_of_items.checked_sub(1);
        self.select_row_by_index(index);
    }

    pub fn select_next_row(&mut self, step_size: usize) {
        let next_row_index = self.process_table.selected().map(|i| {
            let mut i = i + step_size;
            if i >= self.process_table_number_of_items {
                i = 0
            }
            i
        });
        self.select_row_by_index(next_row_index);
    }

    pub fn select_row_by_index(&mut self, index: Option<usize>) {
        self.process_table.select(index);
        self.process_table_scroll_state =
            self.process_table_scroll_state.position(index.unwrap_or(0));
        //FIXME: this is not the right place to reset the scroll
        // self.reset_process_detals_scroll();
    }

    pub fn select_previous_row(&mut self, step_size: usize) {
        let previous_index = self.process_table.selected().map(|i| {
            let i = i.wrapping_sub(step_size);
            i.clamp(0, self.process_table_number_of_items.saturating_sub(1))
        });
        self.select_row_by_index(previous_index);
    }

    pub fn get_selected_process(&self) -> Option<&Process> {
        let selected_index = self.process_table.selected();
        self.search_results.nth(selected_index)
    }

    pub fn update_process_table_number_of_items(&mut self) {
        let number_of_items = self.search_results.len();
        self.process_table_number_of_items = number_of_items;
        self.process_table_scroll_state = self
            .process_table_scroll_state
            .content_length(number_of_items.saturating_sub(1));
        if number_of_items == 0 {
            self.process_table.select(None);
        } else {
            self.process_table.select(Some(0));
        }
    }

    fn create_line<'a>(
        &self,
        item: &ResultItem,
        text: &'a str,
        matched_by: MatchedBy,
        max_len: usize,
    ) -> Line<'a> {
        if item.is_matched_by(matched_by) {
            highlight_text(
                text,
                &item.match_data.match_type,
                self.theme.highlight_style,
                self.theme.default_style,
                max_len,
            )
        } else {
            Line::from(Span::styled(text, self.theme.default_style))
        }
    }
}

impl Component for ProcessTableComponent {
    fn handle_input(&mut self, key: KeyEvent) -> Action {
        use KeyCode::*;
        match key.code {
            Up if key.modifiers.contains(KeyModifiers::CONTROL) => self.select_first_row(),
            Down if key.modifiers.contains(KeyModifiers::CONTROL) => self.select_last_row(),
            Up | BackTab => self.select_previous_row(1),
            Tab | Down => self.select_next_row(1),
            PageUp => self.select_previous_row(10),
            PageDown => self.select_next_row(10),
            Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_next_row(1);
            }
            Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_previous_row(1);
            }
            // Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            //     self.kill_selected_process()
            // }
            // Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            //     self.search_for_processess()
            // }
            // Char('p') if key.modifiers.contains(KeyModifiers::ALT) => {
            //     self.enforce_search_by(ProcessRelatedSearch::Parent);
            // }
            // Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
            //     self.enforce_search_by(ProcessRelatedSearch::Family);
            // }
            // Char('s') if key.modifiers.contains(KeyModifiers::ALT) => {
            //     self.enforce_search_by(ProcessRelatedSearch::Siblings);
            // }
            _ => return Action::Input(key),
        };
        Action::Noop
    }

    fn render(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let rows = self.search_results.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => self.theme.normal_row_color,
                _ => self.theme.alt_row_color,
            };
            let data = &item.process;
            Row::new(vec![
                Line::from(Span::styled(
                    data.user_name.as_str(),
                    self.theme.default_style,
                )),
                Line::from(Span::styled(
                    format!("{}", data.pid),
                    self.theme.default_style,
                )),
                Line::from(Span::styled(
                    data.parent_as_string(),
                    self.theme.default_style,
                )),
                Line::from(Span::styled(&data.run_time, self.theme.default_style)),
                self.create_line(item, &data.cmd, MatchedBy::Cmd, MAX_CMD_LEN),
                self.create_line(
                    item,
                    data.cmd_path.as_deref().unwrap_or(""),
                    MatchedBy::Path,
                    MAX_PATH_LEN,
                ),
                self.create_line(item, &data.args, MatchedBy::Args, MAX_ARGS_LEN),
                self.create_line(
                    item,
                    data.ports.as_deref().unwrap_or(""),
                    MatchedBy::Port,
                    MAX_PORTS_LEN,
                ),
            ])
            .style(Style::new().fg(self.theme.row_fg).bg(color))
        });
        let headers = if self.use_icons {
            TABLE_HEADERS_ICONS
        } else {
            TABLE_HEADERS_PLAIN
        };
        let table = Table::new(rows, TABLE_WIDTHS)
            .header(Row::new(headers))
            .block(
                Block::default()
                    .title_top(
                        Line::from(format!(
                            " {} / {} ",
                            self.process_table.selected().map(|i| i + 1).unwrap_or(0),
                            self.search_results.len()
                        ))
                        .left_aligned(),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(self.theme.process_table_border_color))
                    .border_type(BorderType::Plain),
            )
            .row_highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(self.theme.selected_style_fg),
            )
            .highlight_symbol(Text::from(vec![" ".into()]))
            .highlight_spacing(HighlightSpacing::Always);
        f.render_stateful_widget(table, area, &mut self.process_table);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.process_table_scroll_state,
        );
    }
}
