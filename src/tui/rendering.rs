use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
};
use tui_textarea::TextArea;

use crate::processes::{Process, ProcessSearchResults, SearchBy};

pub struct Theme {
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    process_table_border_color: Color,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: tailwind::BLUE.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            process_table_border_color: tailwind::BLUE.c400,
        }
    }
}

pub struct Tui {
    process_table: TableState,
    process_table_scroll: ScrollbarState,
    theme: Theme,
    number_of_items: usize,
    details_scroll_state: ScrollbarState,
    process_details_scroll: u16,
    search_area: TextArea<'static>,
    error_message: Option<&'static str>,
}

impl Tui {
    pub fn new(search_text: String) -> Self {
        let mut search_area = TextArea::from(search_text.lines());
        search_area.move_cursor(tui_textarea::CursorMove::End);
        Self {
            process_table: TableState::default(),
            process_table_scroll: ScrollbarState::new(0),
            theme: Theme::new(),
            number_of_items: 0,
            process_details_scroll: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            details_scroll_state: ScrollbarState::new(1),
            search_area,
            error_message: None,
        }
    }

    pub fn select_next_row(&mut self) {
        let next_row_index = self.process_table.selected().map(|i| {
            let mut i = i + 1;
            if i >= self.number_of_items {
                i = 0
            }
            i
        });
        self.process_table.select(next_row_index);
        self.process_table_scroll = self
            .process_table_scroll
            .position(next_row_index.unwrap_or(0));
        self.reset_process_detals_scroll();
    }

    pub fn select_previous_row(&mut self) {
        let previous_index = self.process_table.selected().map(|i| {
            let i = i.wrapping_sub(1);
            i.clamp(0, self.number_of_items.saturating_sub(1))
        });
        self.process_table.select(previous_index);
        self.process_table_scroll = self
            .process_table_scroll
            .position(previous_index.unwrap_or(0));
        self.reset_process_detals_scroll();
    }

    pub fn handle_input(&mut self, input: KeyEvent) {
        self.search_area.input(input);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.search_area.insert_char(new_char);
    }

    pub fn process_details_down(&mut self) {
        self.process_details_scroll = self.process_details_scroll.saturating_add(1);
    }

    pub fn process_details_up(&mut self) {
        self.process_details_scroll = self.process_details_scroll.saturating_sub(1);
    }

    fn reset_process_detals_scroll(&mut self) {
        self.process_details_scroll = 0;
    }

    pub fn set_error_message(&mut self, message: &'static str) {
        self.error_message = Some(message);
    }

    pub fn reset_error_message(&mut self) {
        self.error_message = None;
    }

    pub fn delete_char(&mut self) {
        self.search_area.delete_char();
    }

    pub fn get_selected_row_index(&self) -> Option<usize> {
        self.process_table.selected()
    }

    pub fn update_number_of_items(&mut self, number_of_items: usize) {
        self.number_of_items = number_of_items;
        self.process_table_scroll = self
            .process_table_scroll
            .content_length(number_of_items.saturating_sub(1));
        if number_of_items == 0 {
            self.process_table.select(None);
        } else {
            self.process_table.select(Some(0));
        }
    }

    pub fn search_input_text(&self) -> &str {
        &self.search_area.lines()[0]
    }

    pub fn render_ui(&mut self, search_results: &ProcessSearchResults, frame: &mut Frame) {
        let rects = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Max(7),
            Constraint::Length(1),
        ])
        .split(frame.area());

        self.render_search_input(frame, rects[0]);

        self.render_process_table(frame, search_results, rects[1]);

        self.render_process_details(frame, search_results, rects[2]);

        render_help(frame, self.error_message, rects[3]);
    }

    fn render_search_input(&self, f: &mut Frame, area: Rect) {
        let rects = Layout::horizontal([Constraint::Length(2), Constraint::Min(2)]).split(area);
        f.render_widget(Paragraph::new("> "), rects[0]);
        f.render_widget(&self.search_area, rects[1]);
    }

    fn render_process_table(
        &mut self,
        f: &mut Frame,
        search_results: &ProcessSearchResults,
        area: Rect,
    ) {
        let (dynamic_header, value_getter) = dynamic_search_column(search_results);
        let rows = search_results.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.theme.normal_row_color,
                _ => self.theme.alt_row_color,
            };
            //TODO: think about creating this row without allocations
            Row::new(vec![
                format!("{}", data.user_name),
                format!("{}", data.pid),
                format!("{}", data.parent_as_string()),
                format!("{}", data.start_time),
                format!("{}", data.run_time),
                format!("{}", data.cmd),
                format!("{}", data.cmd_path.as_deref().unwrap_or("")),
                format!("{}", value_getter(data)),
            ])
            .style(Style::new().fg(self.theme.row_fg).bg(color))
        });
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(10),
                Constraint::Percentage(25),
                Constraint::Percentage(40),
            ],
        )
        .header(Row::new(vec![
            "USER",
            "PID",
            "PARENT",
            "STARTED",
            "TIME",
            "CMD",
            "CMD_PATH",
            dynamic_header,
        ]))
        .block(
            Block::default()
                .title(
                    Title::from(format!(
                        " {} / {} ",
                        self.process_table.selected().map(|i| i + 1).unwrap_or(0),
                        search_results.len()
                    ))
                    .position(Position::Top)
                    .alignment(Alignment::Left),
                )
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.theme.process_table_border_color))
                .border_type(BorderType::Plain),
        )
        .highlight_style(
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
            &mut self.process_table_scroll,
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
        let info_footer = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .left_aligned()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(
                        Title::from(" Process Details ")
                            .alignment(Alignment::Left)
                            .position(Position::Top),
                    )
                    // .border_style(Style::new().fg(app.colors.footer_border_color))
                    .border_type(BorderType::Rounded),
            )
            .scroll((self.process_details_scroll, 0));
        f.render_widget(info_footer, area);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .thumb_symbol("")
                .track_symbol(None)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.details_scroll_state,
        );
    }
}

fn dynamic_search_column(search_result: &ProcessSearchResults) -> (&str, fn(&Process) -> &str) {
    match search_result.search_by {
        SearchBy::Port => ("PORT", |prc| prc.ports.as_deref().unwrap_or("")),
        SearchBy::Args => ("ARGS", |prc| prc.args.as_str()),
        _ => ("", |_| ""),
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
                    "USER: {} PID: {}{} START_TIME: {}, RUN_TIME: {} MEMORY: {}MB{}",
                    prc.user_name,
                    prc.pid,
                    parent,
                    prc.start_time,
                    prc.run_time,
                    prc.memory / 1024 / 1024,
                    ports,
                )),
                Line::from(format!("CMD: {}", prc.exe())),
                //FIXME: Sometimes args are too long and don't fit in details area
                Line::from(format!("ARGS: {}", prc.args)),
            ]
        }
        None => vec![Line::from("No process selected")],
    }
}

const HELP_TEXT: &str =
    "ESC quit | <C+X> kill process | <C+R> refresh | <C+F> details forward | <C+B> details backward ";

fn render_help(f: &mut Frame, error_message: Option<&str>, area: Rect) {
    let rects = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
        .horizontal_margin(1)
        .split(area);
    let error = Paragraph::new(Span::from(error_message.unwrap_or("")).fg(Color::Red))
        .left_aligned()
        .block(Block::default().borders(Borders::NONE));
    let help = Paragraph::new(Line::from(HELP_TEXT)).right_aligned();
    f.render_widget(error, rects[0]);
    f.render_widget(help, rects[1]);
}
