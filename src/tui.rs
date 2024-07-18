use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
    TerminalOptions, Viewport,
};
use style::palette::tailwind;

use crate::processes::{Process, ProcessManager};

const INFO_TEXT: &str = "ESC quit | CTRL + D kill process";

struct TableColors {
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    fn new() -> Self {
        Self {
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: tailwind::BLUE.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: tailwind::BLUE.c400,
        }
    }
}

struct App {
    state: TableState,
    process_manager: ProcessManager,
    processes: Vec<Process>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    search_criteria: String,
    character_index: usize,
}

impl App {
    fn new(search_criteria: String) -> Result<App> {
        let mut app = App {
            state: TableState::default(),
            process_manager: ProcessManager::new()?,
            processes: vec![],
            scroll_state: ScrollbarState::new(0),
            colors: TableColors::new(),
            search_criteria,
            character_index: 0,
        };
        app.search_for_processess();
        app.move_cursor_to_end();
        Ok(app)
    }
    pub fn next(&mut self) {
        let i = self.state.selected().map(|i| {
            let mut i = i + 1;
            if i >= self.processes.len() {
                i = 0
            }
            i
        });
        self.state.select(i);
        self.scroll_state = self.scroll_state.position(i.unwrap_or(0));
    }

    pub fn previous(&mut self) {
        let previous_index = self.state.selected().map(|i| {
            let i = i.wrapping_sub(1);
            i.clamp(0, self.processes.len().saturating_sub(1))
        });
        self.state.select(previous_index);
        self.scroll_state = self.scroll_state.position(previous_index.unwrap_or(0));
    }

    fn move_cursor_left(&mut self) {
        self.character_index = self.character_index.saturating_sub(1);
    }

    fn search_criteria_len(&self) -> usize {
        self.search_criteria.chars().count()
    }

    fn move_cursor_to_start(&mut self) {
        self.character_index = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.character_index = self.search_criteria_len();
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = cursor_moved_right.clamp(0, self.search_criteria_len())
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search_criteria.insert(index, new_char);
        self.move_cursor_right();
        self.search_for_processess();
    }

    fn search_for_processess(&mut self) {
        self.processes = self.process_manager.find_processes(&self.search_criteria);
        self.scroll_state = self
            .scroll_state
            .content_length(self.processes.len().saturating_sub(1));
        if self.processes.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.search_criteria
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.search_criteria.len())
    }

    fn delete_char(&mut self) {
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
            let before_char_to_delete = self
                .search_criteria
                .chars()
                .take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_criteria.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_criteria = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
        self.search_for_processess();
    }

    fn get_selected_process(&self) -> Option<&Process> {
        self.state.selected().map(|i| &self.processes[i])
    }

    fn kill_selected_process(&mut self) {
        if self.state.selected().is_none() {
            return;
        }
        let selected_row = self.state.selected().unwrap();
        if let Some(prc) = self.processes.get(selected_row) {
            self.process_manager.kill_process(prc.pid);
            self.processes.remove(selected_row);
            //FIXME: this is not refereshing I think there maybe issue with cache / process kill still being executed
            // self.processes = self.process_query.find_processes(&self.search_criteria);
        }
    }
}

pub fn start_tui_app(search_criteria: String) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(20),
        },
    )?;

    // create app and run it
    let app = App::new(search_criteria)?;
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    terminal.clear()?;

    //FIXME: add error handling, for exaple some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                use KeyCode::*;
                match key.code {
                    Esc => return Ok(()),
                    Up | BackTab => app.previous(),
                    Tab | Down => app.next(),
                    End => app.move_cursor_to_end(),
                    Home => app.move_cursor_to_start(),
                    Left => app.move_cursor_left(),
                    Right => app.move_cursor_right(),
                    Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.kill_selected_process()
                    }
                    Char(to_insert) => app.enter_char(to_insert),
                    Backspace => app.delete_char(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(7),
        Constraint::Length(1),
    ])
    .split(f.size());

    render_input(f, app, rects[0]);

    render_table(f, app, rects[1]);

    render_scrollbar(f, app, rects[1]);

    render_details(f, app, rects[2]);

    render_help(f, rects[3]);
}

fn render_input(f: &mut Frame, app: &mut App, area: Rect) {
    //TODO: use loop icon instead of '>'
    let prompt = "> ";
    let current_input = format!("{}{}", prompt, app.search_criteria);
    let input = Paragraph::new(current_input.as_str());
    f.render_widget(input, area);
    f.set_cursor(
        area.x + app.character_index as u16 + prompt.len() as u16,
        area.y,
    );
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default();
    // .fg(app.colors.header_fg)
    // .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = Row::new(vec![
        "USER", "PID", "STARTED", "TIME", "CMD", "CMD_PATH", "ARGS",
    ])
    .style(header_style);
    let rows = app.processes.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        //TODO: think about creating this row without allocations
        Row::new(vec![
            format!("{}", data.user_name),
            format!("{}", data.pid),
            format!("{}", data.start_time),
            format!("{}", data.run_time),
            format!("{}", data.cmd),
            format!("{}", data.cmd_path.as_deref().unwrap_or("")),
            format!("{}", data.args),
        ])
        .style(Style::new().fg(app.colors.row_fg).bg(color))
    });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(
                Title::from(format!(
                    " {} / {} ",
                    app.state.selected().map(|i| i + 1).unwrap_or(0),
                    app.processes.len()
                ))
                .position(block::Position::Top)
                .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_style(Style::new().fg(app.colors.footer_border_color))
            .border_type(BorderType::Plain),
    )
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![" ".into()]))
    // .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let lines = footer_lines(app);
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
                        .position(block::Position::Top),
                )
                // .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Rounded),
        );
    f.render_widget(info_footer, area);
}

fn footer_lines(app: &App) -> Vec<Line> {
    match app.get_selected_process() {
        Some(prc) => {
            let ports = prc
                .ports
                .as_deref()
                .map(|p| format!(" PORTS: {}", p))
                .unwrap_or("".to_string());
            vec![
                Line::from(format!(
                    "USER: {} PID: {} START_TIME: {}, RUN_TIME: {} MEMORY: {}MB{}",
                    prc.user_name,
                    prc.pid,
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

fn render_help(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(INFO_TEXT))
        .right_aligned()
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(help, area);
}
