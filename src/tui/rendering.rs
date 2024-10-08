use std::{borrow::Cow, rc::Rc};

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
    theme: Theme,
    process_table: TableState,
    process_table_scroll_state: ScrollbarState,
    process_table_number_of_items: usize,
    process_details_scroll_state: ScrollbarState,
    process_details_scroll_offset: u16,
    process_details_number_of_lines: u16,
    search_area: TextArea<'static>,
    error_message: Option<&'static str>,
}

impl Tui {
    pub fn new(search_text: String) -> Self {
        let mut search_area = TextArea::from(search_text.lines());
        search_area.move_cursor(tui_textarea::CursorMove::End);
        Self {
            process_table: TableState::default(),
            process_table_scroll_state: ScrollbarState::new(0),
            theme: Theme::new(),
            process_table_number_of_items: 0,
            process_details_scroll_offset: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            process_details_scroll_state: ScrollbarState::new(1),
            search_area,
            error_message: None,
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

    pub fn handle_input(&mut self, input: KeyEvent) {
        self.search_area.input(input);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.search_area.insert_char(new_char);
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

    pub fn search_input_text(&self) -> &str {
        &self.search_area.lines()[0]
    }

    pub fn render_ui(&mut self, search_results: &ProcessSearchResults, frame: &mut Frame) {
        let rects = layout_rects(frame);

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
            Row::new(vec![
                Cow::Borrowed(data.user_name.as_str()),
                Cow::Owned(format!("{}", data.pid)),
                Cow::Owned(data.parent_as_string()),
                Cow::Borrowed(&data.start_time),
                Cow::Borrowed(&data.run_time),
                Cow::Borrowed(&data.cmd),
                Cow::Borrowed(data.cmd_path.as_deref().unwrap_or("")),
                Cow::Borrowed(value_getter(data)),
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
                    .title(
                        Title::from(" Process Details ")
                            .alignment(Alignment::Left)
                            .position(Position::Top),
                    )
                    // .border_style(Style::new().fg(app.colors.footer_border_color))
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
    "ESC/<C+C> quit | <C+X> kill process | <C+R> refresh | <C+F> details forward | <C+B> details backward ";

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

fn layout_rects(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(7),
        Constraint::Length(1),
    ])
    .split(frame.area())
}
