use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Masked, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::{config::ui::ProcessDetailsTheme, processes::Process, tui::LayoutRects};

pub struct ProcessDetailsComponent {
    scroll_state: ScrollbarState,
    scroll: u16,
    process_details_number_of_lines: u16,
    area_content_width: u16,
    theme: ProcessDetailsTheme,
    details: Paragraph<'static>,
}

#[allow(clippy::new_without_default)]
impl ProcessDetailsComponent {
    pub fn new(theme: ProcessDetailsTheme) -> Self {
        Self {
            scroll: 0,
            process_details_number_of_lines: 0,
            //NOTE: we don't update this, value 1 means that this should be rendered
            scroll_state: ScrollbarState::new(1),
            area_content_width: 1,
            theme,
            details: Paragraph::new(""),
        }
    }

    //FIXME: ok cool now scroll does not work at all
    //I would rather rewrite it as it contains too many weird stuff going on
    pub fn process_details_down(&mut self) {
        // new
        self.scroll = self.scroll.saturating_add(1);
        self.scroll_state = self.scroll_state.position(self.scroll as usize);
        self.details = self.details.clone().scroll((self.scroll, 0));
        // // old logic
        // let content_scrolled = self.process_details_number_of_lines - self.scroll;
        // // println!(
        // //     "process_details_down - scroll_offset: {}, content_scrolled: {} area_content_height: {}",
        // //     self.process_details_scroll_offset, content_scrolled, self.area_content_height
        // // );
        //
        // if content_scrolled > self.area_content_height {
        //     self.scroll = self.scroll.saturating_add(1);
        // }
    }

    pub fn process_details_up(&mut self) {
        // new logic
        self.scroll = self.scroll.saturating_sub(1);
        self.scroll_state = self.scroll_state.position(self.scroll as usize);
        self.details = self.details.clone().scroll((self.scroll, 0));
    }

    pub fn select_new_process(&mut self, selected_process: Option<&Process>) {
        // self.scroll = 0;
        // self.update_process_details_number_of_lines(selected_process);
        self.details = self.create_details(selected_process);
    }

    fn update_process_details_number_of_lines(&mut self, selected_process: Option<&Process>) {
        match selected_process.as_ref() {
            Some(process) => {
                let args_number_of_lines =
                    (process.args.chars().count() as u16 / self.area_content_width) + 1;
                self.process_details_number_of_lines = args_number_of_lines + 2;
            }
            None => {
                self.process_details_number_of_lines = 1;
            }
        };

        self.scroll_state = self
            .scroll_state
            .content_length(self.process_details_number_of_lines as usize)
            .viewport_content_length(4);
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame, layout: &LayoutRects) {
        let area = layout.process_details;
        self.area_content_width = area.width - 2;

        frame.render_widget(&self.details, area);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(self.theme.scrollbar.style)
                .thumb_symbol(self.theme.scrollbar.thumb_symbol.as_deref().unwrap_or(""))
                .track_symbol(self.theme.scrollbar.track_symbol.as_deref())
                .begin_symbol(self.theme.scrollbar.begin_symbol.as_deref())
                .end_symbol(self.theme.scrollbar.end_symbol.as_deref()),
            area.inner(self.theme.scrollbar.margin),
            &mut self.scroll_state,
        );
    }

    fn create_details(&mut self, _selected_process: Option<&Process>) -> Paragraph<'static> {
        let s =
            "Veeeeeeeeeeeeeeeery    loooooooooooooooooong   striiiiiiiiiiiiiiiiiiiiiiiiiing.   ";
        let create_block = |title: &'static str| Block::bordered().gray().title(title.bold());
        let mut long_line = s.repeat(usize::from(self.area_content_width) / s.len() + 4);
        long_line.push('\n');
        let text = vec![
            Line::from("This is a line "),
            Line::from("This is a line   "),
            Line::from("This is a line"),
            Line::from("This is a longer line"),
            Line::from(long_line.clone()),
            Line::from("This is a line"),
            Line::from("This is a line "),
            Line::from("This is a line   "),
            Line::from("This is a line"),
            Line::from("This is a longer line"),
            Line::from(long_line.clone()),
            Line::from("This is a line"),
            Line::from(vec![
                Span::raw("Masked text: "),
                Span::styled(Masked::new("password", '*'), Style::new().fg(Color::Red)),
            ]),
        ];

        self.scroll_state = self.scroll_state.content_length(text.len());
        Paragraph::new(text.clone())
            .block(create_block("Vertical scrollbar with arrows"))
            .scroll((self.scroll, 0))
        // frame.render_widget(paragraph, chunks[1]);
        // frame.render_stateful_widget(
        //     Scrollbar::new(ScrollbarOrientation::VerticalRight)
        //         .begin_symbol(Some("↑"))
        //         .end_symbol(Some("↓")),
        //     chunks[1],
        //     &mut self.vertical_scroll_state,
        // );
        // let lines = process_details_lines(selected_process);
        // Paragraph::new(lines)
        //     .wrap(Wrap { trim: false })
        //     .left_aligned()
        //     .block(
        //         Block::default()
        //             .title_position(self.theme.title.position)
        //             .title_alignment(self.theme.title.alignment)
        //             .title(" Process Details ")
        //             .borders(Borders::ALL)
        //             .border_style(self.theme.border.style)
        //             .border_type(self.theme.border._type),
        //     )
        // .scroll((self.scroll, 0))
    }
}

fn process_details_lines(selected_process: Option<&Process>) -> Vec<Line<'static>> {
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
