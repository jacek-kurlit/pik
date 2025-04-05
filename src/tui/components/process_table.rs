use ratatui::{
    layout::Constraint,
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState,
    },
};

use crate::{
    config::ui::{IconsStruct, TableTheme},
    processes::{MatchedBy, ProcessSearchResults, ResultItem},
    tui::{LayoutRects, highlight::highlight_text},
};

pub struct ProcessTableComponent {
    headers: Vec<String>,
    theme: TableTheme,
    process_table: TableState,
    process_table_scroll_state: ScrollbarState,
}

const MAX_CMD_LEN: usize = 20;
const MAX_PATH_LEN: usize = 38;
const MAX_ARGS_LEN: usize = 35;
const MAX_PORTS_LEN: usize = 20;

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
    pub fn new(icons: &IconsStruct, theme: TableTheme) -> Self {
        Self {
            process_table: TableState::default(),
            process_table_scroll_state: ScrollbarState::new(0),
            theme,
            headers: vec![
                format!("USER {}", icons.user).trim().to_string(),
                format!("PID {}", icons.pid).trim().to_string(),
                format!("PARENT {}", icons.parent).trim().to_string(),
                format!("TIME {}", icons.time).trim().to_string(),
                format!("CMD {}", icons.cmd).trim().to_string(),
                format!("PATH {}", icons.path).trim().to_string(),
                format!("ARGS {}", icons.args).trim().to_string(),
                format!("PORTS {}", icons.ports).trim().to_string(),
            ],
        }
    }

    pub fn select_first_row(&mut self, number_of_items: usize) {
        let index = (number_of_items > 0).then_some(0);
        self.select_row_by_index(index)
    }

    pub fn select_last_row(&mut self, number_of_items: usize) {
        let index = number_of_items.checked_sub(1);
        self.select_row_by_index(index)
    }

    pub fn select_next_row(&mut self, step_size: usize, number_of_items: usize) {
        let next_row_index = self.process_table.selected().map(|i| {
            let mut i = i + step_size;
            if i >= number_of_items {
                i = 0
            }
            i
        });
        self.select_row_by_index(next_row_index)
    }

    pub fn select_previous_row(&mut self, step_size: usize, number_of_items: usize) {
        let previous_index = self.process_table.selected().map(|i| {
            let i = i.wrapping_sub(step_size);
            i.clamp(0, number_of_items.saturating_sub(1))
        });
        self.select_row_by_index(previous_index)
    }

    pub fn select_row_by_index(&mut self, index: Option<usize>) {
        self.process_table.select(index);
        self.process_table_scroll_state =
            self.process_table_scroll_state.position(index.unwrap_or(0));
    }

    pub fn get_selected_process_index(&self) -> Option<usize> {
        self.process_table.selected()
    }

    pub fn update_process_table_state(&mut self, number_of_items: usize) {
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
                self.theme.cell.highlighted,
                self.theme.cell.normal,
                max_len,
            )
        } else {
            Line::from(Span::styled(text, self.theme.cell.normal))
        }
    }

    pub fn render(
        &mut self,
        f: &mut ratatui::Frame,
        layout: &LayoutRects,
        search_results: &ProcessSearchResults,
    ) {
        let area = layout.process_table;
        let rows = search_results.iter().enumerate().map(|(i, item)| {
            let row_style = match i % 2 {
                0 => self.theme.row.even,
                _ => self.theme.row.odd,
            };
            let data = &item.process;
            Row::new(vec![
                Line::from(Span::styled(
                    data.user_name.as_str(),
                    self.theme.cell.normal,
                )),
                Line::from(Span::styled(
                    format!("{}", data.pid),
                    self.theme.cell.normal,
                )),
                Line::from(Span::styled(
                    data.parent_as_string(),
                    self.theme.cell.normal,
                )),
                Line::from(Span::styled(&data.run_time, self.theme.cell.normal)),
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
            .style(row_style)
        });
        let table = Table::new(rows, TABLE_WIDTHS)
            .header(Row::new(self.headers.iter().map(|r| r.as_str())))
            .block(
                Block::default()
                    .title_position(self.theme.title.position)
                    .title_alignment(self.theme.title.alignment)
                    .title(Line::from(format!(
                        " {} / {} ",
                        self.process_table.selected().map(|i| i + 1).unwrap_or(0),
                        search_results.len()
                    )))
                    .borders(Borders::ALL)
                    .border_style(self.theme.border.style)
                    .border_type(self.theme.border._type),
            )
            .row_highlight_style(self.theme.row.selected)
            .highlight_symbol(self.theme.row.selected_symbol.as_str())
            .highlight_spacing(HighlightSpacing::Always);
        f.render_stateful_widget(table, area, &mut self.process_table);

        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(self.theme.scrollbar.style)
                .thumb_symbol(self.theme.scrollbar.thumb_symbol.as_deref().unwrap_or(""))
                .track_symbol(self.theme.scrollbar.track_symbol.as_deref())
                .begin_symbol(self.theme.scrollbar.begin_symbol.as_deref())
                .end_symbol(self.theme.scrollbar.end_symbol.as_deref()),
            area.inner(self.theme.scrollbar.margin),
            &mut self.process_table_scroll_state,
        );
    }
}
