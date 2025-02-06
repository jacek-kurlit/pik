use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::Rect,
    text::Line,
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};

use crate::processes::Process;

use super::{Action, Component};

pub struct ProcessDetailsComponent {
    process_details_scroll_state: ScrollbarState,
    process_details_scroll_offset: u16,
    process_details_number_of_lines: u16,
    area_content_height: u16,
    selected_process: Option<Process>,
}

#[allow(clippy::new_without_default)]
impl ProcessDetailsComponent {
    pub fn new() -> Self {
        Self {
            process_details_scroll_offset: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            process_details_scroll_state: ScrollbarState::new(1),
            area_content_height: 0,
            selected_process: None,
        }
    }

    pub fn process_details_down(&mut self) {
        let content_scrolled =
            self.process_details_number_of_lines - self.process_details_scroll_offset;

        if content_scrolled > self.area_content_height {
            self.process_details_scroll_offset =
                self.process_details_scroll_offset.saturating_add(1);
        }
    }

    pub fn process_details_up(&mut self) {
        self.process_details_scroll_offset = self.process_details_scroll_offset.saturating_sub(1);
    }

    pub fn handle_process_select(&mut self, process: Process) {
        self.selected_process = Some(process);
        self.process_details_scroll_offset = 0;
    }

    fn update_process_details_number_of_lines(&mut self, area: Rect) {
        let content_width = area.width - 2;

        match self.selected_process.as_ref() {
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

impl Component for ProcessDetailsComponent {
    fn handle_input(&mut self, key: KeyEvent) -> Action {
        use KeyCode::*;
        match key.code {
            Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.process_details_down();
                Action::Consumed
            }
            Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.process_details_up();
                Action::Consumed
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        self.area_content_height = area.height - 2;
        let lines = process_details_lines(self.selected_process.as_ref());
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
        frame.render_widget(info_footer, area);
        self.update_process_details_number_of_lines(area);
        frame.render_stateful_widget(
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
}
