use ratatui::{
    prelude::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::{config::ui::ProcessDetailsTheme, processes::Process, tui::LayoutRects};

pub struct ProcessDetailsComponent {
    process_details_scroll_state: ScrollbarState,
    process_details_scroll_offset: u16,
    process_details_number_of_lines: u16,
    area_content_height: u16,
    theme: ProcessDetailsTheme,
}

#[allow(clippy::new_without_default)]
impl ProcessDetailsComponent {
    pub fn new(theme: ProcessDetailsTheme) -> Self {
        Self {
            process_details_scroll_offset: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            process_details_scroll_state: ScrollbarState::new(1),
            area_content_height: 0,
            theme,
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

    pub fn reset_details_scroll_offset(&mut self) {
        self.process_details_scroll_offset = 0;
    }

    fn calculate_wrapped_lines(text: &str, width: u16) -> u16 {
        if text.is_empty() {
            return 1;
        }
        let width = width as usize;

        // Simulate word wrapping similar to ratatui's Paragraph
        let mut line_count = 0;
        let mut current_line_width = 0;

        for word in text.split_inclusive(' ') {
            let word_width = word.width();

            if current_line_width == 0 {
                // First word on the line
                current_line_width = word_width;
                line_count += 1;
            } else if current_line_width + word_width <= width {
                // Word fits on current line
                current_line_width += word_width;
            } else {
                // Word needs a new line
                current_line_width = word_width;
                line_count += 1;
            }
        }

        line_count.max(1) as u16
    }

    fn update_process_details_number_of_lines(
        &mut self,
        selected_process: Option<&Process>,
        area: Rect,
    ) {
        let content_width = area.width - 2;

        match selected_process.as_ref() {
            Some(process) => {
                // rebuild to-be-rendered lines to calculate their wrapped height
                let ports = process
                    .ports
                    .as_deref()
                    .map(|p| format!(" PORTS: {p}"))
                    .unwrap_or("".to_string());
                let parent = process
                    .parent_pid
                    .map(|p| format!(" PARENT: {p}"))
                    .unwrap_or("".to_string());

                let line1 = format!(
                    "USER: {} PID: {}{} START TIME: {}, RUN TIME: {} MEMORY: {}MB{}",
                    process.user_name,
                    process.pid,
                    parent,
                    process.start_time,
                    process.run_time,
                    process.memory / 1024 / 1024,
                    ports,
                );
                let line2 = format!("CMD: {}", process.exe());
                let line3 = format!("ARGS: {}", process.args);

                let line1_wrapped = Self::calculate_wrapped_lines(&line1, content_width);
                let line2_wrapped = Self::calculate_wrapped_lines(&line2, content_width);
                let line3_wrapped = Self::calculate_wrapped_lines(&line3, content_width);

                self.process_details_number_of_lines =
                    line1_wrapped + line2_wrapped + line3_wrapped;
            }
            None => {
                self.process_details_number_of_lines = 1;
            }
        }
    }

    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        layout: &LayoutRects,
        selected_process: Option<&Process>,
    ) {
        let area = layout.process_details;
        self.area_content_height = area.height - 2;
        let lines = process_details_lines(selected_process);
        let details = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .left_aligned()
            .block(
                Block::default()
                    .title_position(self.theme.title.position)
                    .title_alignment(self.theme.title.alignment)
                    .title(" Process Details ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border.style)
                    .border_type(self.theme.border._type),
            )
            .scroll((self.process_details_scroll_offset, 0));
        frame.render_widget(details, area);
        self.update_process_details_number_of_lines(selected_process, area);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(self.theme.scrollbar.style)
                .thumb_symbol(self.theme.scrollbar.thumb_symbol.as_deref().unwrap_or(""))
                .track_symbol(self.theme.scrollbar.track_symbol.as_deref())
                .begin_symbol(self.theme.scrollbar.begin_symbol.as_deref())
                .end_symbol(self.theme.scrollbar.end_symbol.as_deref()),
            area.inner(self.theme.scrollbar.margin),
            &mut self.process_details_scroll_state,
        );
    }
}

fn process_details_lines<'a>(selected_process: Option<&'a Process>) -> Vec<Line<'a>> {
    match selected_process {
        Some(prc) => {
            let ports = prc
                .ports
                .as_deref()
                .map(|p| format!(" PORTS: {p}"))
                .unwrap_or("".to_string());
            let parent = prc
                .parent_pid
                .map(|p| format!(" PARENT: {p}"))
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
