use std::{collections::VecDeque, io, time::Duration};

use anyhow::Result;
use components::{
    help_footer::HelpFooterComponent, process_details::ProcessDetailsComponent,
    process_table::ProcessTableComponent, search_bar::SearchBarComponent, Component,
    ComponentEvent, KeyAction,
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
    //NOTE: this is not the nices thing to return app and query string but it works
    fn new(app_settings: AppSettings) -> Result<(App, String)> {
        Ok((
            App {
                search_bar: SearchBarComponent::new(),
                process_table: ProcessTableComponent::new(
                    app_settings.use_icons,
                    app_settings.filter_opions,
                )?,
                process_details: ProcessDetailsComponent::new(),
                help_footer: HelpFooterComponent::default(),
            },
            app_settings.query,
        ))
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
    let (app, initial_query) = App::new(app_settings)?;
    let res = run_app(&mut terminal, app, initial_query);

    // restore terminal
    disable_raw_mode()?;
    terminal.clear()?;

    //FIXME: add error handling, for exaple some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    initial_query: String,
) -> io::Result<()> {
    let mut component_events = VecDeque::new();
    //NOTE: publish initial query so search_bar and search table will be updated right away
    component_events.push_front(ComponentEvent::SearchByTextRequested(initial_query));

    loop {
        //TODO: I dont like te order of this
        //it should be 1 events handling, render, key reads
        terminal.draw(|f| {
            let rects = LayoutRects::new(f);
            app.search_bar.render(f, rects.search_bar);
            app.process_table.render(f, rects.process_table);
            app.process_details.render(f, rects.process_details);
            app.help_footer.render(f, rects.help_footer);
        })?;
        let mut components: Vec<&mut dyn Component> = vec![
            //order matters!
            &mut app.process_table,
            &mut app.process_details,
            &mut app.search_bar,
        ];

        while let Some(event) = component_events.pop_front() {
            //TODO: not cool but what is the other way of doing this?
            //only to have some global state (which may be not a bad idea...)
            if let ComponentEvent::QuitRequested = event {
                return Ok(());
            }
            for component in components.iter_mut() {
                let new_event = component.handle_event(&event);
                if let Some(new_event) = new_event {
                    component_events.push_back(new_event);
                }
            }
        }
        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    for component in components.iter_mut() {
                        let action = component.handle_input(key);
                        match action {
                            KeyAction::Unhandled => continue,
                            KeyAction::Consumed => {
                                break;
                            }
                            KeyAction::Event(act) => {
                                component_events.push_back(act);
                                break;
                            }
                        }
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
