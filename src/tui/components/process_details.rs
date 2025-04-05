use ratatui::{
    prelude::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

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

    fn update_process_details_number_of_lines(
        &mut self,
        selected_process: Option<&Process>,
        area: Rect,
    ) {
        let content_width = area.width - 2;

        match selected_process.as_ref() {
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
