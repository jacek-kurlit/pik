use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Style},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

use crate::processes::Process;

//TODO: this should be moved to a separate module
pub struct Theme {
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub process_table_border_color: Color,
    pub highlight_style: Style,
    pub default_style: Style,
}

impl Theme {
    pub fn new() -> Self {
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
    process_details_scroll_state: ScrollbarState,
    process_details_scroll_offset: u16,
    process_details_number_of_lines: u16,
}

impl Tui {
    pub fn new(_use_icons: bool) -> Self {
        Self {
            process_details_scroll_offset: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            process_details_scroll_state: ScrollbarState::new(1),
        }
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

    pub fn render_ui(
        &mut self,
        frame: &mut Frame,
        selected_process: Option<&Process>,
        rects: Rc<[Rect]>,
    ) {
        let this = &mut *self;
        let area = rects[2];
        let lines = process_details_lines(selected_process);

        this.update_process_details_number_of_lines(area, selected_process);

        let info_footer = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .left_aligned()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_top(Line::from(" Process Details ").left_aligned())
                    .border_type(BorderType::Rounded),
            )
            .scroll((this.process_details_scroll_offset, 0));
        frame.render_widget(info_footer, area);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .thumb_symbol("")
                .track_symbol(None)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut this.process_details_scroll_state,
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

//TODO: crate struct that will hold info about layout areas
fn layout_rects(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(7),
        Constraint::Length(1),
    ])
    .split(frame.area())
}
