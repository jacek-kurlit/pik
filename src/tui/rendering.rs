use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
};

use crate::processes::{MatchedBy, Process, ProcessSearchResults, ResultItem};

use super::highlight::highlight_text;

struct Theme {
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    process_table_border_color: Color,
    highlight_style: Style,
    default_style: Style,
}

impl Theme {
    fn new() -> Self {
        Self {
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: tailwind::BLUE.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            process_table_border_color: tailwind::BLUE.c400,
            highlight_style: Style::new().bg(Color::Yellow).fg(Color::Black),
            default_style: Style::default(),
        }
    }
}

pub struct Tui {
    theme: Theme,
    use_icons: bool,
    process_table: TableState,
    process_table_scroll_state: ScrollbarState,
    process_table_number_of_items: usize,
    process_details_scroll_state: ScrollbarState,
    process_details_scroll_offset: u16,
    process_details_number_of_lines: u16,
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

impl Tui {
    pub fn new(use_icons: bool) -> Self {
        Self {
            process_table: TableState::default(),
            process_table_scroll_state: ScrollbarState::new(0),
            theme: Theme::new(),
            use_icons,
            process_table_number_of_items: 0,
            process_details_scroll_offset: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            process_details_scroll_state: ScrollbarState::new(1),
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
        self.reset_process_detals_scroll();
    }

    pub fn select_previous_row(&mut self, step_size: usize) {
        let previous_index = self.process_table.selected().map(|i| {
            let i = i.wrapping_sub(step_size);
            i.clamp(0, self.process_table_number_of_items.saturating_sub(1))
        });
        self.select_row_by_index(previous_index);
    }

    pub fn process_details_down(&mut self, frame: &mut Frame) {
        let rects = layout_rects(frame);
        let process_details_area = rects[2];
        let area_content_height = process_details_area.height - 2;
        let content_scrolled =
            self.process_details_number_of_lines - self.process_details_scroll_offset;

        if content_scrolled > area_content_height {
            self.process_details_scroll_offset =
                self.process_details_scroll_offset.saturating_add(1);
        }
    }

    pub fn process_details_up(&mut self) {
        self.process_details_scroll_offset = self.process_details_scroll_offset.saturating_sub(1);
    }

    fn reset_process_detals_scroll(&mut self) {
        self.process_details_scroll_offset = 0;
    }

    pub fn get_selected_row_index(&self) -> Option<usize> {
        self.process_table.selected()
    }

    pub fn update_process_table_number_of_items(&mut self, number_of_items: usize) {
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

    pub fn render_ui(
        &mut self,
        search_results: &ProcessSearchResults,
        frame: &mut Frame,
        rects: Rc<[Rect]>,
    ) {
        self.render_process_table(frame, search_results, rects[1]);
        self.render_process_details(frame, search_results, rects[2]);
    }

    fn render_process_table(
        &mut self,
        f: &mut Frame,
        search_results: &ProcessSearchResults,
        area: Rect,
    ) {
        let rows = search_results.iter().enumerate().map(|(i, item)| {
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
                            search_results.len()
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

    fn render_process_details(
        &mut self,
        f: &mut Frame,
        search_results: &ProcessSearchResults,
        area: Rect,
    ) {
        let selected_process = search_results.nth(self.get_selected_row_index());
        let lines = process_details_lines(selected_process);

        self.update_process_details_number_of_lines(area, selected_process);

        let info_footer = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .left_aligned()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_top(Line::from(" Process Details ").left_aligned())
                    .border_type(BorderType::Rounded),
            )
            .scroll((self.process_details_scroll_offset, 0));
        f.render_widget(info_footer, area);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .thumb_symbol("")
                .track_symbol(None)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.process_details_scroll_state,
        );
    }

    fn update_process_details_number_of_lines(
        &mut self,
        area: Rect,
        selected_process: Option<&Process>,
    ) {
        let content_width = area.width - 2;

        match selected_process {
            Some(process) => {
                let args_number_of_lines =
                    (process.args.chars().count() as u16 / content_width) + 1;
                self.process_details_number_of_lines = args_number_of_lines + 2;
            }
            None => {
                self.process_details_number_of_lines = 1;
            }
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

fn process_details_lines(selected_process: Option<&Process>) -> Vec<Line> {
    match selected_process {
        Some(prc) => {
            let ports = prc
                .ports
                .as_deref()
                .map(|p| format!(" PORTS: {}", p))
                .unwrap_or("".to_string());
            let parent = prc
                .parent_pid
                .map(|p| format!(" PARENT: {}", p))
                .unwrap_or("".to_string());
            vec![
                Line::from(format!(
                    "USER: {} PID: {}{} START TIME: {}, RUN TIME: {} MEMORY: {}MB{}",
                    prc.user_name,
                    prc.pid,
                    parent,
                    prc.start_time,
                    prc.run_time,
                    prc.memory / 1024 / 1024,
                    ports,
                )),
                Line::from(format!("CMD: {}", prc.exe())),
                Line::from(format!("ARGS: {}", prc.args)),
            ]
        }
        None => vec![Line::from("No process selected")],
    }
}

fn layout_rects(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(7),
        Constraint::Length(1),
    ])
    .split(frame.area())
}
