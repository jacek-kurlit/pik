use std::{collections::VecDeque, io, time::Duration};

use anyhow::Result;
use components::{
    Component, ComponentEvent, KeyAction, general_input_handler::GeneralInputHandlerComponent,
    help_footer::HelpFooterComponent, help_popup::HelpPopupComponent,
    processes_view::ProcessesViewComponent,
};
use ratatui::crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
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
            //Order matters!
            //Input handling is done in this order
            //Rendering is done in reverse
            //It allows for popups to be rendered on top but they handle input first
            components: vec![
                Box::new(HelpPopupComponent::new()),
                Box::new(GeneralInputHandlerComponent),
                Box::new(HelpFooterComponent::default()),
                Box::new(ProcessesViewComponent::new(
                    &app_settings.ui_config,
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
            // Reading input is blocking and we want to have initial query rendered right away
            // along with process table
            if self.handle_events()? {
                return Ok(());
            }

            self.render(terminal)?;

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
            for component in self.components.iter_mut().rev() {
                component.render(frame, &layout);
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), io::Error> {
        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
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
    Family,   // process + process children
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

    //FIXME: add error handling, for example some error page should be shown
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

pub struct LayoutRects {
    pub top_bar: Rect,
    pub process_table: Rect,
    pub process_details: Rect,
    pub footer: Rect,
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
            top_bar: rects[0],
            process_table: rects[1],
            process_details: rects[2],
            footer: rects[3],
        }
    }
}
