use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
};

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
    character_index: usize,
    search_text: String,
    number_of_items: usize,
}

impl Tui {
    pub fn new(search_text: String) -> Self {
        let mut ui = Self {
            process_table: TableState::default(),
            process_table_scroll: ScrollbarState::new(0),
            theme: Theme::new(),
            character_index: 0,
            search_text,
            number_of_items: 0,
        };
        ui.move_search_cursor_to_end();
        ui
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
    }

    pub fn move_search_cursor_left(&mut self) {
        self.character_index = self.character_index.saturating_sub(1);
    }

    pub fn move_search_cursor_to_start(&mut self) {
        self.character_index = 0;
    }

    pub fn move_search_cursor_to_end(&mut self) {
        self.character_index = self.search_criteria_len();
    }

    pub fn move_search_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = cursor_moved_right.clamp(0, self.search_criteria_len())
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search_text.insert(index, new_char);
        self.move_search_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.character_index == 0 {
            return;
        }
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search_text.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_text.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_search_cursor_left();
        }
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
        &self.search_text
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.search_text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.search_text.len())
    }

    fn search_criteria_len(&self) -> usize {
        self.search_text.chars().count()
    }

    pub fn render_ui(&mut self, search_results: &ProcessSearchResults, frame: &mut Frame) {
        let rects = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Max(7),
            Constraint::Length(1),
        ])
        .split(frame.size());

        self.render_search_input(frame, rects[0]);

        self.render_process_table(frame, search_results, rects[1]);

        self.render_process_details(frame, search_results, rects[2]);

        render_help(frame, rects[3]);
    }

    fn render_search_input(&self, f: &mut Frame, area: Rect) {
        let prompt = "> ";
        let current_input = format!("{}{}", prompt, self.search_input_text());
        let input = Paragraph::new(current_input.as_str());
        f.render_widget(input, area);
        f.set_cursor(
            area.x + self.character_index as u16 + prompt.len() as u16,
            area.y,
        );
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
        self.render_procss_table_scrollbar(f, area);
    }

    fn render_procss_table_scrollbar(&mut self, f: &mut Frame, area: Rect) {
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
        &self,
        f: &mut Frame,
        search_results: &ProcessSearchResults,
        area: Rect,
    ) {
        let selected_process = search_results.nth(self.get_selected_row_index());
        let lines = process_details_lines(selected_process);
        let info_footer = Paragraph::new(lines)
            //TODO: i'm wrapping text but it still migt be too long to fit in details area
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
            );
        f.render_widget(info_footer, area);
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

const HELP_TEXT: &str = "ESC quit | CTRL + D kill process";

fn render_help(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(HELP_TEXT))
        .right_aligned()
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(help, area);
}
