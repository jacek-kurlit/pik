use std::io::{self};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};
use style::palette::tailwind;

use crate::processes::{Process, ProcessQuery};

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT: &str =
    "(q) quit | (k) move up | (j) move down | (l) next color | (h) previous color";

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

struct App {
    state: TableState,
    _process_query: ProcessQuery,
    processes: Vec<Process>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    search_criteria: String,
}

impl App {
    fn new(process_query: ProcessQuery, search_criteria: String) -> Result<App> {
        let processes = process_query.find_processes(&search_criteria)?;
        let scroll_size = processes.len() - 1;
        Ok(App {
            state: TableState::default().with_selected(0),
            _process_query: process_query,
            processes,
            scroll_state: ScrollbarState::new(scroll_size),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            search_criteria,
        })
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.processes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.processes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index])
    }
}

pub fn start_tui_app(search_criteria: String) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    // create app and run it
    let process_query = ProcessQuery::new();
    let app = App::new(process_query, search_criteria)?;
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

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
                    Right => app.next_color(),
                    Left => app.previous_color(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(f.size());

    app.set_colors();

    render_header(f, app, rects[0]);

    render_table(f, app, rects[1]);

    render_scrollbar(f, app, rects[1]);

    render_footer(f, app, rects[2]);
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let criteria = if app.search_criteria.is_empty() {
        "none"
    } else {
        app.search_criteria.as_str()
    };
    let header = Paragraph::new(format!("Criteria: '{}'", criteria))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::default()
                .title(
                    Title::from(Span::styled(
                        " Search criteria ",
                        Style::default().underline_color(Color::Red),
                    ))
                    .alignment(Alignment::Center),
                )
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Double),
        );
    f.render_widget(header, area);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = Row::new(vec!["USER", "PID", "CMD", "ARGS"]).style(header_style);
    let rows = app.processes.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        Row::new(vec![
            format!("{}", data.user_name),
            format!("{}", data.pid),
            format!("{}", data.cmd),
            format!("{}", data.args),
        ])
        .style(Style::new().fg(app.colors.row_fg).bg(color))
    });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(65),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(
                Title::from(format!(
                    " {} / {} ",
                    app.state.selected().unwrap_or(0) + 1,
                    app.processes.len()
                ))
                .position(block::Position::Bottom)
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::new().fg(app.colors.footer_border_color))
            .border_type(BorderType::Double),
    )
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![" ".into()]))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Double),
        );
    f.render_widget(info_footer, area);
}
