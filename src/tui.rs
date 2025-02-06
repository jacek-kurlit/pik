use std::io;

use anyhow::Result;
use components::{
    help_footer::HelpFooterComponent, process_details::ProcessDetailsComponent,
    process_table::ProcessTableComponent, search_bar::SearchBarComponent, Action, Component,
};
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::style::{palette::tailwind, Color, Style};
use ratatui::{prelude::*, TerminalOptions};

pub mod components;
mod highlight;

use crate::settings::AppSettings;

struct App {
    search_bar: SearchBarComponent,
    process_table: ProcessTableComponent,
    process_details: ProcessDetailsComponent,
    help_footer: HelpFooterComponent,
}

impl App {
    fn new(app_settings: AppSettings) -> Result<App> {
        let mut app = App {
            //FIXME: cloning hurts!
            search_bar: SearchBarComponent::new(app_settings.query.clone()),
            process_table: ProcessTableComponent::new(
                app_settings.use_icons,
                app_settings.filter_opions,
            )?,
            process_details: ProcessDetailsComponent::new(),
            help_footer: HelpFooterComponent::default(),
        };
        //FIXME: we are not setting process details
        //maybe it will be fixed with event driven approach?
        app.search_for_processess(app_settings.query.as_str());
        Ok(app)
    }

    fn search_for_processess(&mut self, search_text: &str) {
        self.help_footer.reset_error_message();
        self.process_table.search_for_processess(search_text);
    }
}

pub enum ProcessRelatedSearch {
    Family,   // process + process childrens
    Siblings, // process parent + all his children
    Parent,   // only parent process
}

pub fn start_app(app_settings: AppSettings) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let viewport = app_settings.viewport.clone();
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

    // create app and run it
    let app = App::new(app_settings)?;
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
        terminal.draw(|f| {
            let rects = LayoutRects::new(f);
            app.search_bar.render(f, rects.search_bar);
            app.process_table.render(f, rects.process_table);
            app.process_details.render(f, rects.process_details);
            app.help_footer.render(f, rects.help_footer);
        })?;

        if let Event::Key(key) = event::read()? {
            let components: Vec<&mut dyn Component> = vec![
                //order matters!
                &mut app.process_table,
                &mut app.process_details,
                &mut app.search_bar,
            ];
            if key.kind == KeyEventKind::Press {
                let mut actions = vec![];
                for c in components {
                    let action = c.handle_input(key);
                    match action {
                        Action::Consumed => break,
                        //TODO: i don't like this, special case for kill
                        Action::ProcessKilled | Action::NoProcessToKill => {
                            app.help_footer.reset_error_message();
                            break;
                        }
                        //TODO: this is sad
                        Action::SetSearchText(query) => {
                            actions.push(Action::SetSearchText(query));
                            break;
                        }
                        action => actions.push(action),
                    }
                }
                //TODO: refactor
                for action in actions {
                    match action {
                        Action::Quit => return Ok(()),
                        Action::SearchForProcesses(query) => app.search_for_processess(&query),
                        Action::ProcessSelected(prc) => {
                            app.process_details.handle_process_select(prc);
                        }
                        Action::SetSearchText(query) => {
                            app.search_bar.set_search_text(query);
                        }
                        Action::ProcessKillFailure => {
                            app.help_footer
                                .set_error_message("Failed to kill process. Check permissions");
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub struct LayoutRects {
    pub search_bar: Rect,
    pub process_table: Rect,
    pub process_details: Rect,
    pub help_footer: Rect,
}

impl LayoutRects {
    pub fn new(frame: &Frame) -> Self {
        let rects = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Max(7),
            Constraint::Length(1),
        ])
        .split(frame.area());
        Self {
            search_bar: rects[0],
            process_table: rects[1],
            process_details: rects[2],
            help_footer: rects[3],
        }
    }
}

pub struct Theme {
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub process_table_border_color: Color,
    pub highlight_style: Style,
    pub default_style: Style,
}

#[allow(clippy::new_without_default)]
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
