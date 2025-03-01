use std::{collections::VecDeque, io, time::Duration};

use anyhow::Result;
use components::{
    Component, ComponentEvent, KeyAction, help_footer::HelpFooterComponent,
    processes_view::ProcessesViewComponent,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::style::{Color, Style, palette::tailwind};
use ratatui::{TerminalOptions, prelude::*};

pub mod components;
mod highlight;

use crate::settings::AppSettings;

struct App {
    components: Vec<Box<dyn Component>>,
    component_events: VecDeque<ComponentEvent>,
}

impl App {
    fn new(app_settings: AppSettings) -> Result<App> {
        let component_events = VecDeque::new();

        Ok(App {
            //order matters!
            //It should be according key input handling
            components: vec![
                Box::new(HelpFooterComponent::default()),
                Box::new(ProcessesViewComponent::new(
                    app_settings.use_icons,
                    app_settings.filter_opions,
                    app_settings.query,
                )?),
            ],
            component_events,
        })
    }

    fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            //NOTE: Why this order?
            // reading input is blocking and we want to have initial query rendered right away
            // along with process table
            self.render(terminal)?;

            if self.handle_events()? {
                return Ok(());
            }

            self.handle_input()?;
        }
    }

    fn handle_events(&mut self) -> Result<bool, io::Error> {
        while let Some(event) = self.component_events.pop_front() {
            if let ComponentEvent::QuitRequested = event {
                return Ok(true);
            }
            for component in self.components.iter_mut() {
                let new_event = component.handle_event(&event);
                if let Some(new_event) = new_event {
                    self.component_events.push_back(new_event);
                }
            }
        }
        Ok(false)
    }

    fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), io::Error> {
        terminal.draw(|frame| {
            let layout = LayoutRects::new(frame);
            for component in self.components.iter_mut() {
                component.render(frame, &layout);
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), io::Error> {
        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    //TODO: move it to some component that only is looking for exit input?
                    //maybe event to footer?
                    match key.code {
                        KeyCode::Esc => self
                            .component_events
                            .push_back(ComponentEvent::QuitRequested),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.component_events
                                .push_back(ComponentEvent::QuitRequested);
                        }
                        _ => (),
                    }
                    for component in self.components.iter_mut() {
                        let action = component.handle_input(key);
                        match action {
                            KeyAction::Unhandled => continue,
                            KeyAction::Consumed => {
                                break;
                            }
                            KeyAction::Event(act) => {
                                self.component_events.push_back(act);
                                break;
                            }
                        }
                    }
                }
            }
        };
        Ok(())
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
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    terminal.clear()?;

    //FIXME: add error handling, for exaple some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
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
